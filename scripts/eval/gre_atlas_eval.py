#!/usr/bin/env python3
# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Regenerate GRE Atlas evaluation metrics from a collection snapshot."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from anki.gre_atlas import generate_gre_atlas_eval_report
from anki.collection import Collection

DEFAULT_OUTPUT_DIR = Path("docs/gre-atlas-submission/results")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Regenerate GRE Atlas evaluation metrics (read-only)."
    )
    parser.add_argument(
        "--collection",
        required=True,
        help="Path to collection.anki2 (or profile folder containing it).",
    )
    parser.add_argument(
        "--output-dir",
        default=str(DEFAULT_OUTPUT_DIR),
        help="Directory for eval artifacts (JSON, Markdown).",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    collection_path = Path(args.collection).expanduser()
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    col = Collection(str(collection_path))
    try:
        response = generate_gre_atlas_eval_report(col)
    finally:
        col.close()

    json_path = output_dir / "gre-atlas-eval.json"
    md_path = output_dir / "gre-atlas-eval.md"
    performance_md_path = output_dir / "performance-eval.md"
    json_path.write_text(response.json, encoding="utf-8")
    md_path.write_text(response.markdown, encoding="utf-8")
    performance_md_path.write_text(response.performance_markdown, encoding="utf-8")

    # Validate JSON before declaring success.
    payload = json.loads(response.json)
    assert "prediction_distribution" in payload

    print(f"Wrote {json_path}")
    print(f"Wrote {md_path}")
    print(f"Wrote {performance_md_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
