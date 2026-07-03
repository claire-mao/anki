# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
from pathlib import Path


def _benchmark_module():
    repo_root = Path(__file__).resolve().parents[2]
    module_path = repo_root / "scripts" / "eval" / "gre_atlas_benchmark.py"
    spec = importlib.util.spec_from_file_location("gre_atlas_benchmark", module_path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def test_percentile_summary_is_deterministic() -> None:
    bench = _benchmark_module()
    samples = [1.0, 2.0, 3.0, 4.0, 100.0]
    summary = bench.summarize_timings(samples, iterations=5, warmup=1)
    assert summary.p50_ms == 3.0
    assert summary.worst_ms == 100.0
    assert summary.p95_ms > summary.p50_ms


def test_benchmark_harness_writes_markdown_report(tmp_path: Path) -> None:
    repo_root = Path(__file__).resolve().parents[2]
    script = repo_root / "scripts" / "eval" / "gre_atlas_benchmark.py"
    output_dir = tmp_path / "out"
    proc = subprocess.run(
        [
            sys.executable,
            str(script),
            "--synthetic-cards",
            "100",
            "--iterations",
            "3",
            "--warmup",
            "1",
            "--output-dir",
            str(output_dir),
        ],
        cwd=repo_root,
        env={**os.environ, "PYTHONPATH": str(repo_root / "out" / "pylib")},
        check=True,
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0

    md_path = output_dir / "gre-atlas-benchmark.md"
    json_path = output_dir / "gre-atlas-benchmark.json"
    csv_path = output_dir / "gre-atlas-benchmark.csv"
    markdown = md_path.read_text(encoding="utf-8")
    payload = json.loads(json_path.read_text(encoding="utf-8"))
    csv_text = csv_path.read_text(encoding="utf-8")

    assert "GRE Atlas benchmark report" in markdown
    assert "TopicMastery" in markdown
    assert "p50 (ms)" in markdown
    assert "p95 (ms)" in markdown
    assert "Worst (ms)" in markdown
    assert payload["benchmarks"]
    assert payload["collections"][0]["data_source"] == "synthetic_reference"
    assert "p50_ms" in csv_text
    assert "topic_mastery" in csv_text
