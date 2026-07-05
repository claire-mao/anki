#!/usr/bin/env python3
# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Regenerate GRE Atlas AI evaluation metrics and enforce the release gate."""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

from anki.gre_atlas import generate_gre_atlas_ai_eval_report
from anki.collection import Collection


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Regenerate GRE Atlas AI evaluation metrics (read-only)."
    )
    parser.add_argument(
        "--collection",
        default=":memory:",
        help="Path to collection.anki2 (default :memory:; eval does not read collection data).",
    )
    parser.add_argument(
        "--output-dir",
        default="docs/gre-atlas-submission/results",
        help="Directory for gre-atlas-ai-eval.json and gre-atlas-ai-eval.md.",
    )
    parser.add_argument(
        "--min-accuracy",
        type=float,
        default=None,
        help="Minimum held-out accuracy (default 0.95 or GRE_ATLAS_AI_EVAL_MIN_ACCURACY).",
    )
    parser.add_argument(
        "--max-wrong-answer-rate",
        type=float,
        default=None,
        help="Maximum wrong-answer rate (default 0.0 or GRE_ATLAS_AI_EVAL_MAX_WRONG_ANSWER_RATE).",
    )
    parser.add_argument(
        "--acceptance-cutoff",
        type=float,
        default=None,
        help="Minimum generation confidence (default 0.55 or GRE_ATLAS_AI_EVAL_ACCEPTANCE_CUTOFF).",
    )
    parser.add_argument(
        "--allow-fail",
        action="store_true",
        help="Write the report even when the release gate fails (exit code still non-zero).",
    )
    return parser.parse_args()


def apply_criteria_env(args: argparse.Namespace) -> None:
    if args.min_accuracy is not None:
        os.environ["GRE_ATLAS_AI_EVAL_MIN_ACCURACY"] = str(args.min_accuracy)
    if args.max_wrong_answer_rate is not None:
        os.environ["GRE_ATLAS_AI_EVAL_MAX_WRONG_ANSWER_RATE"] = str(
            args.max_wrong_answer_rate
        )
    if args.acceptance_cutoff is not None:
        os.environ["GRE_ATLAS_AI_EVAL_ACCEPTANCE_CUTOFF"] = str(args.acceptance_cutoff)


def print_report_summary(report: dict) -> None:
    held_out = report["held_out_quality"]
    verdict = report["verdict"]
    criteria = report["acceptance_criteria"]

    print("GRE Atlas AI evaluation report")
    print(f"  model_version: {report['model_version']}")
    print(f"  gold_set: {report['gold_set_label']} ({held_out['evaluated_count']} questions)")
    print(f"  accuracy: {held_out['accuracy'] * 100:.1f}%")
    print(f"  wrong_answer_rate: {held_out['wrong_answer_rate'] * 100:.1f}%")
    print(
        "  acceptance_cutoff: "
        f"{criteria['acceptance_cutoff']:.2f} "
        f"(min_accuracy={criteria['min_accuracy'] * 100:.1f}%, "
        f"max_wrong_answer_rate={criteria['max_wrong_answer_rate'] * 100:.1f}%)"
    )
    print(f"  release_verdict: {'PASS' if verdict['passed'] else 'FAIL'}")
    if verdict["failure_reasons"]:
        print("  failure_reasons:")
        for reason in verdict["failure_reasons"]:
            print(f"    - {reason}")
    failing_topics = held_out.get("failing_topics") or []
    if failing_topics:
        print("  failing_topics:")
        for topic in failing_topics:
            print(f"    - {topic}")


def main() -> int:
    args = parse_args()
    apply_criteria_env(args)

    collection_path = Path(args.collection).expanduser()
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    col = Collection(str(collection_path))
    try:
        response = generate_gre_atlas_ai_eval_report(col)
    finally:
        col.close()

    report = json.loads(response.json)

    json_path = output_dir / "gre-atlas-ai-eval.json"
    md_path = output_dir / "gre-atlas-ai-eval.md"
    json_path.write_text(response.json, encoding="utf-8")
    md_path.write_text(response.markdown, encoding="utf-8")

    print_report_summary(report)
    print(f"Wrote {json_path}")
    print(f"Wrote {md_path}")

    if report["verdict"]["passed"]:
        return 0

    if not args.allow_fail:
        print("Release gate FAILED — model rejected.", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
