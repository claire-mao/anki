// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Parameterized template variants derived from the seed question patterns.

use std::collections::HashSet;

use crate::gre_atlas::domain::GreSection;
use crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft;
use crate::gre_atlas::questions::ai_gen::QuestionAttribution;
use crate::gre_atlas::questions::source::SourceSection;
use crate::gre_atlas::questions::source::GENERATION_SOURCE_NAME;
use crate::timestamp::TimestampSecs;

pub(crate) fn build_variant_draft(
    topic_id: &str,
    gre_section: GreSection,
    source: &SourceSection,
    variant: u32,
    now: TimestampSecs,
) -> GeneratedQuestionDraft {
    let section = gre_section.slug();
    let attribution = QuestionAttribution {
        source_name: GENERATION_SOURCE_NAME.into(),
        source_section: source.section.into(),
        generated_at_secs: now.0,
    };

    match topic_id {
        "gre::quant::arithmetic::percent" => {
            let discount_scenarios = [
                (200, 15, "$170", "15% of $200 is $30; $200 − $30 = $170."),
                (80, 25, "$60", "25% of $80 is $20; $80 − $20 = $60."),
                (50, 30, "$35", "30% of $50 is $15; $50 − $15 = $35."),
                (120, 10, "$108", "10% of $120 is $12; $120 − $12 = $108."),
            ];
            let increase_scenarios = [
                (
                    "A price increases from $50 to $65. What is the percent increase?",
                    vec!["15%", "20%", "30%", "35%"],
                    "30%",
                    "The increase is $15 on $50, a 30 percent increase.",
                ),
                (
                    "A salary rises from $40,000 to $46,000. What is the percent increase?",
                    vec!["10%", "12%", "15%", "20%"],
                    "15%",
                    "The $6,000 increase on $40,000 is a 15 percent increase.",
                ),
            ];
            if variant % 2 == 1 {
                let idx = (variant as usize / 2) % increase_scenarios.len();
                let (stem, choices, answer, explanation) = &increase_scenarios[idx];
                return mcq(
                    topic_id,
                    section,
                    variant,
                    now,
                    stem,
                    str_choices(choices),
                    answer,
                    explanation,
                    0.35,
                    attribution,
                );
            }
            let (price, pct, answer, explanation) =
                discount_scenarios[(variant as usize / 2) % discount_scenarios.len()];
            let wrong = [
                format!("${}", price - pct),
                format!("${}", price + pct),
                format!("${}", price),
            ];
            mcq(
                topic_id,
                section,
                variant,
                now,
                &format!(
                    "A laptop priced at ${price} \
                     is discounted by {pct}%. What is the sale price?",
                ),
                four_choices(
                    answer,
                    &[wrong[0].clone(), wrong[1].clone(), wrong[2].clone()],
                ),
                answer,
                explanation,
                0.35,
                attribution,
            )
        }
        "gre::quant::arithmetic::ratio" => {
            let scenarios = [
                (2, 3, 12, 18, "12 red is 2 parts, so one part is 6. Three parts of blue gives 18."),
                (3, 5, 24, 40, "24 boys represent 3 parts, so one part is 8. Five parts of girls gives 40."),
                (2, 3, 6, 9, "6 cups of flour equal 2 parts, so 1 part is 3 cups. Sugar is 3 parts = 9 cups."),
                (4, 7, 20, 35, "20 is 4 parts, so one part is 5. Seven parts gives 35."),
                (5, 8, 15, 24, "15 is 5 parts, so one part is 3. Eight parts gives 24."),
            ];
            let (a, b, given, answer, explanation) =
                scenarios[(variant as usize) % scenarios.len()];
            mcq(
                topic_id,
                section,
                variant,
                now,
                &format!(
                    "If the ratio of A to B is \
                     {a}:{b} and A equals {given}, what is B?",
                ),
                four_int_choices(answer, [-8, -5, 10]),
                &answer.to_string(),
                explanation,
                0.4,
                attribution,
            )
        }
        "gre::quant::algebra::linear" => {
            let scenarios = [
                (
                    4,
                    9,
                    11,
                    5,
                    "Solve the linear equation: add 9 to both sides, 4x = 20, divide by the coefficient 4 to isolate the variable x = 5.",
                ),
                (
                    3,
                    7,
                    22,
                    5,
                    "Solve the linear equation: subtract 7 from both sides, 3x = 15, divide by 3 to isolate the variable x = 5.",
                ),
                (
                    2,
                    5,
                    11,
                    8,
                    "Solve the linear equation: add 5 to both sides, 2x = 16, divide by 2 to isolate the variable x = 8.",
                ),
                (
                    5,
                    2,
                    20,
                    6,
                    "Solve the linear equation: divide both sides by 5, x − 2 = 4, so the variable x = 6.",
                ),
                (
                    6,
                    4,
                    34,
                    5,
                    "Solve the linear equation: subtract 4 from both sides, 6x = 30, divide by 6 to isolate the variable x = 5.",
                ),
            ];
            let (a, b, c, answer, explanation) = scenarios[(variant as usize) % scenarios.len()];
            mcq(
                topic_id,
                section,
                variant,
                now,
                &format!("Solve this linear equation for the variable x: {a}x + {b} = {c}.",),
                four_int_choices(answer, [-3, -1, 2]),
                &answer.to_string(),
                explanation,
                0.3,
                attribution,
            )
        }
        "gre::quant::algebra::quadratic" => {
            let scenarios = [
                (
                    "What is the positive root of x² − x − 6 = 0?",
                    vec!["2", "3", "5", "6"],
                    "3",
                    "Factor: (x − 3)(x + 2) = 0. Roots are 3 and −2; the positive root is 3.",
                ),
                (
                    "If x² = 49 and x > 0, what is the value of x?",
                    vec!["5", "6", "7", "8"],
                    "7",
                    "The positive square root of 49 is 7.",
                ),
                (
                    "What are the solutions to (x − 3)(x + 4) = 0?",
                    vec![
                        "x = 3 or x = −4",
                        "x = −3 or x = 4",
                        "x = 3 only",
                        "x = −4 only",
                    ],
                    "x = 3 or x = −4",
                    "Set each factor equal to 0: x = 3 or x = −4.",
                ),
                (
                    "What is the positive root of x² − 9 = 0?",
                    vec!["2", "3", "4", "5"],
                    "3",
                    "x² = 9, so x = 3 or x = −3; the positive root is 3.",
                ),
                (
                    "What is the positive root of x² − 5x + 6 = 0?",
                    vec!["2", "3", "4", "6"],
                    "3",
                    "Factor: (x − 2)(x − 3) = 0. Roots are 2 and 3; the positive root is 3.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.45,
                attribution,
            )
        }
        "gre::quant::geometry::triangles" => {
            let scenarios = [
                (5, 12, 13, "√(5² + 12²) = √169 = 13."),
                (6, 8, 10, "√(6² + 8²) = √100 = 10."),
                (9, 12, 15, "√(9² + 12²) = √225 = 15."),
                (8, 15, 17, "√(8² + 15²) = √289 = 17."),
                (7, 24, 25, "√(7² + 24²) = √625 = 25."),
            ];
            let (a, b, answer, explanation) = scenarios[(variant as usize) % scenarios.len()];
            mcq(
                topic_id,
                section,
                variant,
                now,
                &format!(
                    "A right triangle has legs \
                     {a} and {b}. What is the hypotenuse?",
                ),
                four_int_choices(answer, [-2, -1, 3]),
                &answer.to_string(),
                explanation,
                0.4,
                attribution,
            )
        }
        "gre::quant::geometry::circles" => {
            let scenarios = [
                (
                    "A circle has radius 4. What is its area?",
                    vec!["12π", "14π", "16π", "20π"],
                    "16π",
                    "Area = πr² = π(4)² = 16π.",
                ),
                (
                    "A circle has radius 5. What is its area?",
                    vec!["20π", "22π", "25π", "30π"],
                    "25π",
                    "Area = πr² = π(5²) = 25π.",
                ),
                (
                    "A circle has radius 4. What is its circumference?",
                    vec!["6π", "8π", "10π", "12π"],
                    "8π",
                    "Circumference = 2πr = 2π(4) = 8π.",
                ),
                (
                    "A circle has diameter 10. What is its circumference?",
                    vec!["8π", "9π", "10π", "12π"],
                    "10π",
                    "Circumference = πd = π(10) = 10π.",
                ),
                (
                    "A circle has radius 6. What is its circumference?",
                    vec!["10π", "11π", "12π", "14π"],
                    "12π",
                    "Circumference = 2πr = 2π(6) = 12π.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.4,
                attribution,
            )
        }
        "gre::quant::data_interpretation" => {
            let scenarios = [
                (
                    "According to the chart, revenue rose from $50M to $65M. What is the percent increase?",
                    vec!["15%", "25%", "30%", "35%"],
                    "30%",
                    "The chart shows an increase of $15M on $50M → 15/50 = 30%.",
                ),
                (
                    "The table lists monthly sales of $20K, $30K, and $50K. What is the three-month total?",
                    vec!["$80K", "$90K", "$100K", "$110K"],
                    "$100K",
                    "Add the table values: 20 + 30 + 50 = 100, so $100K.",
                ),
                (
                    "A chart shows 25% of a $200 budget goes to marketing. How much is that?",
                    vec!["$25", "$40", "$50", "$60"],
                    "$50",
                    "0.25 × $200 = $50 according to the chart.",
                ),
                (
                    "Compare the chart values: Company A revenue grew 25% while Company B grew 20%. Which grew faster?",
                    vec!["Company A", "Company B", "Both equally", "Cannot determine"],
                    "Company A",
                    "Comparing percent increases, 25% exceeds 20%, so Company A grew faster.",
                ),
                (
                    "A table shows expenses of $40K, $55K, and $65K over three quarters. What is the total?",
                    vec!["$140K", "$150K", "$160K", "$170K"],
                    "$160K",
                    "Add the table values: 40 + 55 + 65 = 160, so $160K.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.5,
                attribution,
            )
        }
        "gre::quant::statistics::probability" => {
            let scenarios = [
                (
                    "What is the probability that a randomly chosen marble from a bag with 3 red and 7 blue marbles is red?",
                    vec!["1/10", "3/10", "3/7", "7/10"],
                    "3/10",
                    "Probability is 3 favorable outcomes out of 10 total → 3/10.",
                ),
                (
                    "A fair die is rolled once. What is the probability of an even outcome?",
                    vec!["1/6", "1/3", "1/2", "2/3"],
                    "1/2",
                    "Three of six equally likely outcomes are even → 3/6 = 1/2.",
                ),
                (
                    "Two fair coins are flipped. What is the probability of both heads?",
                    vec!["1/8", "1/4", "1/3", "1/2"],
                    "1/4",
                    "Independent events: probability 1/2 × 1/2 = 1/4.",
                ),
                (
                    "A bag has 2 red and 3 blue marbles. What is the probability of drawing red?",
                    vec!["1/5", "2/5", "3/5", "2/3"],
                    "2/5",
                    "2 favorable outcomes out of 5 total → 2/5.",
                ),
                (
                    "A spinner has 5 equal sections numbered 1 through 5. What is the probability of landing on an odd number?",
                    vec!["1/5", "2/5", "3/5", "4/5"],
                    "3/5",
                    "Odd outcomes 1, 3, and 5 are three of five equally likely sections.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.35,
                attribution,
            )
        }
        "gre::quant::statistics::data_analysis" => {
            let scenarios = [
                (
                    "What is the median of 4, 9, 11, 15, 22?",
                    vec!["9", "11", "12", "15"],
                    "11",
                    "The middle value of the sorted list is 11.",
                ),
                (
                    "What is the mean of 4, 8, 10, 10, and 8?",
                    vec!["6", "7", "8", "9"],
                    "8",
                    "The sum is 40 and there are 5 values, so the mean is 8.",
                ),
                (
                    "What is the median of 3, 7, 9, 12, 15?",
                    vec!["7", "8", "9", "12"],
                    "9",
                    "With five ordered values, the median is the middle value, 9.",
                ),
                (
                    "What is the mode of 2, 3, 3, 5, 7?",
                    vec!["2", "3", "5", "7"],
                    "3",
                    "The mode is the value that appears most often, which is 3.",
                ),
                (
                    "What is the range of 5, 8, 12, 12, and 20?",
                    vec!["8", "12", "15", "20"],
                    "15",
                    "Range is maximum minus minimum: 20 − 5 = 15.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.35,
                attribution,
            )
        }
        "gre::quant::word_problems" => {
            let scenarios = [
                (
                    "A train travels 120 miles in 2 hours. What is its average speed?",
                    vec!["40 mph", "50 mph", "60 mph", "70 mph"],
                    "60 mph",
                    "Speed = distance / time = 120 / 2 = 60 mph.",
                ),
                (
                    "If 5 pens cost $10, how much do 8 pens cost at the same unit price?",
                    vec!["$12", "$14", "$16", "$18"],
                    "$16",
                    "Each pen costs $2, so 8 pens cost $16.",
                ),
                (
                    "The sum of two consecutive integers is 27. What is the larger integer?",
                    vec!["12", "13", "14", "15"],
                    "14",
                    "The integers are 13 and 14, so the larger is 14.",
                ),
                (
                    "A car travels 180 miles in 3 hours. What is its average speed?",
                    vec!["50 mph", "55 mph", "60 mph", "65 mph"],
                    "60 mph",
                    "Speed = 180 / 3 = 60 mph.",
                ),
                (
                    "Worker A completes a job in 6 hours and Worker B in 3 hours. How long do they take working together?",
                    vec!["1 hour", "2 hours", "3 hours", "4 hours"],
                    "2 hours",
                    "Combined rate is 1/6 + 1/3 = 1/2 job per hour, so the job takes 2 hours.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.35,
                attribution,
            )
        }
        "gre::quant::number_properties" => {
            let scenarios = [
                (
                    "What is the remainder when 47 is divided by 6?",
                    vec!["3", "4", "5", "6"],
                    "5",
                    "47 = 6×7 + 5, so the remainder is 5.",
                ),
                (
                    "What is the greatest common divisor of 12 and 18?",
                    vec!["3", "4", "6", "9"],
                    "6",
                    "The greatest shared divisor of 12 and 18 is 6.",
                ),
                (
                    "What is the smallest prime number greater than 10?",
                    vec!["11", "12", "13", "15"],
                    "11",
                    "11 is the first prime above 10.",
                ),
                (
                    "Which of the following is a prime number: 51, 53, 55, or 57?",
                    vec!["51", "53", "55", "57"],
                    "53",
                    "53 has no factors other than 1 and itself.",
                ),
                (
                    "How many positive divisors does 36 have?",
                    vec!["6", "7", "8", "9"],
                    "9",
                    "36 = 2² × 3², so it has (2+1)(2+1) = 9 positive divisors.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.35,
                attribution,
            )
        }
        "gre::verbal::text_completion" => {
            let scenarios = [
                (
                    "In context, the committee's report was so ______ that even dissenting members accepted its conclusions.",
                    vec!["equivocal", "persuasive", "opaque", "fragmentary"],
                    "persuasive",
                    "Dissenters accepting conclusions implies the report was convincing in context.",
                ),
                (
                    "Given the logical contrast in context, despite her ______ manner in public, the negotiator was famously ruthless at the bargaining table.",
                    vec!["genial", "abrasive", "hostile", "brusque"],
                    "genial",
                    "\"Despite\" signals a contrast with ruthlessness in context.",
                ),
                (
                    "In context, the scientist's findings were so ______ that even skeptical reviewers revised their conclusions.",
                    vec!["equivocal", "compelling", "peripheral", "tenuous"],
                    "compelling",
                    "Skeptics changing their minds implies convincing evidence in context.",
                ),
                (
                    "From context, the critic's review was so ______ that the author felt encouraged rather than discouraged.",
                    vec!["scathing", "laudatory", "dismissive", "caustic"],
                    "laudatory",
                    "Feeling encouraged points to supportive praise in context.",
                ),
                (
                    "Using context clues, although the instructions were ______, the team completed the assembly without errors.",
                    vec!["ambiguous", "lucid", "cryptic", "obscure"],
                    "lucid",
                    "Context clues show successful completion implies clear instructions.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.5,
                attribution,
            )
        }
        "gre::verbal::sentence_equivalence" => {
            let scenarios = [
                (
                    "Select two equivalent words: the historian's account was surprisingly ______, given the contentious subject matter.",
                    vec![
                        "dispassionate", "inflammatory", "neutral", "biased", "polemical",
                        "temperate",
                    ],
                    "dispassionate",
                    "Surprisingly calm tone fits dispassionate; equivalent meaning to neutral tone.",
                ),
                (
                    "Select two equivalent words: although the CEO's apology was ______, many employees remained skeptical.",
                    vec!["heartfelt", "perfunctory", "sincere", "grudging", "elaborate", "cursory"],
                    "heartfelt",
                    "Equivalent meaning to sincere tone contrasts with employee skepticism.",
                ),
                (
                    "Select two equivalent words: the valley's ______ rainfall supported unusually dense forests.",
                    vec!["scarce", "abundant", "meager", "plentiful", "sparse", "erratic"],
                    "abundant",
                    "Plentiful is an equivalent synonym for abundant rainfall.",
                ),
                (
                    "Select two equivalent words: her explanation was admirably ______, conveying the entire idea in just a few words.",
                    vec!["verbose", "succinct", "rambling", "concise", "elaborate", "tedious"],
                    "succinct",
                    "Concise is an equivalent synonym for succinct wording.",
                ),
                (
                    "Select two equivalent words: the diplomat's remarks were deliberately ______, leaving room for later negotiation.",
                    vec!["explicit", "ambiguous", "vague", "precise", "candid", "evasive"],
                    "ambiguous",
                    "Vague is an equivalent synonym for ambiguous wording in diplomacy.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            GeneratedQuestionDraft {
                id: new_generated_id(topic_id, variant, now),
                topic: topic_id.into(),
                section: section.into(),
                format: "sentence_equivalence".into(),
                stem: (*stem).into(),
                choices: to_string_vec(choices),
                correct_answer: (*answer).into(),
                explanation: (*explanation).into(),
                difficulty: Some(0.55),
                confidence: 0.0,
                attribution,
            }
        }
        "gre::verbal::reading::inference" => {
            let scenarios = [
                (
                    "Passage: Cities that expand transit see fewer solo car commutes, but housing near stations often becomes more expensive. Which inference is best supported?",
                    vec![
                        "Transit expansion always lowers housing costs.",
                        "Convenience may trade off with affordability.",
                        "Commutes are unaffected by transit.",
                        "Housing prices are unrelated to transit.",
                    ],
                    "Convenience may trade off with affordability.",
                    "The passage links reduced commutes with higher nearby housing costs.",
                ),
                (
                    "Passage: A policy reduced factory emissions, yet nearby residents reported little change because pollution shifted downwind. Which is best inferred?",
                    vec![
                        "The policy eliminated all pollution.",
                        "Emissions controls can relocate pollution rather than remove it.",
                        "Residents were mistaken about the air quality.",
                        "Downwind communities have no pollution.",
                    ],
                    "Emissions controls can relocate pollution rather than remove it.",
                    "Pollution shifting downwind implies it moved rather than disappeared.",
                ),
                (
                    "Passage: Mixed-use zoning reduces commute times but critics say it raises housing costs. Which inference is best supported?",
                    vec![
                        "Zoning always lowers housing costs.",
                        "Convenience may trade off with affordability.",
                        "Critics deny any commute benefit.",
                        "Housing prices are unrelated to zoning.",
                    ],
                    "Convenience may trade off with affordability.",
                    "The passage contrasts commute benefits with higher housing costs.",
                ),
                (
                    "Passage: After the library extended evening hours, attendance rose but wait times for study rooms increased. Which inference is best supported?",
                    vec![
                        "Longer hours always eliminate crowding.",
                        "Higher use can create new bottlenecks.",
                        "Attendance is unrelated to hours.",
                        "Study rooms are unnecessary.",
                    ],
                    "Higher use can create new bottlenecks.",
                    "More attendance paired with longer waits suggests demand can outpace capacity.",
                ),
                (
                    "Passage: A drought-resistant crop failed in one region because soil chemistry differed from test plots. Which is best inferred?",
                    vec![
                        "The crop works everywhere equally.",
                        "Local conditions can limit generalizations from trials.",
                        "Soil chemistry never matters.",
                        "Test plots are always representative.",
                    ],
                    "Local conditions can limit generalizations from trials.",
                    "Failure outside test conditions suggests results may not transfer directly.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.55,
                attribution,
            )
        }
        "gre::verbal::reading::main_idea" => {
            let scenarios = [
                (
                    "Passage: Coral reefs support diverse fisheries, but warming oceans cause bleaching that collapses those food webs. What is the main idea?",
                    vec![
                        "Fisheries are unrelated to coral health.",
                        "Ocean warming threatens reef ecosystems and dependent fisheries.",
                        "Bleaching improves biodiversity.",
                        "Coral reefs exist only in cold water.",
                    ],
                    "Ocean warming threatens reef ecosystems and dependent fisheries.",
                    "The passage connects warming, bleaching, and fishery collapse.",
                ),
                (
                    "Passage: Modern observatories combine light from many instruments and reconstruct images no single lens could form. What is the main idea?",
                    vec![
                        "Telescopes exist only to magnify objects.",
                        "Modern astronomy depends on combining and computing data.",
                        "Early telescopes were useless.",
                        "Observatories are too expensive to build.",
                    ],
                    "Modern astronomy depends on combining and computing data.",
                    "The passage contrasts simple magnification with computational combination.",
                ),
                (
                    "Passage: Early conservationists saved species by protecting habitats, but climate change now forces relocation of entire ecosystems. What is the main idea?",
                    vec![
                        "Habitat protection is unnecessary.",
                        "Climate change is forcing conservation strategies to adapt.",
                        "Species never need relocation.",
                        "Conservation has always failed.",
                    ],
                    "Climate change is forcing conservation strategies to adapt.",
                    "The passage moves from habitat protection to forced relocation.",
                ),
                (
                    "Passage: Historians once treated letters as private artifacts, but digitization now lets scholars search thousands at once. What is the main idea?",
                    vec![
                        "Letters were never useful to historians.",
                        "Digital tools are changing how historians access primary sources.",
                        "Digitization destroys original documents.",
                        "Scholars no longer read individual letters.",
                    ],
                    "Digital tools are changing how historians access primary sources.",
                    "The passage contrasts old limits with searchable archives.",
                ),
                (
                    "Passage: Startups tout flexible schedules, yet many employees report longer total hours once commutes are replaced by always-on messaging. What is the main idea?",
                    vec![
                        "Flexible schedules always reduce work hours.",
                        "Remote flexibility can blur boundaries between work and personal time.",
                        "Commutes have no effect on productivity.",
                        "Messaging tools eliminate the need for schedules.",
                    ],
                    "Remote flexibility can blur boundaries between work and personal time.",
                    "The passage contrasts promised flexibility with longer effective hours.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.55,
                attribution,
            )
        }
        "gre::verbal::reading::detail" => {
            let scenarios = [
                (
                    "Passage: The trial lasted twelve weeks and included 240 participants. How long did the trial last?",
                    vec!["eight weeks", "ten weeks", "twelve weeks", "twenty weeks"],
                    "twelve weeks",
                    "The passage explicitly states twelve weeks.",
                ),
                (
                    "Passage: The 1876 Centennial Exhibition introduced the telephone to a mass audience. What was introduced?",
                    vec!["The telegraph", "The telephone", "The radio", "The light bulb"],
                    "The telephone",
                    "The passage states directly that the telephone was introduced.",
                ),
                (
                    "Passage: The study enrolled 180 patients across four clinics. How many patients were enrolled?",
                    vec!["120", "160", "180", "240"],
                    "180",
                    "The passage explicitly states 180 patients.",
                ),
                (
                    "Passage: The contract requires delivery within thirty days of signing. What is the delivery window?",
                    vec!["ten days", "twenty days", "thirty days", "forty days"],
                    "thirty days",
                    "The passage explicitly states thirty days.",
                ),
                (
                    "Passage: The orchestra performed three encores after the scheduled program ended. How many encores were performed?",
                    vec!["one", "two", "three", "four"],
                    "three",
                    "The passage explicitly states three encores.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.4,
                attribution,
            )
        }
        "gre::verbal::reading::function" => {
            let scenarios = [
                (
                    "Passage: Many firms adopt remote work. However, collaboration costs can rise without in-person contact. What is the function of the second sentence?",
                    vec![
                        "It introduces a counterpoint to the first sentence.",
                        "It summarizes the entire passage.",
                        "It defines remote work.",
                        "It provides unrelated historical background.",
                    ],
                    "It introduces a counterpoint to the first sentence.",
                    "\"However\" signals a contrast with the benefit stated first.",
                ),
                (
                    "Passage: (1) Critics insisted the bridge would fail. (2) Yet after twenty years, not one support has failed. What is the function of sentence 2?",
                    vec![
                        "introduce the critics' main argument",
                        "provide evidence that undercuts the critics' claim",
                        "describe construction methods",
                        "propose building a new bridge",
                    ],
                    "provide evidence that undercuts the critics' claim",
                    "Real-world evidence rebuts the critics introduced in sentence one.",
                ),
                (
                    "Passage: (1) The drug showed promise in early trials. (2) Nevertheless, regulators demanded a larger study before approval. What is the function of sentence 2?",
                    vec![
                        "It introduces a limitation on the optimistic claim.",
                        "It summarizes the entire passage.",
                        "It defines the drug.",
                        "It provides unrelated history.",
                    ],
                    "It introduces a limitation on the optimistic claim.",
                    "\"Nevertheless\" signals a contrast with the early promise.",
                ),
                (
                    "Passage: (1) Solar costs have fallen sharply. (2) Still, storage remains expensive. What is the function of sentence 2?",
                    vec![
                        "It qualifies the optimistic claim in sentence 1.",
                        "It defines solar power.",
                        "It summarizes the passage.",
                        "It introduces an unrelated topic.",
                    ],
                    "It qualifies the optimistic claim in sentence 1.",
                    "\"Still\" signals a remaining obstacle despite progress.",
                ),
                (
                    "Passage: (1) The museum expanded its hours. (2) For example, it now opens on Mondays. What is the function of sentence 2?",
                    vec![
                        "It illustrates the claim made in sentence 1.",
                        "It contradicts sentence 1.",
                        "It introduces an unrelated topic.",
                        "It summarizes the entire passage.",
                    ],
                    "It illustrates the claim made in sentence 1.",
                    "\"For example\" introduces supporting detail for the expansion claim.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.55,
                attribution,
            )
        }
        "gre::verbal::vocabulary::context" => {
            let scenarios = [
                (
                    "From context clues in the sentence, although the instructions were ______, the team completed the assembly without errors.",
                    vec!["ambiguous", "lucid", "cryptic", "obscure"],
                    "lucid",
                    "Context clues show successful completion implies clear instructions.",
                ),
                (
                    "In the sentence \"Few crops survive in the region's arid climate,\" \"arid\" most nearly means:",
                    vec!["humid", "dry", "cold", "fertile"],
                    "dry",
                    "Crops struggling to survive indicates a dry climate.",
                ),
                (
                    "In the sentence \"The committee adopted a novel approach,\" \"novel\" most nearly means:",
                    vec!["fictional", "new", "lengthy", "difficult"],
                    "new",
                    "A novel approach is a new or original one.",
                ),
                (
                    "In the sentence \"The speaker's tone was unexpectedly genial,\" \"genial\" most nearly means:",
                    vec!["hostile", "friendly", "formal", "confused"],
                    "friendly",
                    "An unexpectedly pleasant manner indicates a friendly tone.",
                ),
                (
                    "In the sentence \"Skeptics remained unconvinced by the laudatory review,\" \"laudatory\" most nearly means:",
                    vec!["critical", "praising", "neutral", "technical"],
                    "praising",
                    "Skeptics doubting praise implies the review was laudatory.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.45,
                attribution,
            )
        }
        "gre::verbal::vocabulary::advanced" => {
            let scenarios = [
                (
                    "In advanced vocabulary, the CEO's ______ apology failed to reassure investors who wanted concrete reforms.",
                    vec!["abject", "perfunctory", "sincere", "heartfelt"],
                    "perfunctory",
                    "Investors wanted substance; a perfunctory apology shows superficial word choice.",
                ),
                (
                    "In academic vocabulary, \"ephemeral\" most nearly means:",
                    vec!["permanent", "fleeting", "abundant", "obscure"],
                    "fleeting",
                    "Ephemeral describes precise vocabulary for something that lasts a very short time.",
                ),
                (
                    "Advanced vocabulary: \"garrulous\" most nearly means:",
                    vec!["talkative", "silent", "angry", "generous"],
                    "talkative",
                    "Garrulous is advanced vocabulary describing someone who talks a great deal.",
                ),
                (
                    "Advanced vocabulary: \"obdurate\" most nearly means:",
                    vec!["stubborn", "flexible", "cheerful", "timid"],
                    "stubborn",
                    "Obdurate describes someone who refuses to change their mind.",
                ),
                (
                    "Advanced vocabulary: \"sanguine\" most nearly means:",
                    vec!["optimistic", "bleak", "angry", "confused"],
                    "optimistic",
                    "Sanguine describes a confidently optimistic outlook.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.55,
                attribution,
            )
        }
        "gre::awa::issue" => {
            let scenarios = [
                (
                    "Issue: \"Success is determined solely by financial wealth.\" What is the strongest critique?",
                    vec![
                        "Wealth is the only measurable outcome.",
                        "Success can include non-financial contributions and well-being.",
                        "Financial wealth is impossible to define.",
                        "Critiques of wealth are always invalid.",
                    ],
                    "Success can include non-financial contributions and well-being.",
                    "The claim overgeneralizes; success has multiple dimensions.",
                ),
                (
                    "Issue: \"Governments should focus on preventing environmental problems rather than developing fixes.\" What is the strongest critique?",
                    vec![
                        "Prevention and technological innovation can complement each other.",
                        "Environmental problems are impossible to prevent.",
                        "Technology always harms the environment.",
                        "Governments should ignore environmental issues.",
                    ],
                    "Prevention and technological innovation can complement each other.",
                    "The best critique notes a false dichotomy.",
                ),
                (
                    "Issue: \"Universities should eliminate grades and adopt pass/fail systems only.\" What is the strongest critique?",
                    vec![
                        "Pass/fail may reduce unhealthy competition but can blur meaningful distinctions.",
                        "Grades are always perfectly fair.",
                        "Students never respond to incentives.",
                        "Universities should not evaluate learning.",
                    ],
                    "Pass/fail may reduce unhealthy competition but can blur meaningful distinctions.",
                    "The best critique notes tradeoffs rather than an absolute choice.",
                ),
                (
                    "Issue: \"Technology in classrooms always improves learning outcomes.\" What is the strongest critique?",
                    vec![
                        "Effects depend on how technology is used, not technology alone.",
                        "Technology never belongs in classrooms.",
                        "Learning outcomes cannot be measured.",
                        "Classrooms should avoid all change.",
                    ],
                    "Effects depend on how technology is used, not technology alone.",
                    "The claim overstates a tool's impact without considering implementation.",
                ),
                (
                    "Issue: \"Governments should never regulate businesses because regulation stifles innovation.\" What is the strongest critique?",
                    vec![
                        "Both individual action and sensible regulation can be necessary at scale.",
                        "Innovation never requires rules.",
                        "All regulation is identical.",
                        "Businesses should never face external constraints.",
                    ],
                    "Both individual action and sensible regulation can be necessary at scale.",
                    "The claim sets up a false either-or between freedom and oversight.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.6,
                attribution,
            )
        }
        "gre::awa::argument" => {
            let scenarios = [
                (
                    "Argument: \"Our app downloads increased, so customer satisfaction must have improved.\" What is the main flaw?",
                    vec![
                        "Downloads may not reflect satisfaction.",
                        "Satisfaction always equals downloads.",
                        "Apps cannot be measured.",
                        "Customers never use downloaded apps.",
                    ],
                    "Downloads may not reflect satisfaction.",
                    "The argument equates usage metrics with satisfaction without evidence.",
                ),
                (
                    "Argument: \"Sales fell after we changed the logo, so the new logo caused the decline.\" What is the strongest objection?",
                    vec![
                        "Correlation does not establish causation; other factors could explain the decline.",
                        "The new logo is unattractive.",
                        "Store sales always fall every month.",
                        "Logos never affect sales in any way.",
                    ],
                    "Correlation does not establish causation; other factors could explain the decline.",
                    "The argument assumes the logo caused the drop simply because it came first.",
                ),
                (
                    "Argument: \"90% of website visitors are satisfied, so nearly all customers are satisfied.\" What is the strongest objection?",
                    vec![
                        "Website visitors may not be a representative sample of all customers.",
                        "Ninety percent is not a large number.",
                        "Surveys can never be trusted.",
                        "Customers are impossible to satisfy.",
                    ],
                    "Website visitors may not be a representative sample of all customers.",
                    "The argument generalizes from a possibly biased sample.",
                ),
                (
                    "Argument: \"Membership grew after we raised prices, so higher prices must attract members.\" What is the main flaw?",
                    vec![
                        "Other factors such as marketing or seasonality could explain the growth.",
                        "Higher prices always increase membership.",
                        "Membership cannot be measured.",
                        "Prices never affect demand.",
                    ],
                    "Other factors such as marketing or seasonality could explain the growth.",
                    "The argument assumes price caused the increase without ruling out alternatives.",
                ),
                (
                    "Argument: \"Employee training attendance rose, so skills must have improved.\" What is the strongest objection?",
                    vec![
                        "Attendance does not guarantee learning or skill transfer.",
                        "Training is impossible to measure.",
                        "Skills never improve after training.",
                        "Employees always attend voluntarily.",
                    ],
                    "Attendance does not guarantee learning or skill transfer.",
                    "The argument equates showing up with mastering the material.",
                ),
            ];
            let idx = (variant as usize) % scenarios.len();
            let (stem, choices, answer, explanation) = &scenarios[idx];
            mcq(
                topic_id,
                section,
                variant,
                now,
                stem,
                str_choices(choices),
                answer,
                explanation,
                0.6,
                attribution,
            )
        }
        _ => GeneratedQuestionDraft {
            id: new_generated_id(topic_id, variant, now),
            topic: topic_id.into(),
            section: section.into(),
            format: "mcq".into(),
            stem: format!(
                "{excerpt} Which statement \
                 best reflects this section?",
                excerpt = source.excerpt
            ),
            choices: vec![
                source
                    .keywords
                    .first()
                    .copied()
                    .unwrap_or("concept")
                    .to_string(),
                "unrelated detail".into(),
                "unsupported claim".into(),
                "contradictory idea".into(),
            ],
            correct_answer: source.keywords.first().copied().unwrap_or("concept").into(),
            explanation: "The correct choice aligns with the source excerpt keywords.".into(),
            difficulty: Some(0.5),
            confidence: 0.0,
            attribution,
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn mcq(
    topic_id: &str,
    section: &str,
    variant: u32,
    now: TimestampSecs,
    stem: &str,
    choices: Vec<String>,
    correct: &str,
    explanation: &str,
    difficulty: f32,
    attribution: QuestionAttribution,
) -> GeneratedQuestionDraft {
    validate_mcq_choices("generated variant", correct, &choices);
    debug_assert!(
        correct_answer_in_choices(correct, &choices),
        "correct answer {correct:?} missing from choices {choices:?}"
    );
    GeneratedQuestionDraft {
        id: new_generated_id(topic_id, variant, now),
        topic: topic_id.into(),
        section: section.into(),
        format: "mcq".into(),
        stem: stem.into(),
        choices,
        correct_answer: correct.into(),
        explanation: explanation.into(),
        difficulty: Some(difficulty),
        confidence: 0.0,
        attribution,
    }
}

/// Whether the trimmed correct answer appears among the presented choices.
pub(crate) fn correct_answer_in_choices(correct: &str, choices: &[String]) -> bool {
    choices
        .iter()
        .any(|choice| normalize_choice(choice) == normalize_choice(correct))
}

/// Case-insensitive, whitespace-collapsed choice key for uniqueness checks.
pub(crate) fn normalize_choice(choice: &str) -> String {
    choice.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Fail loudly when MCQ choices duplicate or the correct answer is ambiguous.
pub(crate) fn validate_mcq_choices(question_id: &str, correct: &str, choices: &[String]) {
    assert!(
        !choices.is_empty(),
        "{question_id}: MCQ has no choices"
    );
    let mut seen = HashSet::new();
    for choice in choices {
        let key = normalize_choice(choice);
        assert!(
            seen.insert(key),
            "{question_id}: duplicate choice {choice:?} among {choices:?}"
        );
    }
    let matches = choices
        .iter()
        .filter(|choice| normalize_choice(choice) == normalize_choice(correct))
        .count();
    assert_eq!(
        matches, 1,
        "{question_id}: expected exactly one correct choice for {correct:?}, got {matches} in {choices:?}"
    );
}

fn str_choices(choices: &[&str]) -> Vec<String> {
    choices.iter().map(|choice| (*choice).to_string()).collect()
}

fn four_choices(correct: &str, distractors: &[String]) -> Vec<String> {
    let mut out = Vec::with_capacity(4);
    let mut seen = HashSet::new();
    let correct_key = normalize_choice(correct);
    for distractor in distractors {
        let key = normalize_choice(distractor);
        if key == correct_key || !seen.insert(key) {
            continue;
        }
        out.push(distractor.clone());
        if out.len() == 3 {
            break;
        }
    }
    if out.len() < 3 || !correct_answer_in_choices(correct, &out) {
        out.push(correct.to_string());
    }
    validate_mcq_choices("template four_choices", correct, &out);
    out
}

fn four_int_choices(correct: i32, offsets: [i32; 3]) -> Vec<String> {
    let distractors: Vec<String> = offsets
        .iter()
        .map(|offset| (correct + offset).to_string())
        .collect();
    four_choices(&correct.to_string(), &distractors)
}

fn to_string_vec(choices: &[&str]) -> Vec<String> {
    choices.iter().map(|s| (*s).to_string()).collect()
}

fn new_generated_id(topic_id: &str, variant: u32, now: TimestampSecs) -> String {
    let slug = topic_id
        .strip_prefix("gre::")
        .unwrap_or(topic_id)
        .replace("::", "-");
    format!("ai-{slug}-v{variant}-{now}", now = now.0)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::gre_atlas::domain::GreCatalog;
    use crate::gre_atlas::questions::bank::MIN_PRACTICE_BANK_PER_TOPIC;
    use crate::gre_atlas::questions::source::source_section_for_topic;

    #[test]
    fn first_five_variants_have_unique_stems_per_topic() {
        use std::collections::HashSet;

        let now = TimestampSecs(1_700_000_000);
        for leaf in GreCatalog::leaf_topics() {
            let Some(source) = source_section_for_topic(leaf.id) else {
                continue;
            };
            let mut stems = HashSet::new();
            for variant in 0..MIN_PRACTICE_BANK_PER_TOPIC {
                let draft = build_variant_draft(leaf.id, leaf.section, source, variant, now);
                assert!(
                    stems.insert(draft.stem.clone()),
                    "{} variant {variant} repeats stem {:?}",
                    leaf.id,
                    draft.stem
                );
            }
        }
    }

    #[test]
    fn all_variant_drafts_include_correct_choice() {
        let now = TimestampSecs(1_700_000_000);
        for leaf in GreCatalog::leaf_topics() {
            let Some(source) = source_section_for_topic(leaf.id) else {
                continue;
            };
            for variant in 0..8 {
                let draft = build_variant_draft(leaf.id, leaf.section, source, variant, now);
                assert!(
                    correct_answer_in_choices(&draft.correct_answer, &draft.choices),
                    "{} variant {variant}: correct {:?} not in choices {:?}",
                    leaf.id,
                    draft.correct_answer,
                    draft.choices
                );
                assert!(
                    !draft.choices.is_empty(),
                    "{} variant {variant} has no choices",
                    leaf.id
                );
                validate_mcq_choices(&draft.id, &draft.correct_answer, &draft.choices);
            }
        }
    }

    #[test]
    fn validate_mcq_choices_rejects_duplicates() {
        let choices = vec![
            "12π".into(),
            "12π".into(),
            "6π".into(),
            "36π".into(),
        ];
        let result = std::panic::catch_unwind(|| {
            validate_mcq_choices("test-q", "12π", &choices);
        });
        assert!(result.is_err());
    }
}
