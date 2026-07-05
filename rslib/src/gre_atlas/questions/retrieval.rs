// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Retrieval baselines and catalog-aware AI retrieval for the GRE Atlas eval
//! harness.
//!
//! Baselines (deterministic, no external APIs):
//! - **Keyword** — bag-of-words overlap between query tokens and source
//!   metadata
//! - **BM25** — Okapi BM25 over tokenized source documents
//! - **Vector (TF-IDF)** — cosine similarity between query and document TF-IDF
//!   vectors
//!
//! **AI retrieval** combines BM25 with GRE catalog labels, topic-path tokens,
//! and foundation exemplar overlap — the same signals used by the generation
//! pipeline.

use std::collections::HashMap;
use std::collections::HashSet;

use crate::gre_atlas::domain::GreCatalog;
use crate::gre_atlas::questions::ai_gen::keyword_overlap;
use crate::gre_atlas::questions::ai_gen::GoldEvalQuestion;
use crate::gre_atlas::questions::eval_pipeline::normalize_stem;
use crate::gre_atlas::questions::foundation::exemplars_for_topic;
use crate::gre_atlas::questions::source::SourceSection;
use crate::gre_atlas::questions::source::SOURCE_SECTIONS;

const BM25_K1: f32 = 1.2;
const BM25_B: f32 = 0.75;
const MIN_RETRIEVAL_SCORE: f32 = 0.01;

/// Which retrieval algorithm to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalMethod {
    Keyword,
    Bm25,
    VectorTfidf,
    AiEnhanced,
}

/// Outcome of retrieving a source section / topic for one gold question.
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub gold_id: String,
    pub predicted_topic: String,
    pub matched_section: String,
    pub score: f32,
    pub topic_match: bool,
    /// Share of gold keywords matched by the retrieved section metadata.
    pub keyword_recall: f32,
    /// True when the retriever could not produce a confident prediction.
    pub failed: bool,
}

/// Build the eval query from a held-out gold question (stem only — no oracle
/// keywords).
pub fn query_from_gold(gold: &GoldEvalQuestion) -> String {
    gold.stem.clone()
}

fn expanded_query_tokens(query: &str) -> HashSet<String> {
    let mut tokens = tokenize(query);
    let lower = query.to_ascii_lowercase();
    if lower.contains('%') || lower.contains("percent") {
        tokens.insert("percent".into());
    }
    if lower.contains("ratio") || lower.contains(':') {
        tokens.insert("ratio".into());
    }
    if lower.contains("equation") || lower.contains('=') {
        tokens.insert("equation".into());
        tokens.insert("linear".into());
    }
    if lower.contains("passage:") {
        tokens.insert("passage".into());
    }
    if lower.starts_with("issue:") {
        tokens.insert("issue".into());
    }
    if lower.starts_with("argument:") {
        tokens.insert("argument".into());
    }
    tokens
}

pub fn retrieve(gold: &GoldEvalQuestion, method: RetrievalMethod) -> RetrievalResult {
    let query = query_from_gold(gold);
    let query_tokens = expanded_query_tokens(&query);
    if query_tokens.is_empty() {
        return failed_result(gold, method, &query_tokens);
    }

    let corpus = build_corpus();
    let (section, score) = match method {
        RetrievalMethod::Keyword => keyword_score(&query_tokens),
        RetrievalMethod::Bm25 => bm25_score(&query_tokens, &corpus),
        RetrievalMethod::VectorTfidf => tfidf_score(&query_tokens, &corpus),
        RetrievalMethod::AiEnhanced => ai_score(&query, &query_tokens, &corpus),
    };

    let failed = score < MIN_RETRIEVAL_SCORE;
    let keyword_recall = section_keyword_recall(section, gold);
    RetrievalResult {
        gold_id: gold.id.clone(),
        predicted_topic: section.topic_id.to_string(),
        matched_section: section.section.to_string(),
        score,
        topic_match: section.topic_id == gold.topic,
        keyword_recall,
        failed,
    }
}

fn failed_result(
    gold: &GoldEvalQuestion,
    _method: RetrievalMethod,
    _query_tokens: &HashSet<String>,
) -> RetrievalResult {
    let fallback = &SOURCE_SECTIONS[0];
    RetrievalResult {
        gold_id: gold.id.clone(),
        predicted_topic: fallback.topic_id.to_string(),
        matched_section: fallback.section.to_string(),
        score: 0.0,
        topic_match: fallback.topic_id == gold.topic,
        keyword_recall: 0.0,
        failed: true,
    }
}

fn section_keyword_recall(section: &SourceSection, gold: &GoldEvalQuestion) -> f32 {
    let haystack = format!(
        "{} {} {}",
        section.section,
        section.excerpt,
        section.keywords.join(" ")
    );
    keyword_overlap(&haystack, &gold.keywords)
}

struct CorpusDoc {
    section: &'static SourceSection,
    tokens: Vec<String>,
    length: f32,
}

fn build_corpus() -> Vec<CorpusDoc> {
    SOURCE_SECTIONS
        .iter()
        .map(|section| {
            let text = document_text(section);
            let tokens = tokenize_to_vec(&text);
            let length = tokens.len() as f32;
            CorpusDoc {
                section,
                tokens,
                length,
            }
        })
        .collect()
}

fn document_text(section: &SourceSection) -> String {
    let catalog_text = GreCatalog::topic_by_id(section.topic_id)
        .map(|t| format!("{} {}", t.display_name, t.id))
        .unwrap_or_default();
    format!(
        "{} {} {} {}",
        section.section,
        section.excerpt,
        section.keywords.join(" "),
        catalog_text
    )
}

fn keyword_score(query_tokens: &HashSet<String>) -> (&'static SourceSection, f32) {
    let mut best: Option<(&SourceSection, f32)> = None;
    for section in SOURCE_SECTIONS {
        let doc_tokens = tokenize(&document_text(section));
        let overlap = query_tokens.intersection(&doc_tokens).count() as f32;
        let recall = overlap / query_tokens.len().max(1) as f32;
        if best.map_or(true, |(_, best_score)| recall > best_score) {
            best = Some((section, recall));
        }
    }
    best.unwrap_or((&SOURCE_SECTIONS[0], 0.0))
}

fn bm25_score(
    query_tokens: &HashSet<String>,
    corpus: &[CorpusDoc],
) -> (&'static SourceSection, f32) {
    let avg_dl = corpus.iter().map(|d| d.length).sum::<f32>() / corpus.len().max(1) as f32;
    let df = document_frequencies(corpus);
    let n = corpus.len() as f32;

    let mut best: Option<(&SourceSection, f32)> = None;
    for doc in corpus {
        let mut score = 0.0f32;
        for term in query_tokens {
            let tf = doc
                .tokens
                .iter()
                .filter(|t| t.as_str() == term.as_str())
                .count() as f32;
            if tf == 0.0 {
                continue;
            }
            let df_t = *df.get(term).unwrap_or(&0) as f32;
            let idf = ((n - df_t + 0.5) / (df_t + 0.5) + 1.0).ln();
            let denom = tf + BM25_K1 * (1.0 - BM25_B + BM25_B * doc.length / avg_dl.max(1.0));
            score += idf * (tf * (BM25_K1 + 1.0)) / denom;
        }
        if best.map_or(true, |(_, best_score)| score > best_score) {
            best = Some((doc.section, score));
        }
    }
    best.unwrap_or((&SOURCE_SECTIONS[0], 0.0))
}

fn tfidf_score(
    query_tokens: &HashSet<String>,
    corpus: &[CorpusDoc],
) -> (&'static SourceSection, f32) {
    let n = corpus.len() as f32;
    let df = document_frequencies(corpus);
    let query_vec = tfidf_vector(query_tokens, &df, n, 1.0);
    let mut best: Option<(&SourceSection, f32)> = None;

    for doc in corpus {
        let doc_tokens: HashSet<String> = doc.tokens.iter().cloned().collect();
        let doc_vec = tfidf_vector(&doc_tokens, &df, n, doc.length);
        let sim = cosine_similarity(&query_vec, &doc_vec);
        if best.map_or(true, |(_, best_score)| sim > best_score) {
            best = Some((doc.section, sim));
        }
    }
    best.unwrap_or((&SOURCE_SECTIONS[0], 0.0))
}

fn doc_bm25(
    query_tokens: &HashSet<String>,
    doc: &CorpusDoc,
    avg_dl: f32,
    df: &HashMap<String, u32>,
    n: f32,
) -> f32 {
    let mut score = 0.0f32;
    for term in query_tokens {
        let tf = doc
            .tokens
            .iter()
            .filter(|t| t.as_str() == term.as_str())
            .count() as f32;
        if tf == 0.0 {
            continue;
        }
        let df_t = *df.get(term).unwrap_or(&0) as f32;
        let idf = ((n - df_t + 0.5) / (df_t + 0.5) + 1.0).ln();
        let denom = tf + BM25_K1 * (1.0 - BM25_B + BM25_B * doc.length / avg_dl.max(1.0));
        score += idf * (tf * (BM25_K1 + 1.0)) / denom;
    }
    score
}

fn ai_score(
    query: &str,
    query_tokens: &HashSet<String>,
    corpus: &[CorpusDoc],
) -> (&'static SourceSection, f32) {
    let avg_dl = corpus.iter().map(|d| d.length).sum::<f32>() / corpus.len().max(1) as f32;
    let df = document_frequencies(corpus);
    let n = corpus.len() as f32;
    let (bm25_section, bm25_raw) = bm25_score(query_tokens, corpus);
    let mut best: Option<(&SourceSection, f32)> = None;

    for doc in corpus {
        let mut score = 0.0f32;
        let bm25_val = doc_bm25(query_tokens, doc, avg_dl, &df, n);
        score += bm25_val * 0.45;

        if let Some(topic) = GreCatalog::topic_by_id(doc.section.topic_id) {
            let catalog_tokens = tokenize(&format!("{} {}", topic.display_name, topic.id));
            let catalog_overlap = jaccard(query_tokens, &catalog_tokens);
            score += catalog_overlap * 0.20;

            let path_tokens = topic_path_tokens(topic.id);
            let path_overlap = jaccard(query_tokens, &path_tokens);
            score += path_overlap * 0.10;
        }

        let exemplar_overlap = exemplar_query_overlap(query, doc.section.topic_id);
        score += exemplar_overlap * 0.15;

        let section_recall = keyword_overlap(
            query,
            &doc.section
                .keywords
                .iter()
                .map(|k| (*k).to_string())
                .collect::<Vec<_>>(),
        );
        score += section_recall * 0.10;

        // Strong lexical anchors from the GRE catalog path (e.g.
        // verbal::reading::detail).
        let anchor_overlap = topic_path_tokens(doc.section.topic_id)
            .intersection(query_tokens)
            .count() as f32;
        score += anchor_overlap * 0.08;

        // Tie-break toward BM25 winner when scores are close.
        if doc.section.topic_id == bm25_section.topic_id {
            score += bm25_raw * 0.05;
        }

        if best.map_or(true, |(_, best_score)| score > best_score) {
            best = Some((doc.section, score));
        }
    }
    best.unwrap_or((bm25_section, bm25_raw))
}

fn exemplar_query_overlap(query: &str, topic_id: &str) -> f32 {
    let exemplars = exemplars_for_topic(topic_id);
    if exemplars.is_empty() {
        return 0.0;
    }
    let mut best = 0.0f32;
    for exemplar in exemplars {
        let stem_tokens = tokenize(exemplar.stem_text());
        let query_tokens = tokenize(query);
        best = best.max(jaccard(&query_tokens, &stem_tokens));
    }
    best
}

fn topic_path_tokens(topic_id: &str) -> HashSet<String> {
    topic_id.split("::").flat_map(tokenize).collect()
}

fn document_frequencies(corpus: &[CorpusDoc]) -> HashMap<String, u32> {
    let mut df: HashMap<String, u32> = HashMap::new();
    for doc in corpus {
        let unique: HashSet<String> = doc.tokens.iter().cloned().collect();
        for term in unique {
            *df.entry(term).or_insert(0) += 1;
        }
    }
    df
}

fn tfidf_vector(
    tokens: &HashSet<String>,
    df: &HashMap<String, u32>,
    corpus_size: f32,
    doc_length: f32,
) -> HashMap<String, f32> {
    let mut vec = HashMap::new();
    for term in tokens {
        let tf = tokens.iter().filter(|t| *t == term).count() as f32 / doc_length.max(1.0);
        let df_t = *df.get(term).unwrap_or(&0) as f32;
        let idf = (corpus_size / (df_t + 1.0)).ln_1p();
        vec.insert(term.clone(), tf * idf);
    }
    vec
}

fn cosine_similarity(a: &HashMap<String, f32>, b: &HashMap<String, f32>) -> f32 {
    let mut dot = 0.0f32;
    for (term, weight) in a {
        if let Some(other) = b.get(term) {
            dot += weight * other;
        }
    }
    let norm_a = a.values().map(|v| v * v).sum::<f32>().sqrt();
    let norm_b = b.values().map(|v| v * v).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count() as f32;
    let union = a.union(b).count() as f32;
    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

pub fn tokenize(text: &str) -> HashSet<String> {
    tokenize_to_vec(text).into_iter().collect()
}

fn tokenize_to_vec(text: &str) -> Vec<String> {
    normalize_stem(text)
        .split_whitespace()
        .filter(|t| t.len() > 1 || t.chars().all(|c| c.is_ascii_digit()))
        .map(|t| t.to_string())
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::gre_atlas::questions::ai_gen::load_gold_eval_set;

    #[test]
    fn ai_retrieval_beats_keyword_on_gold_set() {
        let gold = load_gold_eval_set().unwrap();
        let ai_acc = accuracy(&gold.questions, RetrievalMethod::AiEnhanced);
        let kw_acc = accuracy(&gold.questions, RetrievalMethod::Keyword);
        assert!(
            ai_acc >= kw_acc,
            "AI retrieval accuracy {ai_acc} should be >= keyword baseline {kw_acc}"
        );
    }

    #[test]
    fn ai_retrieval_beats_bm25_on_gold_set() {
        let gold = load_gold_eval_set().unwrap();
        let ai_acc = accuracy(&gold.questions, RetrievalMethod::AiEnhanced);
        let bm25_acc = accuracy(&gold.questions, RetrievalMethod::Bm25);
        assert!(
            ai_acc >= bm25_acc,
            "AI retrieval accuracy {ai_acc} should be >= BM25 baseline {bm25_acc}"
        );
    }

    #[test]
    fn all_methods_run_on_sample_question() {
        let gold = GoldEvalQuestion {
            id: "sample".into(),
            topic: "gre::quant::algebra::linear".into(),
            section: "quant".into(),
            stem: "If 3x + 7 = 22, what is the value of x?".into(),
            keywords: vec!["linear".into(), "equation".into()],
            correct_answer: "5".into(),
        };
        for method in [
            RetrievalMethod::Keyword,
            RetrievalMethod::Bm25,
            RetrievalMethod::VectorTfidf,
            RetrievalMethod::AiEnhanced,
        ] {
            let result = retrieve(&gold, method);
            assert!(!result.predicted_topic.is_empty(), "{method:?}");
        }
    }

    fn accuracy(gold: &[GoldEvalQuestion], method: RetrievalMethod) -> f32 {
        let matches = gold
            .iter()
            .filter(|q| retrieve(q, method).topic_match)
            .count();
        matches as f32 / gold.len() as f32
    }
}
