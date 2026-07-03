#!/usr/bin/env python3
# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Benchmark GRE Atlas production entry points (read-only timing harness)."""

from __future__ import annotations

import argparse
import json
import statistics
import sys
import tempfile
import time
from collections.abc import Callable
from dataclasses import asdict, dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from anki.gre_atlas import (
    GRE_DECK_NAME,
    LEGACY_GRE_DECK_NAME,
    TOPIC_TAG_PREFIX,
    get_dashboard,
    get_scores,
    get_study_plan,
    list_questions,
    resolve_gre_deck_id,
)
from anki.collection import Collection

DEFAULT_ITERATIONS = 30
DEFAULT_WARMUP = 3
DEFAULT_SYNTHETIC_CARDS = 10_000
DEFAULT_OUTPUT_DIR = "docs/gre-atlas-submission/results"

BENCHMARK_TOPIC_MASTERY = "topic_mastery"
BENCHMARK_DASHBOARD = "dashboard_generation"
BENCHMARK_READINESS = "readiness_calculation"
BENCHMARK_STUDY_PLAN = "study_plan_generation"

BENCHMARK_ORDER = (
    BENCHMARK_TOPIC_MASTERY,
    BENCHMARK_DASHBOARD,
    BENCHMARK_READINESS,
    BENCHMARK_STUDY_PLAN,
)


@dataclass
class TimingSummary:
    iterations: int
    warmup: int
    samples: int
    p50_ms: float
    p95_ms: float
    worst_ms: float
    mean_ms: float
    min_ms: float


@dataclass
class BenchmarkResult:
    id: str
    label: str
    timing: TimingSummary


@dataclass
class CollectionProfile:
    path: str
    data_source: str
    label: str
    gre_deck_exists: bool
    total_cards: int
    gre_deck_cards: int
    revlog_entries: int
    fsrs_enabled: bool
    synthetic_card_target: int | None = None


@dataclass
class BenchmarkReport:
    generated_at_utc: str
    harness: str
    iterations: int
    warmup: int
    collections: list[CollectionProfile] = field(default_factory=list)
    benchmarks: dict[str, list[BenchmarkResult]] = field(default_factory=dict)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Benchmark GRE Atlas production APIs (TopicMastery, dashboard, "
            "readiness, study plan). Does not modify production code paths."
        )
    )
    parser.add_argument(
        "--collection",
        action="append",
        default=[],
        help="Path to collection.anki2 (repeat for multiple large collections).",
    )
    parser.add_argument(
        "--synthetic-cards",
        type=int,
        default=0,
        help=(
            "Generate a labeled synthetic GRE-tagged collection with this many "
            f"cards (default when no --collection: {DEFAULT_SYNTHETIC_CARDS})."
        ),
    )
    parser.add_argument(
        "--iterations",
        type=int,
        default=DEFAULT_ITERATIONS,
        help=f"Timed iterations per benchmark (default: {DEFAULT_ITERATIONS}).",
    )
    parser.add_argument(
        "--warmup",
        type=int,
        default=DEFAULT_WARMUP,
        help=f"Warmup iterations excluded from stats (default: {DEFAULT_WARMUP}).",
    )
    parser.add_argument(
        "--output-dir",
        default=DEFAULT_OUTPUT_DIR,
        help="Directory for gre-atlas-benchmark.md, .json, and .csv.",
    )
    return parser.parse_args()


def percentile(values: list[float], p: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    if len(ordered) == 1:
        return ordered[0]
    rank = (len(ordered) - 1) * (p / 100.0)
    lower = int(rank)
    upper = min(lower + 1, len(ordered) - 1)
    weight = rank - lower
    return ordered[lower] + (ordered[upper] - ordered[lower]) * weight


def summarize_timings(
    samples_ms: list[float], *, iterations: int, warmup: int
) -> TimingSummary:
    return TimingSummary(
        iterations=iterations,
        warmup=warmup,
        samples=len(samples_ms),
        p50_ms=percentile(samples_ms, 50),
        p95_ms=percentile(samples_ms, 95),
        worst_ms=max(samples_ms),
        mean_ms=statistics.mean(samples_ms),
        min_ms=min(samples_ms),
    )


def time_call(fn: Callable[[], Any]) -> float:
    start = time.perf_counter()
    fn()
    return (time.perf_counter() - start) * 1000.0


def benchmark_operation(
    operation_id: str,
    label: str,
    fn: Callable[[], Any],
    *,
    iterations: int,
    warmup: int,
) -> BenchmarkResult:
    for _ in range(warmup):
        fn()
    samples = [time_call(fn) for _ in range(iterations)]
    return BenchmarkResult(
        id=operation_id,
        label=label,
        timing=summarize_timings(samples, iterations=iterations, warmup=warmup),
    )


def synthetic_leaf_tags(col: Collection) -> list[str]:
    questions = list_questions(col, limit=500)
    tags = sorted({question.topic for question in questions.questions})
    if tags:
        return tags
    return ["gre::quant::algebra::linear", "gre::verbal::text_completion"]


def build_synthetic_collection(
    collection_path: Path,
    *,
    card_count: int,
    review_every: int = 5,
) -> None:
    """Build a labeled synthetic large collection for benchmark runs."""
    collection_path.parent.mkdir(parents=True, exist_ok=True)
    col = Collection(str(collection_path))
    try:
        col.set_config("fsrs", True)
        gre_deck = resolve_gre_deck_id(col)
        if gre_deck is None:
            gre_deck = col.decks.id(GRE_DECK_NAME)
        col.decks.select(gre_deck)
        tags = synthetic_leaf_tags(col)

        for index in range(card_count):
            note = col.newNote()
            note["Front"] = f"benchmark card {index}"
            note["Back"] = f"synthetic benchmark payload {index}"
            note.tags = [tags[index % len(tags)]]
            col.addNote(note)
            card = note.cards()[0]
            col.set_deck([card.id], gre_deck)

            if review_every > 0 and index % review_every == 0:
                card = col.get_card(card.id)
                card.start_timer()
                col.sched.answerCard(card, 3)
    finally:
        col.close()


def profile_collection(
    collection_path: Path,
    *,
    data_source: str,
    label: str,
    synthetic_card_target: int | None = None,
) -> CollectionProfile:
    col = Collection(str(collection_path))
    try:
        gre_deck_exists = resolve_gre_deck_id(col) is not None
        total_cards = col.db.scalar("select count() from cards") or 0
        gre_deck_cards = (
            col.db.scalar(
                """
                select count()
                from cards
                join decks on cards.did = decks.id
                where instr(decks.name, ?) > 0 or instr(decks.name, ?) > 0
                """,
                GRE_DECK_NAME,
                LEGACY_GRE_DECK_NAME,
            )
            or 0
        )
        revlog_entries = col.db.scalar("select count() from revlog") or 0
        fsrs_enabled = bool(col.get_config("fsrs"))
    finally:
        col.close()

    return CollectionProfile(
        path=str(collection_path),
        data_source=data_source,
        label=label,
        gre_deck_exists=gre_deck_exists,
        total_cards=int(total_cards),
        gre_deck_cards=int(gre_deck_cards),
        revlog_entries=int(revlog_entries),
        fsrs_enabled=fsrs_enabled,
        synthetic_card_target=synthetic_card_target,
    )


def run_collection_benchmarks(
    collection_path: Path,
    *,
    iterations: int,
    warmup: int,
    data_source: str,
    label: str,
    synthetic_card_target: int | None = None,
) -> tuple[CollectionProfile, list[BenchmarkResult]]:
    profile = profile_collection(
        collection_path,
        data_source=data_source,
        label=label,
        synthetic_card_target=synthetic_card_target,
    )

    col = Collection(str(collection_path))
    try:
        mastery_search = f'deck:"{GRE_DECK_NAME}" OR deck:"{LEGACY_GRE_DECK_NAME}"'

        operations: list[tuple[str, str, Callable[[], Any]]] = [
            (
                BENCHMARK_TOPIC_MASTERY,
                "TopicMastery",
                lambda: col.topic_mastery(
                    search=mastery_search,
                    topic_tag_prefix=TOPIC_TAG_PREFIX,
                    min_reviews=1,
                ),
            ),
            (
                BENCHMARK_DASHBOARD,
                "Dashboard generation",
                lambda: get_dashboard(
                    col, recent_activity_limit=10, topic_insight_limit=5
                ),
            ),
            (
                BENCHMARK_READINESS,
                "Readiness calculation",
                lambda: get_scores(col),
            ),
            (
                BENCHMARK_STUDY_PLAN,
                "Study plan generation",
                lambda: get_study_plan(col, limit=10),
            ),
        ]

        results = [
            benchmark_operation(
                operation_id,
                label,
                fn,
                iterations=iterations,
                warmup=warmup,
            )
            for operation_id, label, fn in operations
        ]
    finally:
        col.close()

    return profile, results


def resolve_collection_targets(
    args: argparse.Namespace,
) -> list[tuple[Path, str, str, int | None]]:
    targets: list[tuple[Path, str, str, int | None]] = []

    for collection in args.collection:
        path = Path(collection).expanduser()
        targets.append((path, "collection", path.name, None))

    synthetic_cards = args.synthetic_cards
    if not targets and synthetic_cards == 0:
        synthetic_cards = DEFAULT_SYNTHETIC_CARDS

    if synthetic_cards > 0:
        temp_dir = Path(tempfile.mkdtemp(prefix="gre-atlas-bench-"))
        collection_path = temp_dir / "collection.anki2"
        print(
            f"Generating labeled synthetic collection with {synthetic_cards} cards "
            f"at {collection_path} ..."
        )
        build_synthetic_collection(collection_path, card_count=synthetic_cards)
        targets.append(
            (
                collection_path,
                "synthetic_reference",
                f"synthetic ({synthetic_cards} cards)",
                synthetic_cards,
            )
        )

    if not targets:
        raise SystemExit(
            "Provide --collection and/or --synthetic-cards to benchmark large collections."
        )

    return targets


def render_csv(report: BenchmarkReport) -> str:
    lines = [
        "collection_label,data_source,benchmark_id,benchmark_label,iterations,warmup,samples,p50_ms,p95_ms,worst_ms,mean_ms,min_ms",
    ]
    for profile in report.collections:
        results = report.benchmarks.get(profile.path, [])
        results_by_id = {result.id: result for result in results}
        for benchmark_id in BENCHMARK_ORDER:
            result = results_by_id[benchmark_id]
            timing = result.timing
            lines.append(
                ",".join(
                    [
                        _csv_cell(profile.label),
                        _csv_cell(profile.data_source),
                        _csv_cell(benchmark_id),
                        _csv_cell(result.label),
                        str(timing.iterations),
                        str(timing.warmup),
                        str(timing.samples),
                        f"{timing.p50_ms:.4f}",
                        f"{timing.p95_ms:.4f}",
                        f"{timing.worst_ms:.4f}",
                        f"{timing.mean_ms:.4f}",
                        f"{timing.min_ms:.4f}",
                    ]
                )
            )
    return "\n".join(lines) + "\n"


def _csv_cell(value: str) -> str:
    if any(char in value for char in [",", '"', "\n"]):
        return '"' + value.replace('"', '""') + '"'
    return value


def render_markdown(report: BenchmarkReport) -> str:
    lines = [
        "# GRE Atlas benchmark report",
        "",
        f"- Generated at (UTC): {report.generated_at_utc}",
        f"- Harness: `{report.harness}`",
        f"- Iterations per benchmark: {report.iterations}",
        f"- Warmup iterations (excluded): {report.warmup}",
        "",
        "Production entry points timed via pylib (no benchmark hooks in Rust):",
        "",
        "| Benchmark | Production API |",
        "| --- | --- |",
        f'| TopicMastery | `Collection.topic_mastery(search=deck:"{GRE_DECK_NAME}" OR deck:"{LEGACY_GRE_DECK_NAME}")` |',
        "| Dashboard generation | `gre_atlas.get_dashboard()` |",
        "| Readiness calculation | `gre_atlas.get_scores()` |",
        "| Study plan generation | `gre_atlas.get_study_plan()` |",
        "",
    ]

    for profile in report.collections:
        lines.extend(
            [
                f"## {profile.label}",
                "",
                f"- Data source: `{profile.data_source}`",
                f"- Collection path: `{profile.path}`",
                f"- GRE deck exists: {profile.gre_deck_exists}",
                f"- Total cards: {profile.total_cards:,}",
                f"- GRE deck cards: {profile.gre_deck_cards:,}",
                f"- Revlog entries: {profile.revlog_entries:,}",
                f"- FSRS enabled: {profile.fsrs_enabled}",
            ]
        )
        if profile.synthetic_card_target is not None:
            lines.append(
                f"- Synthetic card target: {profile.synthetic_card_target:,} "
                "(clearly labeled synthetic_reference data)"
            )
        lines.extend(
            [
                "",
                "| Benchmark | p50 (ms) | p95 (ms) | Worst (ms) | Mean (ms) |",
                "| --- | ---: | ---: | ---: | ---: |",
            ]
        )

        results = report.benchmarks.get(profile.path, [])
        results_by_id = {result.id: result for result in results}
        for benchmark_id in BENCHMARK_ORDER:
            result = results_by_id[benchmark_id]
            timing = result.timing
            lines.append(
                f"| {result.label} | {timing.p50_ms:.2f} | {timing.p95_ms:.2f} | "
                f"{timing.worst_ms:.2f} | {timing.mean_ms:.2f} |"
            )
        lines.append("")

    lines.extend(
        [
            "## Reproducibility",
            "",
            "Re-run with:",
            "",
            "```bash",
            "just bench-gre-atlas --collection /path/to/large/collection.anki2",
            "# or labeled synthetic large collection:",
            "just bench-gre-atlas --synthetic-cards 50000",
            "```",
            "",
            "Warmup iterations are excluded from p50/p95/worst statistics.",
            "",
        ]
    )
    return "\n".join(lines)


def main() -> int:
    args = parse_args()
    if args.iterations <= 0:
        raise SystemExit("--iterations must be positive")
    if args.warmup < 0:
        raise SystemExit("--warmup must be >= 0")

    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    report = BenchmarkReport(
        generated_at_utc=datetime.now(tz=UTC).isoformat(),
        harness="scripts/eval/gre_atlas_benchmark.py",
        iterations=args.iterations,
        warmup=args.warmup,
    )

    for path, data_source, label, synthetic_target in resolve_collection_targets(args):
        print(f"Benchmarking {label} ({path}) ...")
        profile, results = run_collection_benchmarks(
            path,
            iterations=args.iterations,
            warmup=args.warmup,
            data_source=data_source,
            label=label,
            synthetic_card_target=synthetic_target,
        )
        report.collections.append(profile)
        report.benchmarks[profile.path] = results

    markdown = render_markdown(report)
    csv_report = render_csv(report)
    json_path = output_dir / "gre-atlas-benchmark.json"
    md_path = output_dir / "gre-atlas-benchmark.md"
    csv_path = output_dir / "gre-atlas-benchmark.csv"
    json_path.write_text(
        json.dumps(asdict(report), indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    md_path.write_text(markdown, encoding="utf-8")
    csv_path.write_text(csv_report, encoding="utf-8")

    print(f"Wrote {json_path}")
    print(f"Wrote {md_path}")
    print(f"Wrote {csv_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
