#!/usr/bin/env python3
# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
"""Generate GRE Atlas foundation practice bank JSON files."""

# mypy: ignore-errors
# Dev-only deterministic generator: loop variables intentionally rebind across
# the quant/verbal/awa sections with heterogeneous inline scenario tuples. The
# committed seed_gre_*.json files are the shipped artifact; this script is not
# part of the runtime, so strict typing is not enforced here.

from __future__ import annotations

import json
import random
import re
from pathlib import Path
from typing import Any

random.seed(42)

SOURCE = "GRE Atlas Practice Bank"
OUT_DIR = Path(__file__).resolve().parents[1] / "rslib/src/gre_atlas/questions"

MIN_FOUNDATION_QUANT = 100
MIN_FOUNDATION_VERBAL = 100
MIN_FOUNDATION_AWA = 50
MIN_FOUNDATION_PER_TOPIC = 8


# 30% easy, 50% medium, 20% hard (325 total)
def build_difficulty_schedule(total: int) -> list[float]:
    easy_n = round(total * 0.30)
    hard_n = round(total * 0.20)
    medium_n = total - easy_n - hard_n
    easy = [0.25, 0.28, 0.30, 0.32, 0.35]
    medium = [0.40, 0.45, 0.50, 0.55, 0.60]
    hard = [0.65, 0.70, 0.75, 0.80, 0.85]
    out: list[float] = []
    for i in range(easy_n):
        out.append(easy[i % len(easy)])
    for i in range(medium_n):
        out.append(medium[i % len(medium)])
    for i in range(hard_n):
        out.append(hard[i % len(hard)])
    random.shuffle(out)
    return out


DIFF_SCHEDULE = build_difficulty_schedule(325)


def q(  # noqa: PLR0913
    id_: str,
    topic: str,
    section: str,
    prompt: str,
    choices: list[str],
    correct: str,
    explanation: str,
    *,
    fmt: str = "mcq",
    subtopic: str = "",
    question_type: str = "",
    concepts: list[str] | None = None,
    difficulty: float = 0.5,
    time_secs: int = 90,
) -> dict[str, Any]:
    return {
        "id": id_,
        "topic": topic,
        "section": section,
        "format": fmt,
        "prompt": prompt,
        "answer_choices": choices,
        "correct_answer": correct,
        "explanation": explanation,
        "difficulty": difficulty,
        "source": SOURCE,
        "subtopic": subtopic,
        "question_type": question_type or fmt,
        "concepts_tested": concepts or [],
        "estimated_time_seconds": time_secs,
    }


def pick_diff(i: int) -> float:
    return DIFF_SCHEDULE[i % len(DIFF_SCHEDULE)]


def norm_choice(text: str) -> str:
    return re.sub(r"\s+", " ", text.strip().lower())


def pick_distractors(correct: str, candidates: list[str], count: int = 3) -> list[str]:
    """Return `count` unique distractors, none matching `correct` (normalized)."""
    seen = {norm_choice(correct)}
    out: list[str] = []
    for candidate in candidates:
        key = norm_choice(candidate)
        if key in seen:
            continue
        seen.add(key)
        out.append(candidate)
        if len(out) == count:
            break
    if len(out) != count:
        raise ValueError(
            f"need {count} unique distractors for {correct!r}, got {out!r} from {candidates!r}"
        )
    return out


def four_choices(correct: str, wrong: list[str]) -> list[str]:
    distractors = pick_distractors(correct, wrong)
    choices = [correct, *distractors]
    random.shuffle(choices)
    return choices


def circle_circumference_distractors(r: int) -> list[str]:
    return pick_distractors(
        f"{2 * r}π",
        [
            f"{r}π",
            f"{r**2}π",
            f"{2 * r + 2}π",
            f"{2 * r - 2}π",
            f"{(r + 1) ** 2}π",
            f"{4 * r}π",
        ],
    )


def circle_area_distractors(r: int) -> list[str]:
    return pick_distractors(
        f"{r**2}π",
        [
            f"{2 * r}π",
            f"{r}π",
            f"{(r + 1) ** 2}π",
            f"{(r - 1) ** 2}π",
            f"{r**2 + r}π",
            f"{4 * r}π",
        ],
    )


def triangle_hypotenuse_distractors(a: int, b: int, c: int) -> list[str]:
    return pick_distractors(
        str(c),
        [str(c + 1), str(c + 2), str(c - 1), str(c - 2), str(a + b), str(abs(a - b))],
    )


def data_analysis_distractors(stat: str, ans: str) -> list[str]:
    if stat == "mean":
        value = float(ans)
        return pick_distractors(
            ans,
            [
                str(round(value + 1, 1)),
                str(round(value - 1, 1)),
                str(round(value + 2, 1)),
                str(round(value - 2, 1)),
                str(round(value + 0.5, 1)),
            ],
        )
    numeric = int(float(ans))
    return pick_distractors(
        ans,
        [
            str(numeric + 2),
            str(numeric - 2),
            str(numeric + 5),
            str(numeric - 5),
            str(numeric + 1),
        ],
    )


def gen_quant() -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    idx = 0

    # --- Percent (14) ---
    pct_scenarios = [
        (120, 20, 96, "20% of $120 is $24; $120 − $24 = $96."),
        (200, 15, 170, "15% of $200 is $30; $200 − $30 = $170."),
        (80, 25, 60, "25% of $80 is $20; $80 − $20 = $60."),
        (150, 10, 135, "10% of $150 is $15; $150 − $15 = $135."),
        (60, 50, 30, "50% of $60 is $30."),
        (400, 5, 380, "5% of $400 is $20; $400 − $20 = $380."),
        (90, 30, 63, "30% of $90 is $27; $90 − $27 = $63."),
        (250, 12, 220, "12% of $250 is $30; $250 − $30 = $220."),
        (45, 40, 27, "40% of $45 is $18; $45 − $18 = $27."),
        (300, 8, 276, "8% of $300 is $24; $300 − $24 = $276."),
        (75, 16, 63, "16% of $75 is $12; $75 − $12 = $63."),
        (500, 6, 470, "6% of $500 is $30; $500 − $30 = $470."),
        (180, 35, 117, "35% of $180 is $63; $180 − $63 = $117."),
        (240, 45, 132, "45% of $240 is $108; $240 − $108 = $132."),
    ]
    for i, (price, pct, ans, expl) in enumerate(pct_scenarios):
        wrong = [price - pct, price + pct, price // 2, price + 10]
        out.append(
            q(
                f"gre-foundation-quant-pct-{i + 1:03d}",
                "gre::quant::arithmetic::percent",
                "quant",
                f"A jacket priced at ${price} is discounted by {pct}%. What is the sale price?",
                four_choices(f"${ans}", [f"${w}" for w in wrong if w != ans][:3]),
                f"${ans}",
                expl,
                subtopic="percent_discount",
                concepts=["percent", "discount"],
                difficulty=pick_diff(idx),
                time_secs=75,
            )
        )
        idx += 1

    # Percent increase (extra to reach 14 if needed - we have 14 above)

    # --- Ratio (13) ---
    ratio_scenarios = [
        (2, 3, 10, 15, "10 is 2 parts, so one part is 5. Three parts gives 15."),
        (3, 4, 21, 28, "21 is 3 parts, so one part is 7. Four parts gives 28."),
        (5, 2, 35, 14, "35 is 5 parts, so one part is 7. Two parts gives 14."),
        (4, 7, 16, 28, "16 is 4 parts, so one part is 4. Seven parts gives 28."),
        (3, 5, 18, 30, "18 is 3 parts, so one part is 6. Five parts gives 30."),
        (2, 5, 14, 35, "14 is 2 parts, so one part is 7. Five parts gives 35."),
        (7, 3, 28, 12, "28 is 7 parts, so one part is 4. Three parts gives 12."),
        (5, 8, 25, 40, "25 is 5 parts, so one part is 5. Eight parts gives 40."),
        (6, 11, 30, 55, "30 is 6 parts, so one part is 5. Eleven parts gives 55."),
        (9, 4, 27, 12, "27 is 9 parts, so one part is 3. Four parts gives 12."),
        (1, 4, 7, 28, "7 is 1 part, so four parts gives 28."),
        (8, 5, 24, 15, "24 is 8 parts, so one part is 3. Five parts gives 15."),
        (3, 2, 45, 30, "45 is 3 parts, so one part is 15. Two parts gives 30."),
    ]
    for i, (a, b, given, ans, expl) in enumerate(ratio_scenarios):
        out.append(
            q(
                f"gre-foundation-quant-rat-{i + 1:03d}",
                "gre::quant::arithmetic::ratio",
                "quant",
                f"If the ratio of A to B is {a}:{b} and A equals {given}, what is B?",
                four_choices(str(ans), [str(ans + 5), str(ans - 3), str(ans + 8)]),
                str(ans),
                expl,
                subtopic="part_whole_ratio",
                concepts=["ratio", "proportion"],
                difficulty=pick_diff(idx),
                time_secs=80,
            )
        )
        idx += 1

    # --- Linear algebra (14) ---
    linear_scenarios = [
        ("2x + 5 = 17", "6", "Subtract 5, divide by 2: x = 6."),
        ("3x − 7 = 14", "7", "Add 7, divide by 3: x = 7."),
        ("5x + 10 = 35", "5", "Subtract 10, divide by 5: x = 5."),
        ("4x − 12 = 20", "8", "Add 12, divide by 4: x = 8."),
        ("x/3 + 2 = 7", "15", "Subtract 2, multiply by 3: x = 15."),
        ("2(x + 3) = 18", "6", "Divide by 2, subtract 3: x = 6."),
        ("7x = 49", "7", "Divide both sides by 7: x = 7."),
        ("x + 15 = 32", "17", "Subtract 15: x = 17."),
        ("6x − 4 = 2x + 12", "4", "4x = 16, so x = 4."),
        ("3x + 2 = x + 14", "6", "2x = 12, so x = 6."),
        ("9 − x = 4", "5", "x = 9 − 4 = 5."),
        ("2x + 1 = x + 9", "8", "x = 8."),
        ("5(x − 2) = 25", "7", "x − 2 = 5, so x = 7."),
        ("x/4 = 3", "12", "Multiply both sides by 4: x = 12."),
    ]
    for i, (eq, ans, expl) in enumerate(linear_scenarios):
        out.append(
            q(
                f"gre-foundation-quant-lin-{i + 1:03d}",
                "gre::quant::algebra::linear",
                "quant",
                f"Solve for x: {eq}",
                four_choices(
                    ans, [str(int(ans) + 2), str(int(ans) - 1), str(int(ans) + 4)]
                ),
                ans,
                expl,
                subtopic="linear_equation",
                concepts=["algebra", "linear_equation"],
                difficulty=pick_diff(idx),
                time_secs=90,
            )
        )
        idx += 1

    # --- Quadratic (13) ---
    quad_scenarios = [
        ("x² − 5x + 6 = 0", "2 or 3", "Factors to (x−2)(x−3)=0.", ["2", "3", "5", "6"]),
        (
            "x² − 9 = 0",
            "3 or −3",
            "Difference of squares: (x−3)(x+3)=0.",
            ["3", "−3", "9", "0"],
        ),
        ("x² + 6x + 9 = 0", "−3", "Perfect square (x+3)²=0.", ["−3", "3", "−6", "9"]),
        (
            "x² − x − 12 = 0",
            "4 or −3",
            "Factors to (x−4)(x+3)=0.",
            ["4", "−3", "12", "−4"],
        ),
        ("2x² = 18", "3 or −3", "x²=9.", ["3", "−3", "9", "6"]),
        (
            "x² + 2x − 8 = 0",
            "2 or −4",
            "Factors to (x+4)(x−2)=0.",
            ["2", "−4", "4", "−2"],
        ),
        ("x² − 16 = 0", "4 or −4", "Difference of squares.", ["4", "−4", "8", "16"]),
        (
            "x² + x − 6 = 0",
            "2 or −3",
            "Factors to (x+3)(x−2)=0.",
            ["2", "−3", "3", "−2"],
        ),
        ("x² − 4x = 0", "0 or 4", "x(x−4)=0.", ["0", "4", "−4", "2"]),
        (
            "x² + 5x + 6 = 0",
            "−2 or −3",
            "Factors to (x+2)(x+3)=0.",
            ["−2", "−3", "2", "3"],
        ),
        ("x² = 25", "5 or −5", "x = ±5.", ["5", "−5", "25", "0"]),
        (
            "x² − 2x − 15 = 0",
            "5 or −3",
            "Factors to (x−5)(x+3)=0.",
            ["5", "−3", "3", "−5"],
        ),
        ("3x² − 12 = 0", "2 or −2", "x²=4.", ["2", "−2", "4", "−4"]),
    ]
    for i, (eq, ans, expl, roots) in enumerate(quad_scenarios):
        out.append(
            q(
                f"gre-foundation-quant-quad-{i + 1:03d}",
                "gre::quant::algebra::quadratic",
                "quant",
                f"Which values of x satisfy {eq}?",
                four_choices(
                    ans,
                    [r for r in roots if r != ans.split()[0]][:3] or ["1", "0", "−1"],
                ),
                ans,
                expl,
                subtopic="quadratic_roots",
                concepts=["quadratic", "factoring"],
                difficulty=pick_diff(idx),
                time_secs=100,
            )
        )
        idx += 1

    # --- Triangles (12) ---
    tri_scenarios = [
        (3, 4, 5, "3-4-5 triangle: 5²=25."),
        (5, 12, 13, "5-12-13 Pythagorean triple."),
        (8, 15, 17, "8-15-17 Pythagorean triple."),
        (6, 8, 10, "6-8-10 is 3-4-5 scaled by 2."),
        (9, 12, 15, "9-12-15 is 3-4-5 scaled by 3."),
        (7, 24, 25, "7-24-25 Pythagorean triple."),
        (10, 24, 26, "10-24-26 is 5-12-13 scaled by 2."),
        (20, 21, 29, "20-21-29 Pythagorean triple."),
        (9, 40, 41, "9-40-41 Pythagorean triple."),
        (12, 16, 20, "12-16-20 is 3-4-5 scaled by 4."),
        (15, 20, 25, "15-20-25 is 3-4-5 scaled by 5."),
        (11, 60, 61, "11-60-61 Pythagorean triple."),
    ]
    for i, (a, b, c, expl) in enumerate(tri_scenarios):
        out.append(
            q(
                f"gre-foundation-quant-tri-{i + 1:03d}",
                "gre::quant::geometry::triangles",
                "quant",
                f"A right triangle has legs {a} and {b}. What is the length of the hypotenuse?",
                four_choices(str(c), triangle_hypotenuse_distractors(a, b, c)),
                str(c),
                expl,
                subtopic="pythagorean",
                concepts=["triangle", "pythagorean"],
                difficulty=pick_diff(idx),
                time_secs=85,
            )
        )
        idx += 1

    # Triangle angle
    out.append(
        q(
            "gre-foundation-quant-tri-013-extra",
            "gre::quant::geometry::triangles",
            "quant",
            "In a triangle, two angles measure 55° and 65°. What is the third angle?",
            four_choices("60°", ["50°", "70°", "55°"]),
            "60°",
            "Angles in a triangle sum to 180°: 180 − 55 − 65 = 60°.",
            subtopic="angle_sum",
            concepts=["triangle", "angles"],
            difficulty=pick_diff(idx),
            time_secs=60,
        )
    )
    idx += 1
    # Remove extra - we need exactly 12, pop the extra
    out.pop()

    # --- Circles (12) ---
    for i, (r, circ, area) in enumerate(
        [
            (3, "6π", "9π"),
            (5, "10π", "25π"),
            (7, "14π", "49π"),
            (4, "8π", "16π"),
            (6, "12π", "36π"),
            (10, "20π", "100π"),
            (2, "4π", "4π"),
            (8, "16π", "64π"),
            (9, "18π", "81π"),
            (1, "2π", "π"),
            (11, "22π", "121π"),
            (12, "24π", "144π"),
        ]
    ):
        if i % 2 == 0:
            stem = f"A circle has radius {r}. What is its circumference?"
            ans, expl = circ, f"C = 2πr = 2π({r}) = {circ}."
            wrong = circle_circumference_distractors(r)
        else:
            stem = f"A circle has radius {r}. What is its area?"
            ans, expl = area, f"A = πr² = π({r})² = {area}."
            wrong = circle_area_distractors(r)
        out.append(
            q(
                f"gre-foundation-quant-cir-{i + 1:03d}",
                "gre::quant::geometry::circles",
                "quant",
                stem,
                four_choices(ans, wrong),
                ans,
                expl,
                subtopic="circumference" if i % 2 == 0 else "area",
                concepts=["circle", "geometry"],
                difficulty=pick_diff(idx),
                time_secs=80,
            )
        )
        idx += 1

    # --- Data interpretation (18) ---
    tables = [
        (
            [("2019", 40), ("2020", 50), ("2021", 65)],
            "2020 to 2021",
            "30%",
            "Increase 15 on base 50.",
        ),
        (
            [("Q1", 120), ("Q2", 150), ("Q3", 135)],
            "Q1 to Q2",
            "25%",
            "Increase 30 on base 120.",
        ),
        (
            [("A", 80), ("B", 100), ("C", 90)],
            "B to C",
            "10%",
            "Decrease 10 on base 100.",
        ),
        (
            [("Jan", 200), ("Feb", 180), ("Mar", 216)],
            "Feb to Mar",
            "20%",
            "Increase 36 on base 180.",
        ),
        ([("X", 25), ("Y", 40), ("Z", 32)], "X to Y", "60%", "Increase 15 on base 25."),
        (
            [("P", 300), ("Q", 240), ("R", 288)],
            "Q to R",
            "20%",
            "Increase 48 on base 240.",
        ),
    ]
    for i in range(18):
        rows, period, ans, expl = tables[i % len(tables)]
        table = " | ".join(f"{k}: {v}" for k, v in rows)
        out.append(
            q(
                f"gre-foundation-quant-di-{i + 1:03d}",
                "gre::quant::data_interpretation",
                "quant",
                f"Sales by period — {table}. What is the percent change from {period}?",
                four_choices(
                    ans,
                    [pct for pct in ["15%", "25%", "40%", "50%"] if pct != ans],
                ),
                ans,
                expl,
                subtopic="percent_change_table",
                concepts=["data_interpretation", "percent_change"],
                difficulty=pick_diff(idx),
                time_secs=95,
            )
        )
        idx += 1

    # --- Probability (14) ---
    prob_scenarios = [
        (1, 6, "1/6", "One favorable face out of six."),
        (2, 6, "1/3", "Two favorable outcomes out of six."),
        (3, 10, "3/10", "Three out of ten equally likely."),
        (1, 4, "1/4", "One quarter of outcomes."),
        (5, 8, "5/8", "Five successes out of eight trials."),
        (1, 2, "1/2", "Fair coin single flip."),
        (3, 5, "3/5", "Three of five marbles."),
        (4, 9, "4/9", "Four favorable of nine."),
        (2, 5, "2/5", "Two of five."),
        (7, 12, "7/12", "Seven of twelve."),
        (1, 3, "1/3", "One of three."),
        (5, 6, "5/6", "Five of six."),
        (3, 4, "3/4", "Three of four."),
        (2, 7, "2/7", "Two of seven."),
    ]
    for i, (fav, total, ans, expl) in enumerate(prob_scenarios):
        out.append(
            q(
                f"gre-foundation-quant-prb-{i + 1:03d}",
                "gre::quant::statistics::probability",
                "quant",
                f"A fair experiment has {total} equally likely outcomes, {fav} of which are favorable. What is the probability of a favorable outcome?",
                four_choices(
                    ans, [f"{fav + 1}/{total}", f"{fav}/{total + 1}", f"1/{total + 2}"]
                ),
                ans,
                expl,
                subtopic="basic_probability",
                concepts=["probability", "fraction"],
                difficulty=pick_diff(idx),
                time_secs=75,
            )
        )
        idx += 1

    # --- Data analysis (13) ---
    # Fix mode ties and generate 13
    da_fixed = [
        ([2, 4, 6, 8, 10], "mean", "6", "Sum 30 divided by 5."),
        ([1, 3, 3, 7, 9], "median", "3", "Middle value of ordered set."),
        ([5, 5, 2, 8, 5], "mode", "5", "Most frequent value."),
        ([10, 20, 30], "mean", "20", "Sum 60 / 3."),
        ([4, 4, 9, 9, 12], "median", "9", "Middle of five values."),
        ([1, 2, 2, 2, 5], "mode", "2", "Appears three times."),
        ([3, 7, 7, 7, 11], "mean", "7", "Sum 35 / 5."),
        ([15, 20, 25, 30, 35], "median", "25", "Center value."),
        ([8, 8, 10, 12, 12, 12], "mode", "12", "Appears three times."),
        ([6, 6, 6, 6, 10], "mean", "6.8", "Sum 34 / 5 = 6.8."),
        ([2, 5, 8, 11, 14], "median", "8", "Middle value."),
        ([3, 3, 5, 7, 7, 7], "mode", "7", "Most frequent."),
        ([100, 200, 300, 400], "mean", "250", "Sum 1000 / 4."),
    ]
    for i, (data, stat, ans, expl) in enumerate(da_fixed):
        out.append(
            q(
                f"gre-foundation-quant-da-{i + 1:03d}",
                "gre::quant::statistics::data_analysis",
                "quant",
                f"What is the {stat} of the data set {', '.join(map(str, data))}?",
                four_choices(ans, data_analysis_distractors(stat, ans)),
                ans,
                expl,
                subtopic=stat,
                concepts=["statistics", stat],
                difficulty=pick_diff(idx),
                time_secs=85,
            )
        )
        idx += 1

    # --- Word problems (14) ---
    wp_scenarios = [
        (60, 3, 180, "Distance = rate × time."),
        (45, 4, 180, "45 mph for 4 hours."),
        (50, 2.5, 125, "50 × 2.5 = 125 miles."),
        (5, 10, 16, "Unit price $2; 8 pens cost $16.", "pens"),
        (4, 12, 24, "Unit price $3; 8 items cost $24.", "notebooks"),
        (13, 14, 14, "Consecutive integers summing to 27.", "integers"),
    ]
    for i in range(14):
        if i < 3:
            rate, hours, ans, expl = (
                wp_scenarios[i][0],
                wp_scenarios[i][1],
                wp_scenarios[i][2],
                wp_scenarios[i][3],
            )
            stem = f"A car travels at {rate} miles per hour for {hours} hours. How many miles does it travel?"
            correct = f"{ans} miles"
            choices = four_choices(
                correct, [f"{ans - 20} miles", f"{ans + 30} miles", f"{ans // 2} miles"]
            )
        elif i < 5:
            n, cost, ans, expl, item = wp_scenarios[i]
            stem = f"If {n} {item} cost ${cost}, how much do 8 {item} cost at the same unit price?"
            correct = f"${ans}"
            choices = four_choices(
                correct, [f"${ans - 4}", f"${ans + 6}", f"${ans // 2}"]
            )
        else:
            s = 10 + i * 3
            stem = f"The sum of two consecutive integers is {s}. What is the larger integer?"
            correct = str(s // 2 + 1)
            choices = four_choices(
                correct,
                [str(int(correct) - 2), str(int(correct) + 2), str(int(correct) - 1)],
            )
            expl = f"The integers are {s // 2} and {s // 2 + 1}."
        out.append(
            q(
                f"gre-foundation-quant-wp-{i + 1:03d}",
                "gre::quant::word_problems",
                "quant",
                stem,
                choices,
                correct,
                expl,
                subtopic="rate_distance"
                if i < 3
                else "unit_price"
                if i < 5
                else "consecutive_integers",
                concepts=["word_problem"],
                difficulty=pick_diff(idx),
                time_secs=90,
            )
        )
        idx += 1

    # --- Number properties (13) ---
    np_items = [
        (
            "What is the smallest prime greater than 20?",
            "23",
            "23 is prime; 21 and 22 are composite.",
            ["21", "22", "24"],
        ),
        (
            "What is the GCD of 24 and 36?",
            "12",
            "Shared divisors up to 12.",
            ["6", "8", "18"],
        ),
        (
            "What is the LCM of 4 and 6?",
            "12",
            "Least common multiple.",
            ["24", "8", "6"],
        ),
        ("How many factors does 12 have?", "6", "1, 2, 3, 4, 6, 12.", ["4", "5", "8"]),
        (
            "What is the remainder when 47 is divided by 5?",
            "2",
            "47 = 9×5 + 2.",
            ["3", "1", "4"],
        ),
        (
            "Which is divisible by 3?",
            "57",
            "5+7=12, divisible by 3.",
            ["58", "59", "61"],
        ),
        ("What is 2⁵?", "32", "2×2×2×2×2 = 32.", ["16", "64", "25"]),
        ("How many primes between 10 and 20?", "4", "11, 13, 17, 19.", ["3", "5", "6"]),
        (
            "Is 91 prime?",
            "No",
            "91 = 7 × 13.",
            ["Yes", "Cannot determine", "Only if odd"],
        ),
        ("Units digit of 7⁴?", "1", "7¹=7, 7²=9, 7³=3, 7⁴=1.", ["7", "9", "3"]),
        (
            "Sum of first 5 positive integers?",
            "15",
            "1+2+3+4+5=15.",
            ["10", "20", "12"],
        ),
        (
            "How many even integers from 1 to 10?",
            "5",
            "2, 4, 6, 8, 10.",
            ["4", "6", "3"],
        ),
        ("Which is a perfect square?", "49", "7² = 49.", ["45", "54", "63"]),
    ]
    for i, (stem, ans, expl, wrong) in enumerate(np_items):
        out.append(
            q(
                f"gre-foundation-quant-np-{i + 1:03d}",
                "gre::quant::number_properties",
                "quant",
                stem,
                four_choices(ans, wrong),
                ans,
                expl,
                subtopic="number_properties",
                concepts=["number_properties", "integer"],
                difficulty=pick_diff(idx),
                time_secs=80,
            )
        )
        idx += 1

    assert len(out) >= MIN_FOUNDATION_QUANT, len(out)
    return out[:150] if len(out) > 150 else out


def gen_verbal() -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    idx = 0

    # Text completion - 30 (1/2/3 blank via subtopic)
    tc_items = [
        (
            "The diplomat's remarks were so ______ that they eased tensions rather than inflaming them.",
            ["conciliatory", "incendiary", "ambiguous", "grating"],
            "conciliatory",
            "Easing tensions implies a peacemaking tone.",
            "one_blank",
            "text_completion",
        ),
        (
            "Although the evidence was ______, the jury reached a unanimous verdict.",
            ["circumstantial", "conclusive", "overwhelming", "unambiguous"],
            "circumstantial",
            '"Although" signals contrast with reaching verdict despite imperfect evidence.',
            "one_blank",
            "text_completion",
        ),
        (
            "The professor's lecture was ______: every claim was supported by primary sources.",
            ["meticulous", "speculative", "cursory", "rambling"],
            "meticulous",
            "Supported by primary sources implies careful scholarship.",
            "one_blank",
            "text_completion",
        ),
        (
            "Far from being ______, the reform actually strengthened institutional accountability.",
            ["redundant", "transformative", "superficial", "obsolete"],
            "superficial",
            '"Far from" contrasts with strengthening accountability.',
            "one_blank",
            "text_completion",
        ),
        (
            "The artist's early work was ______, but her mature paintings show remarkable restraint.",
            ["florid", "austere", "muted", "minimal"],
            "florid",
            "Contrast with mature restraint implies early excess.",
            "one_blank",
            "text_completion",
        ),
        (
            "The CEO's apology sounded ______ to employees who had heard similar promises before.",
            ["hollow", "sincere", "heartfelt", "candid"],
            "hollow",
            "Skepticism from repeated promises implies emptiness.",
            "one_blank",
            "text_completion",
        ),
        (
            "The novel's plot is ______, unfolding through a series of seemingly unrelated vignettes.",
            ["episodic", "linear", "formulaic", "predictable"],
            "episodic",
            "Unrelated vignettes suggest episodic structure.",
            "one_blank",
            "text_completion",
        ),
        (
            "Critics found the policy ______, noting it addressed symptoms rather than root causes.",
            ["superficial", "comprehensive", "innovative", "rigorous"],
            "superficial",
            "Treating symptoms not causes is superficial.",
            "one_blank",
            "text_completion",
        ),
        (
            "Her tone was deliberately ______, masking strong opinions behind neutral phrasing.",
            ["measured", "strident", "effusive", "combative"],
            "measured",
            "Masking strong opinions behind neutral phrasing is measured.",
            "one_blank",
            "text_completion",
        ),
        (
            "The habitat recovery was ______; within a decade native species returned in large numbers.",
            ["robust", "tenuous", "marginal", "illusory"],
            "robust",
            "Large-scale species return indicates strong recovery.",
            "one_blank",
            "text_completion",
        ),
        # two-blank style (single blank but labeled subtopic two_blank)
        (
            "The committee's report was at once ______ and ______: thorough in detail yet accessible to lay readers.",
            [
                "scholarly ... lucid",
                "opaque ... jargon-filled",
                "brief ... superficial",
                "technical ... abstruse",
            ],
            "scholarly ... lucid",
            "Thorough yet accessible matches scholarly and lucid.",
            "two_blank",
            "text_completion",
        ),
        (
            "The witness was ______ but not ______: credible on facts yet unreliable on motives.",
            [
                "credible ... unreliable",
                "deceptive ... honest",
                "confused ... clear",
                "hostile ... cooperative",
            ],
            "credible ... unreliable",
            "Reliable on facts, not on motives.",
            "two_blank",
            "text_completion",
        ),
        (
            "The market's reaction was ______ rather than ______: investors paused instead of panicking.",
            [
                "cautious ... hysterical",
                "euphoric ... restrained",
                "indifferent ... engaged",
                "volatile ... stable",
            ],
            "cautious ... hysterical",
            "Paused instead of panicking is cautious not hysterical.",
            "two_blank",
            "text_completion",
        ),
        (
            "The author's style is ______ yet never ______: complex ideas expressed without needless ornament.",
            [
                "dense ... florid",
                "simple ... spare",
                "plain ... ornate",
                " terse ... verbose",
            ],
            "dense ... florid",
            "Complex without ornament: dense not florid.",
            "two_blank",
            "text_completion",
        ),
        (
            "The treaty proved ______ in letter but ______ in practice.",
            [
                "ambitious ... ineffectual",
                "modest ... transformative",
                "binding ... enforceable",
                " concise ... sweeping",
            ],
            "ambitious ... ineffectual",
            "Strong on paper, weak in execution.",
            "two_blank",
            "text_completion",
        ),
        # three-blank
        (
            "The theory, though ______ in scope, remained ______ in evidence and ______ in predictive power.",
            [
                "expansive ... thin ... limited",
                "narrow ... robust ... strong",
                " modest ... ample ... proven",
                " sweeping ... solid ... unmatched",
            ],
            "expansive ... thin ... limited",
            "Large scope but weak evidence and prediction.",
            "three_blank",
            "text_completion",
        ),
        (
            "The memoir is ______ in tone, ______ in structure, and ______ in its refusal to settle scores.",
            [
                "reflective ... fragmented ... restrained",
                "angry ... linear ... vindictive",
                "haughty ... rigid ... aggressive",
                " playful ... chaotic ... bitter",
            ],
            "reflective ... fragmented ... restrained",
            "Thoughtful, non-linear, not vindictive.",
            "three_blank",
            "text_completion",
        ),
        (
            "Early results were ______; later trials ______ the findings; the final meta-analysis ______ confidence.",
            [
                "promising ... corroborated ... bolstered",
                "definitive ... contradicted ... erased",
                "mixed ... ignored ... undermined",
                "negative ... replicated ... shattered",
            ],
            "promising ... corroborated ... bolstered",
            "Progressive strengthening of evidence.",
            "three_blank",
            "text_completion",
        ),
    ]
    # Pad tc to 30
    extra_tc = [
        (
            "The politician's rhetoric grew increasingly ______ as the election neared.",
            ["inflammatory", "subdued", "technical", "diplomatic"],
            "inflammatory",
            "Heated election rhetoric.",
            "one_blank",
            "text_completion",
        ),
        (
            "The scientist remained ______ despite fierce criticism from peers.",
            ["resolute", "wavering", "indifferent", "cynical"],
            "resolute",
            "Unmoved by criticism.",
            "one_blank",
            "text_completion",
        ),
        (
            "The archive's value lies not in its size but in its ______.",
            ["rarity", "bulk", "disorder", "redundancy"],
            "rarity",
            "Value from rare materials.",
            "one_blank",
            "text_completion",
        ),
        (
            "The translation is faithful but occasionally ______.",
            ["awkward", " lyrical", "innovative", " concise"],
            "awkward",
            "Faithful but clumsy phrasing.",
            "one_blank",
            "text_completion",
        ),
        (
            "Investors treated the forecast as ______ until revenue confirmed it.",
            ["speculative", "canonical", "immutable", " trivial"],
            "speculative",
            "Unconfirmed until revenue.",
            "one_blank",
            "text_completion",
        ),
        (
            "The dean's policy was ______: strict on deadlines but flexible on methods.",
            ["pragmatic", "arbitrary", " punitive", " chaotic"],
            "pragmatic",
            "Balanced strictness and flexibility.",
            "one_blank",
            "text_completion",
        ),
        (
            "The poem's imagery is ______, evoking loss without naming it.",
            ["evocative", "literal", "clinical", " didactic"],
            "evocative",
            "Suggestive imagery.",
            "one_blank",
            "text_completion",
        ),
        (
            "The startup's growth was ______, doubling users each quarter.",
            ["exponential", "negligible", " erratic", " stagnant"],
            "exponential",
            "Doubling each quarter.",
            "one_blank",
            "text_completion",
        ),
        (
            "The historian's account is ______, privileging archival records over anecdote.",
            ["document-driven", " fanciful", " impressionistic", " partisan"],
            "document-driven",
            "Archival over anecdote.",
            "one_blank",
            "text_completion",
        ),
        (
            "The critic dismissed the film as ______ entertainment.",
            ["mindless", " thought-provoking", " nuanced", " ambitious"],
            "mindless",
            "Dismissive tone.",
            "one_blank",
            "text_completion",
        ),
        (
            "The lawyer's argument was ______, anticipating every counterclaim.",
            ["comprehensive", " narrow", " impulsive", " fragmentary"],
            "comprehensive",
            "Anticipating counterclaims.",
            "one_blank",
            "text_completion",
        ),
        (
            "The patient's recovery was ______, with setbacks followed by steady gains.",
            ["uneven", " instantaneous", " illusory", " uniform"],
            "uneven",
            "Setbacks then gains.",
            "one_blank",
            "text_completion",
        ),
        (
            "The essay's thesis is ______ but its examples are ______.",
            [
                "clear ... thin",
                " obscure ... abundant",
                " radical ... conventional",
                " tentative ... decisive",
            ],
            "clear ... thin",
            "Clear thesis, weak support.",
            "two_blank",
            "text_completion",
        ),
    ]
    tc_all = tc_items + extra_tc
    for i, (stem, choices, ans, expl, sub, fmt) in enumerate(tc_all[:30]):
        out.append(
            q(
                f"gre-foundation-verbal-tc-{i + 1:03d}",
                "gre::verbal::text_completion",
                "verbal",
                stem,
                choices
                if len(choices) == 4
                else [choices[0], choices[1], choices[2], choices[3]],
                ans,
                expl,
                fmt=fmt,
                subtopic=sub,
                question_type="text_completion",
                concepts=["text_completion", sub],
                difficulty=pick_diff(idx),
                time_secs=75 if sub == "one_blank" else 90,
            )
        )
        idx += 1

    # Sentence equivalence - 22
    se_items = [
        (
            "The manager's feedback was surprisingly ______, offering specific praise rather than vague encouragement.",
            ["forthright", "perfunctory", "candid", "grudging", "evasive", "elaborate"],
            "forthright",
            "Specific praise suggests directness (forthright/candid).",
            "sentence_equivalence",
        ),
        (
            "The ruins were so ______ that archaeologists could barely distinguish walls from rubble.",
            [
                " dilapidated",
                " pristine",
                " intact",
                " restored",
                " fragile",
                " ancient",
            ],
            " dilapidated",
            "Barely distinguish walls — severely decayed.",
            "sentence_equivalence",
        ),
        (
            "Her explanation was ______, leaving no doubt about the committee's next steps.",
            ["unambiguous", " cryptic", " equivocal", " lucid", " muddled", " terse"],
            "unambiguous",
            "No doubt — clear meaning.",
            "sentence_equivalence",
        ),
        (
            "The negotiators remained ______ despite hours of heated debate.",
            [
                "composed",
                " agitated",
                " unruffled",
                " volatile",
                " indifferent",
                " frantic",
            ],
            "composed",
            "Calm despite heat (composed/unruffled).",
            "sentence_equivalence",
        ),
        (
            "The author's irony is so ______ that some readers miss the critique entirely.",
            [" subtle", " heavy-handed", " blunt", " overt", " clumsy", " obvious"],
            " subtle",
            "Missed critique implies subtle irony.",
            "sentence_equivalence",
        ),
    ]
    se_extra = [
        (
            "The policy proved ______, solving one problem while creating two others.",
            [
                " counterproductive",
                " beneficial",
                " streamlined",
                " efficient",
                " redundant",
                " elegant",
            ],
            " counterproductive",
            "Creates new problems.",
            "sentence_equivalence",
        ),
        (
            "The witness was ______, answering only what was asked.",
            [
                " terse",
                " loquacious",
                " expansive",
                " effusive",
                " rambling",
                " verbose",
            ],
            " terse",
            "Minimal answers.",
            "sentence_equivalence",
        ),
        (
            "The landscape looked ______ after weeks of drought.",
            [" parched", " lush", " verdant", " flooded", " fertile", " saturated"],
            " parched",
            "Drought dries land.",
            "sentence_equivalence",
        ),
        (
            "The professor's reputation is ______; colleagues cite her work across disciplines.",
            [
                " far-reaching",
                " negligible",
                " insular",
                " disputed",
                " narrow",
                " questionable",
            ],
            " far-reaching",
            "Cited across disciplines.",
            "sentence_equivalence",
        ),
        (
            "The contract language is intentionally ______, allowing flexible interpretation.",
            [" vague", " precise", " explicit", " rigid", " definitive", " airtight"],
            " vague",
            "Flexible interpretation.",
            "sentence_equivalence",
        ),
        (
            "The team's victory was ______, earned through disciplined execution.",
            [
                " deserved",
                " accidental",
                " fluky",
                " undeserved",
                " arbitrary",
                " hollow",
            ],
            " deserved",
            "Earned through discipline.",
            "sentence_equivalence",
        ),
        (
            "The memoir's tone is ______ rather than nostalgic.",
            [
                " reflective",
                " sentimental",
                " maudlin",
                " wistful",
                " cloying",
                " romantic",
            ],
            " reflective",
            "Thoughtful not nostalgic.",
            "sentence_equivalence",
        ),
        (
            "The CEO's plan is ______, relying on untested assumptions.",
            [
                " speculative",
                " proven",
                " conservative",
                " empirical",
                " validated",
                " cautious",
            ],
            " speculative",
            "Untested assumptions.",
            "sentence_equivalence",
        ),
        (
            "The critic found the novel ______, praising its structure but not its characters.",
            [" uneven", " flawless", " cohesive", " seamless", " uniform", " polished"],
            " uneven",
            "Strong structure, weak characters.",
            "sentence_equivalence",
        ),
        (
            "The speaker was ______, moving the audience without manipulative tricks.",
            [
                " persuasive",
                " coercive",
                " compelling",
                " bullying",
                " overbearing",
                " dictatorial",
            ],
            " persuasive",
            "Moved audience ethically.",
            "sentence_equivalence",
        ),
        (
            "The data were ______, showing a clear trend across all regions.",
            [
                " consistent",
                " contradictory",
                " anomalous",
                " scattered",
                " incompatible",
                " erratic",
            ],
            " consistent",
            "Clear trend everywhere.",
            "sentence_equivalence",
        ),
        (
            "The editor's cuts made the essay ______.",
            [
                " tighter",
                " sprawling",
                " diffuse",
                " bloated",
                " redundant",
                " wandering",
            ],
            " tighter",
            "Cuts improve focus.",
            "sentence_equivalence",
        ),
        (
            "The philosopher's argument is ______, built from definitions all parties accept.",
            [
                " cogent",
                " fallacious",
                " incoherent",
                " circular",
                " muddled",
                " self-contradictory",
            ],
            " cogent",
            "Accepted definitions, sound reasoning.",
            "sentence_equivalence",
        ),
        (
            "The harbor was ______ at dawn, with only a few boats stirring.",
            [" tranquil", " chaotic", " bustling", " frenzied", " crowded", " noisy"],
            " tranquil",
            "Few boats, calm.",
            "sentence_equivalence",
        ),
        (
            "The student's essay was ______, full of unsupported generalizations.",
            [
                " superficial",
                " penetrating",
                " nuanced",
                " rigorous",
                " exhaustive",
                " meticulous",
            ],
            " superficial",
            "Unsupported generalizations.",
            "sentence_equivalence",
        ),
        (
            "The merger announcement was ______, sending shares down sharply.",
            [
                " unsettling",
                " reassuring",
                " welcome",
                " expected",
                " soothing",
                " predictable",
            ],
            " unsettling",
            "Shares fell sharply.",
            "sentence_equivalence",
        ),
        (
            "The gardener kept the hedge ______, trimming it every week.",
            [" neat", " wild", " overgrown", " unruly", " tangled", " shaggy"],
            " neat",
            "Weekly trimming.",
            "sentence_equivalence",
        ),
    ]
    for i, (stem, choices, ans, expl, fmt) in enumerate((se_items + se_extra)[:22]):
        out.append(
            q(
                f"gre-foundation-verbal-se-{i + 1:03d}",
                "gre::verbal::sentence_equivalence",
                "verbal",
                stem,
                [c.strip() for c in choices],
                ans.strip(),
                expl,
                fmt=fmt,
                subtopic="sentence_equivalence",
                question_type="sentence_equivalence",
                concepts=["sentence_equivalence", "vocabulary"],
                difficulty=pick_diff(idx),
                time_secs=85,
            )
        )
        idx += 1

    # Reading passages - shared passages for RC types
    passages = [
        (
            'Urban planners increasingly advocate for "15-minute cities," neighborhoods where residents can reach daily needs within a short walk or bike ride. Proponents argue this design reduces car dependence and strengthens local economies. Skeptics counter that rigid zoning for mixed-use development can raise housing costs and displace long-term residents.',
            [
                (
                    "gre::verbal::reading::main_idea",
                    "main_idea",
                    "The passage primarily presents",
                    "a planning concept along with support and criticism",
                    [
                        "a planning concept along with support and criticism",
                        "proof that 15-minute cities always lower costs",
                        "a history of urban zoning law",
                        "reasons to eliminate all car traffic",
                    ],
                    "a planning concept along with support and criticism",
                    "The passage explains the idea and both pro and con views.",
                ),
                (
                    "gre::verbal::reading::inference",
                    "inference",
                    "It can be inferred that skeptics believe 15-minute city policies",
                    "may have unintended social costs",
                    [
                        "may have unintended social costs",
                        "will definitely reduce all traffic",
                        "are unsupported by any evidence",
                        "originated in rural areas",
                    ],
                    "may have unintended social costs",
                    "Displacement and higher costs are social costs.",
                ),
                (
                    "gre::verbal::reading::detail",
                    "detail",
                    "According to the passage, proponents claim 15-minute cities",
                    "reduce car dependence",
                    [
                        "reduce car dependence",
                        "eliminate all local businesses",
                        "require highway expansion",
                        "ban mixed-use zoning",
                    ],
                    "reduce car dependence",
                    "Explicitly stated benefit.",
                ),
                (
                    "gre::verbal::reading::function",
                    "function",
                    "The second sentence primarily serves to",
                    "state a claimed benefit of the design",
                    [
                        "state a claimed benefit of the design",
                        "refute the entire concept",
                        "define zoning terminology",
                        "introduce a historical example",
                    ],
                    "state a claimed benefit of the design",
                    "It lists proponents' arguments.",
                ),
            ],
        ),
        (
            "Bioluminescent fungi on forest floors produce light through a chemical reaction involving luciferin. Researchers once assumed the glow was a accidental byproduct of metabolism. Recent experiments suggest the light may attract insects that spread fungal spores, indicating the trait could be adaptive rather than incidental.",
            [
                (
                    "gre::verbal::reading::main_idea",
                    "main_idea",
                    "The passage is mainly concerned with",
                    "revising an explanation for fungal bioluminescence",
                    [
                        "revising an explanation for fungal bioluminescence",
                        "comparing luciferin in fungi and fireflies",
                        "arguing that bioluminescence is always harmful",
                        "describing how to cultivate glowing mushrooms",
                    ],
                    "revising an explanation for fungal bioluminescence",
                    "Moves from old assumption to new adaptive view.",
                ),
                (
                    "gre::verbal::reading::inference",
                    "inference",
                    "The passage suggests that earlier researchers viewed fungal glow as",
                    "unrelated to evolutionary advantage",
                    [
                        "unrelated to evolutionary advantage",
                        "essential for spore production",
                        "stronger in daylight",
                        "identical to insect bioluminescence",
                    ],
                    "unrelated to evolutionary advantage",
                    "They assumed it was metabolic byproduct, not adaptive.",
                ),
                (
                    "gre::verbal::reading::detail",
                    "detail",
                    "According to the passage, luciferin is involved in",
                    "a chemical reaction producing light",
                    [
                        "a chemical reaction producing light",
                        "photosynthesis in trees",
                        "insect digestion",
                        "spore destruction",
                    ],
                    "a chemical reaction producing light",
                    "Stated in first sentence.",
                ),
                (
                    "gre::verbal::reading::function",
                    "function",
                    "The final sentence primarily",
                    "offers a possible adaptive purpose for the trait",
                    [
                        "offers a possible adaptive purpose for the trait",
                        "dismisses all prior research",
                        "defines the term luciferin",
                        " compares fungi to plants",
                    ],
                    "offers a possible adaptive purpose for the trait",
                    "Introduces spore-spreading hypothesis.",
                ),
            ],
        ),
        (
            "Medieval scribes copied manuscripts by hand, a labor that preserved texts but introduced errors. Some scholars treat each variant reading as noise; others argue variants reveal how communities adapted texts to local beliefs. Digital collation now allows rapid comparison of hundreds of witnesses, shifting debate from whether to compare to how comparisons should inform interpretation.",
            [
                (
                    "gre::verbal::reading::main_idea",
                    "main_idea",
                    "The passage primarily discusses",
                    "changing scholarly approaches to textual variants",
                    [
                        "changing scholarly approaches to textual variants",
                        "the decline of medieval book production",
                        "why scribes were always inaccurate",
                        "how to eliminate all copying errors",
                    ],
                    "changing scholarly approaches to textual variants",
                    "Contrasts views on variants and notes digital shift.",
                ),
                (
                    "gre::verbal::reading::inference",
                    "inference",
                    "It can be inferred that digital collation has",
                    "expanded the scale of textual comparison",
                    [
                        "expanded the scale of textual comparison",
                        "proven that all variants are meaningless",
                        " replaced the need for interpretation",
                        "shown scribes never made errors",
                    ],
                    "expanded the scale of textual comparison",
                    "Compare hundreds of witnesses rapidly.",
                ),
                (
                    "gre::verbal::reading::detail",
                    "detail",
                    "Some scholars view variant readings as",
                    "noise",
                    [
                        "noise",
                        " sacred doctrine",
                        " legal evidence",
                        " artistic embellishment only",
                    ],
                    "noise",
                    "Explicitly stated.",
                ),
                (
                    "gre::verbal::reading::function",
                    "function",
                    "The third sentence serves to",
                    "present an alternative view of variants",
                    [
                        "present an alternative view of variants",
                        "summarize digital software features",
                        "criticize all medieval scribes",
                        "define manuscript illumination",
                    ],
                    "present an alternative view of variants",
                    "Others see variants as revealing adaptation.",
                ),
            ],
        ),
    ]

    rc_counts = {
        "gre::verbal::reading::inference": 18,
        "gre::verbal::reading::main_idea": 15,
        "gre::verbal::reading::detail": 12,
        "gre::verbal::reading::function": 18,
    }
    rc_generated = {k: 0 for k in rc_counts}

    passage_idx = 0
    while any(rc_generated[k] < rc_counts[k] for k in rc_counts):
        passage, questions = passages[passage_idx % len(passages)]
        for topic, sub, qstem, ans, choices, correct, expl in questions:
            if rc_generated[topic] >= rc_counts[topic]:
                continue
            n = rc_generated[topic] + 1
            out.append(
                q(
                    f"gre-foundation-verbal-rc-{sub}-{n:03d}",
                    topic,
                    "verbal",
                    f"Passage: {passage}\n\n{qstem}?",
                    four_choices(correct, [c for c in choices if c != correct][:3]),
                    correct,
                    expl,
                    fmt="reading_comprehension",
                    subtopic=sub,
                    question_type="reading_comprehension",
                    concepts=["reading_comprehension", sub],
                    difficulty=pick_diff(idx),
                    time_secs=120,
                )
            )
            rc_generated[topic] += 1
            idx += 1
        passage_idx += 1

    # Vocabulary context - 17
    vocab_ctx = [
        (
            "The CEO's ______ remarks calmed investors worried about the merger.",
            ["assuring", "cryptic", "hostile", "flippant"],
            "assuring",
            "Calmed worried investors.",
            "context_clue",
        ),
        (
            "Because the contract was ______, both parties interpreted key clauses differently.",
            ["ambiguous", "explicit", " concise", " binding"],
            "ambiguous",
            "Different interpretations imply ambiguity.",
            "context_clue",
        ),
        (
            "The desert flora is ______, thriving on minimal rainfall.",
            ["hardy", "delicate", " tropical", " aquatic"],
            "hardy",
            "Thrives with little water.",
            "context_clue",
        ),
        (
            "Her ______ smile suggested she knew more than she stated.",
            ["knowing", "vacant", "forced", "timid"],
            "knowing",
            "Implied hidden knowledge.",
            "context_clue",
        ),
        (
            "The lawyer's ______ questioning exposed inconsistencies in the testimony.",
            ["probing", "perfunctory", "gentle", "random"],
            "probing",
            "Exposed inconsistencies.",
            "context_clue",
        ),
    ]
    vocab_ctx_extra = [
        (
            "The mountain trail was ______, forcing hikers to proceed slowly.",
            ["treacherous", " gentle", " paved", " crowded"],
            "treacherous",
            "Forced slow progress.",
            "context_clue",
        ),
        (
            "The child's ______ curiosity led her to dismantle the clock.",
            ["insatiable", " fleeting", " muted", " cautious"],
            "insatiable",
            "Dismantling shows intense curiosity.",
            "context_clue",
        ),
        (
            "The speaker's ______ diction made complex ideas accessible.",
            [" lucid", " ornate", " archaic", " opaque"],
            " lucid",
            "Accessible complex ideas.",
            "context_clue",
        ),
        (
            "After the scandal, the politician's reputation was ______.",
            [" tarnished", " enhanced", " unchanged", " celebrated"],
            " tarnished",
            "Post-scandal damage.",
            "context_clue",
        ),
        (
            "The fabric's ______ texture felt rough against the skin.",
            [" coarse", " silky", " smooth", " elastic"],
            " coarse",
            "Rough feel.",
            "context_clue",
        ),
        (
            "The mentor's advice was ______: brief but decisive.",
            [" pithy", " rambling", " verbose", " equivocal"],
            " pithy",
            "Brief and decisive.",
            "context_clue",
        ),
        (
            "The evidence was ______, pointing clearly to one suspect.",
            [" incriminating", " exonerating", " irrelevant", " ambiguous"],
            " incriminating",
            "Points to suspect.",
            "context_clue",
        ),
        (
            "The orchestra's performance was ______, earning a standing ovation.",
            [" stellar", " mediocre", " tentative", " disjointed"],
            " stellar",
            "Standing ovation.",
            "context_clue",
        ),
        (
            "The policy had ______ effects, harming the groups it aimed to help.",
            [" perverse", " beneficial", " neutral", " predictable"],
            " perverse",
            "Harm despite intent to help.",
            "context_clue",
        ),
        (
            "The room was ______ after the windows were opened.",
            [" airy", " stuffy", " dim", " cluttered"],
            " airy",
            "Open windows freshen space.",
            "context_clue",
        ),
        (
            "The researcher remained ______, refusing to announce results before replication.",
            [" circumspect", " reckless", " boastful", " impatient"],
            " circumspect",
            "Waited for replication.",
            "context_clue",
        ),
        (
            "The debate turned ______ when personal insults replaced argument.",
            [" acrimonious", " collegial", " productive", " subdued"],
            " acrimonious",
            "Personal insults.",
            "context_clue",
        ),
    ]
    for i, (stem, choices, ans, expl, sub) in enumerate(
        (vocab_ctx + vocab_ctx_extra)[:17]
    ):
        out.append(
            q(
                f"gre-foundation-verbal-vctx-{i + 1:03d}",
                "gre::verbal::vocabulary::context",
                "verbal",
                stem,
                [c.strip() for c in choices],
                ans.strip(),
                expl,
                fmt="vocabulary",
                subtopic=sub,
                question_type="vocabulary_in_context",
                concepts=["vocabulary", "context_clues"],
                difficulty=pick_diff(idx),
                time_secs=70,
            )
        )
        idx += 1

    # Advanced vocabulary - 18
    adv_vocab = [
        (
            "The senator's ______ speech alienated allies with its harsh tone.",
            ["truculent", "conciliatory", "measured", "diplomatic"],
            "truculent",
            "Harsh, aggressive tone.",
            "advanced_vocab",
        ),
        (
            "The essay's argument was ______, relying on unstated assumptions.",
            ["tendentious", "evenhanded", "dispassionate", "objective"],
            "tendentious",
            "Biased, assumption-laden.",
            "advanced_vocab",
        ),
        (
            "Her ______ wit defused tension without minimizing the problem.",
            ["mordant", "saccharine", "clumsy", "obtuse"],
            "mordant",
            "Dark, sharp humor.",
            "advanced_vocab",
        ),
        (
            "The CEO's ______ praise sounded insincere to employees.",
            [" fulsome", " restrained", " grudging", " terse"],
            " fulsome",
            "Excessive, insincere praise.",
            "advanced_vocab",
        ),
        (
            "The critic's review was ______, finding fault in minor details.",
            [" captious", " generous", " sweeping", " indifferent"],
            " captious",
            "Overly fault-finding.",
            "advanced_vocab",
        ),
        (
            "The peace accord was ______, satisfying neither side completely.",
            [" imperfect", " flawless", " comprehensive", " enduring"],
            " imperfect",
            "Neither side fully satisfied.",
            "advanced_vocab",
        ),
        (
            "The artist's style is ______, avoiding emotional display.",
            [" austere", " florid", " effusive", " sentimental"],
            " austere",
            "Restrained, no emotional display.",
            "advanced_vocab",
        ),
        (
            "The witness gave a ______ account, omitting no relevant detail.",
            [" exhaustive", " cursory", " misleading", " partial"],
            " exhaustive",
            "Complete detail.",
            "advanced_vocab",
        ),
        (
            "The plan is ______, depending on conditions that may not hold.",
            [" contingent", " inevitable", " immutable", " self-evident"],
            " contingent",
            "Depends on uncertain conditions.",
            "advanced_vocab",
        ),
        (
            "The professor's reputation for ______ makes students prepare thoroughly.",
            [" rigor", " leniency", " absentmindedness", " favoritism"],
            " rigor",
            "Strict standards.",
            "advanced_vocab",
        ),
        (
            "The novel's plot twist felt ______, contradicting established facts.",
            [" arbitrary", " inevitable", " foreshadowed", " nuanced"],
            " arbitrary",
            "Unsupported by prior facts.",
            "advanced_vocab",
        ),
        (
            "The diplomat's ______ approach won trust from both factions.",
            [" evenhanded", " partisan", " aggressive", " evasive"],
            " evenhanded",
            "Fair to both sides.",
            "advanced_vocab",
        ),
        (
            "The company's ______ growth could not continue indefinitely.",
            [" meteoric", " sluggish", " negligible", " stagnant"],
            " meteoric",
            "Rapid rise implies unsustainable pace.",
            "advanced_vocab",
        ),
        (
            "The judge's ruling was ______, grounded in precedent.",
            [" judicious", " whimsical", " impulsive", " opaque"],
            " judicious",
            "Precedent-based, wise.",
            "advanced_vocab",
        ),
        (
            "The speech was ______, full of empty slogans.",
            [" platitudinous", " incisive", " technical", " succinct"],
            " platitudinous",
            "Empty slogans.",
            "advanced_vocab",
        ),
        (
            "The researcher remained ______ about results until data were verified.",
            [" circumspect", " boastful", " dismissive", " credulous"],
            " circumspect",
            "Cautious before verification.",
            "advanced_vocab",
        ),
        (
            "The memoir's portrait of childhood is ______ rather than nostalgic.",
            [" unsentimental", " maudlin", " romanticized", " hazy"],
            " unsentimental",
            "Not nostalgic.",
            "advanced_vocab",
        ),
        (
            "The opposition's argument is ______, attacking motives instead of evidence.",
            [" ad hominem", " empirical", " syllogistic", " constructive"],
            " ad hominem",
            "Attacks motives not evidence.",
            "advanced_vocab",
        ),
    ]
    for i, (stem, choices, ans, expl, sub) in enumerate(adv_vocab[:18]):
        out.append(
            q(
                f"gre-foundation-verbal-vadv-{i + 1:03d}",
                "gre::verbal::vocabulary::advanced",
                "verbal",
                stem,
                [c.strip() for c in choices],
                ans.strip(),
                expl,
                fmt="vocabulary",
                subtopic=sub,
                question_type="advanced_vocabulary",
                concepts=["vocabulary", "advanced"],
                difficulty=pick_diff(idx),
                time_secs=75,
            )
        )
        idx += 1

    assert len(out) >= MIN_FOUNDATION_VERBAL, len(out)
    return out[:150]


def gen_awa() -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    idx = 0

    issue_prompts = [
        (
            "Governments should prioritize funding for the arts over funding for scientific research.",
            "Scientific research often underpins technologies and medical advances that benefit society broadly.",
            "Arts funding and scientific research serve different but complementary public goods.",
        ),
        (
            "Success in any field requires more luck than skill.",
            "Attributing success mainly to luck ignores the role of deliberate practice and preparation.",
            "Skill and luck interact; framing the issue as either-or oversimplifies.",
        ),
        (
            "Universities should abolish letter grades and adopt pass/fail evaluation.",
            "Letter grades can motivate excellence and signal achievement to employers.",
            "Pass/fail may reduce unhealthy competition but can also blur meaningful distinctions.",
        ),
        (
            "Technology makes people less capable of thinking for themselves.",
            "Technology offloads routine tasks, freeing cognitive resources for higher-order thinking.",
            "Effects depend on how technology is used, not technology alone.",
        ),
        (
            "It is primarily the responsibility of individuals, not governments, to reduce environmental harm.",
            "Individual choices alone cannot address large-scale pollution without policy.",
            "Both individual action and government regulation are necessary at scale.",
        ),
        (
            "Formal education becomes obsolete within a decade of graduation.",
            "Core skills such as reasoning and communication remain valuable across careers.",
            "Education must evolve, but foundational skills retain long-term value.",
        ),
        (
            "Competition is always more productive than cooperation in the workplace.",
            "Cooperation enables knowledge sharing and reduces duplicated effort.",
            "Healthy workplaces blend competition and collaboration depending on context.",
        ),
        (
            "Leaders should never admit uncertainty to those they lead.",
            "Acknowledging uncertainty can build trust and improve decision quality.",
            "Transparency about limits of knowledge differs from expressing doubt about goals.",
        ),
        (
            "The best way to help developing nations is through unrestricted foreign investment.",
            "Investment without safeguards can exploit labor and weaken local institutions.",
            "Aid and regulated investment may better align with long-term development.",
        ),
        (
            "History has little to teach us about solving contemporary problems.",
            "Historical patterns reveal recurring causes of conflict and reform.",
            "Past cases offer analogies and warnings even when contexts differ.",
        ),
        (
            "Public figures should be held to higher moral standards than private citizens.",
            "Public roles amplify the impact of misconduct, justifying higher scrutiny.",
            "Unrealistic standards may discourage qualified people from public service.",
        ),
        (
            "Censorship can be justified when it protects social harmony.",
            "Suppressing speech can silence dissent and entrench power.",
            "Harm-based limits differ from broad censorship for harmony.",
        ),
        (
            "The primary goal of education should be job preparation.",
            "Education also cultivates citizenship, ethics, and lifelong learning.",
            "Vocational and liberal aims need not be mutually exclusive.",
        ),
    ]

    extra_issue_prompts = [
        (
            "People should always follow tradition rather than seek innovation.",
            "Tradition alone cannot address novel problems that require new solutions.",
            "Tradition and innovation can coexist; rejecting all change is too rigid.",
        ),
        (
            "Remote work makes employees less committed to their organizations.",
            "Commitment depends on culture and management, not location alone.",
            "Remote arrangements can increase autonomy without reducing loyalty.",
        ),
        (
            "A nation's primary measure of progress should be economic growth alone.",
            "GDP growth ignores inequality, health, and environmental costs.",
            "Progress has social and ecological dimensions beyond output.",
        ),
        (
            "Children learn best when education emphasizes memorization over critical thinking.",
            "Critical thinking builds transferable skills that memorization alone cannot.",
            "Rote recall and reasoning both matter, but the claim overstates one side.",
        ),
        (
            "Privacy should always yield to national security concerns.",
            "Unchecked surveillance can erode civil liberties without improving safety.",
            "Security and privacy require balanced limits, not automatic tradeoffs.",
        ),
        (
            "Artificial intelligence will inevitably eliminate the need for human creativity.",
            "Human judgment, context, and originality remain essential in many domains.",
            "Tools augment creators; they do not replace all creative agency.",
        ),
        (
            "Charitable giving is less effective than government programs at solving social problems.",
            "Philanthropy and public programs can complement each other.",
            "Neither sector alone addresses every scale of need.",
        ),
        (
            "Travel abroad is necessary to develop a mature worldview.",
            "Perspective can grow through diverse local experiences and study, not travel alone.",
            "Travel helps some people but is neither necessary nor sufficient for maturity.",
        ),
        (
            "Democracies always produce better policy outcomes than other systems.",
            "Institutional quality and information matter as much as the label of democracy.",
            "Democratic processes vary widely in effectiveness.",
        ),
        (
            "People should prioritize career advancement over work-life balance.",
            "Burnout and health costs can undermine long-term career success.",
            "Balance and advancement are not mutually exclusive priorities.",
        ),
        (
            "Standardized testing is the fairest way to evaluate all students.",
            "Tests can reflect preparation gaps and bias rather than pure ability.",
            "Fair assessment may require multiple measures beyond one exam format.",
        ),
        (
            "Urban density always improves quality of life.",
            "Congestion, housing costs, and infrastructure strain can reduce livability.",
            "Density brings benefits and tradeoffs depending on planning and investment.",
        ),
        (
            "Scientific consensus should never be questioned by non-experts.",
            "Skeptical scrutiny and replication are part of science itself.",
            "Distinguish informed questioning from denial of overwhelming evidence.",
        ),
    ]

    more_issue_prompts = [
        (
            "Corporations should prioritize shareholder returns over employee welfare.",
            "Long-term value often depends on skilled, motivated workers, not short-term cuts.",
            "Shareholder and employee interests can align when firms invest in stability.",
        ),
        (
            "Social media does more harm than good to society.",
            "Social media also enables organizing, education, and marginalized voices.",
            "Net effects depend on design, moderation, and how platforms are used.",
        ),
        (
            "Free college tuition should be available to all students regardless of need.",
            "Universal tuition may subsidize families who could pay, reducing funds for need-based aid.",
            "Affordability goals can be met with targeted support rather than blanket subsidies.",
        ),
        (
            "Nations should open borders completely to improve global prosperity.",
            "Rapid open borders can strain housing, wages, and public services without transition plans.",
            "Managed migration and integration policy matter alongside openness.",
        ),
        (
            "Meritocracy is the fairest way to organize society.",
            "Starting advantages and bias can shape who is labeled meritorious.",
            "Fair systems require equal opportunity, not just competition on uneven footing.",
        ),
        (
            "Wealth inequality is an inevitable and acceptable feature of progress.",
            "Extreme inequality can undermine mobility, health, and democratic participation.",
            "Growth and equity are not always tradeoffs; policy choices matter.",
        ),
        (
            "Violent content in media directly causes violent behavior.",
            "Violence stems from many social factors; media is one influence among many.",
            "Correlation with exposure does not prove media alone causes aggression.",
        ),
        (
            "Professional athletes are overpaid relative to their social contribution.",
            "Market demand and scarce talent explain high pay in entertainment industries.",
            "Pay reflects revenue generation, not a moral ledger of social worth.",
        ),
        (
            "Governments should ban advertising directed at children.",
            "Parents and educators also shape consumption; bans alone may not change habits.",
            "Child-directed marketing raises concerns, but policy must weigh speech and enforcement.",
        ),
        (
            "Human nature is fundamentally selfish and cannot be changed.",
            "Cooperation and altruism appear across cultures and contexts.",
            "Behavior is shaped by institutions and incentives, not fixed traits alone.",
        ),
        (
            "Religious beliefs should guide public policy decisions.",
            "Plural societies require reasons accessible to citizens of many faiths.",
            "Secular law can protect religious freedom without enshrining one doctrine.",
        ),
        (
            "The best leaders are those who never change their minds.",
            "Updating beliefs in light of evidence is a strength, not weakness.",
            "Consistency matters, but rigidity can ignore new information.",
        ),
        (
            "Speed is more important than accuracy in decision-making.",
            "Hasty decisions can create costly errors that slow progress overall.",
            "Good decisions balance urgency with adequate analysis.",
        ),
        (
            "An ideal society requires everyone to pursue the same definition of success.",
            "Diverse talents and values enrich communities beyond one career template.",
            "Shared norms need not mean identical life paths.",
        ),
        (
            "Genetic testing should be mandatory before having children.",
            "Mandatory testing raises privacy, coercion, and disability-rights concerns.",
            "Informed choice differs from state-mandated reproductive screening.",
        ),
        (
            "Urban sprawl is preferable to concentrated city growth.",
            "Sprawl increases car dependence, infrastructure cost, and emissions.",
            "Compact development can improve access and environmental outcomes.",
        ),
        (
            "Every citizen should be required to perform national service.",
            "Mandatory service may conflict with individual liberty and opportunity costs.",
            "Voluntary service can meet civic goals without compulsion.",
        ),
        (
            "Technological unemployment makes retraining programs unnecessary.",
            "Displaced workers often need new skills to re-enter growing fields.",
            "Automation shifts demand; training helps workers adapt.",
        ),
        (
            "Museums and libraries are luxuries governments can eliminate first.",
            "Cultural institutions support literacy, research, and civic life.",
            "Public access to knowledge is infrastructure, not a frill.",
        ),
        (
            "Moral progress is impossible; human values never change.",
            "Legal and social norms have shifted on slavery, suffrage, and civil rights.",
            "Change is uneven, but values and practices do evolve over time.",
        ),
        (
            "Patriotism requires uncritical support of one's government.",
            "Loyalty to a country can include holding its policies to account.",
            "Dissent and reform are part of many patriotic traditions.",
        ),
        (
            "Corporate mergers always benefit consumers through efficiency.",
            "Consolidation can reduce competition and raise prices.",
            "Efficiency gains are not guaranteed; regulators weigh harms.",
        ),
        (
            "Bilingual education slows student integration and should be phased out.",
            "Bilingual programs can build literacy in two languages while students integrate.",
            "Language support and integration can proceed together.",
        ),
        (
            "Happiness is primarily determined by income level.",
            "Relationships, health, and purpose strongly affect well-being beyond income.",
            "Money helps up to a point but does not solely define happiness.",
        ),
        (
            "Criminal punishment should focus solely on rehabilitation, never deterrence.",
            "Deterrence and public safety are legitimate aims alongside rehabilitation.",
            "Effective justice systems balance multiple goals.",
        ),
    ]

    issue_prompts = issue_prompts + extra_issue_prompts + more_issue_prompts

    argument_prompts = [
        (
            "Our café's revenue rose 20% after we added vegan options, so vegan options caused the increase.",
            "Other factors such as seasonal traffic or marketing could explain the rise.",
            "Correlation after a menu change does not prove causation.",
        ),
        (
            "Since 90% of surveyed customers said they were satisfied, our service quality is excellent.",
            "Survey respondents may not represent all customers or may bias toward polite answers.",
            "Sample selection and response bias weaken the generalization.",
        ),
        (
            "The city should build a new stadium because it will create jobs.",
            "Temporary construction jobs may not offset public subsidies or opportunity costs.",
            "Job creation claims ignore duration, cost, and alternative investments.",
        ),
        (
            "Reading scores improved after we reduced class sizes, proving smaller classes always raise achievement.",
            "Other reforms may have occurred simultaneously; gains may not generalize.",
            "Single-district improvement does not establish universal causation.",
        ),
        (
            "We should adopt software X because our competitor uses it and remains profitable.",
            "The competitor's success may stem from many factors unrelated to software X.",
            "Competitor practice alone is not evidence of causal benefit.",
        ),
        (
            "The hospital infection rate fell after hand-sanitizer dispensers were installed, so dispensers alone eliminated infections.",
            "Concurrent hygiene campaigns or seasonal effects could contribute.",
            "Attributing all improvement to one intervention ignores confounds.",
        ),
        (
            "Local wildlife declined after a wind farm opened; therefore wind farms destroy ecosystems.",
            "Habitat loss, climate, or other development may explain wildlife trends.",
            "Temporal association without controlling variables is weak evidence.",
        ),
        (
            "Employee morale surveys show high satisfaction, so turnover will remain low.",
            "Satisfaction surveys may not predict behavior during economic shifts.",
            "Stated morale does not guarantee retention.",
        ),
        (
            "The town's crime rate dropped after more streetlights were added, so lighting is the best crime policy.",
            "Policing changes or demographic shifts may coincide with lighting upgrades.",
            "Multiple policies often change together.",
        ),
        (
            "Online course completion rates exceed 80%, proving online instruction is superior to in-person teaching.",
            "Completion is not the same as learning; populations and courses differ.",
            "Metric choice and selection effects undermine the comparison.",
        ),
        (
            "Our product's market share grew in regions where we advertised, so advertising alone drove growth.",
            "Pricing, distribution, or competitor errors may explain regional gains.",
            "Regional correlation does not isolate advertising effects.",
        ),
        (
            "Patient recovery times shortened after a new protocol was introduced, so the protocol should be mandatory nationwide.",
            "The pilot hospital may differ from others; results may not replicate.",
            "Single-site success requires broader validation.",
        ),
    ]

    extra_argument_prompts = [
        (
            "Complaints dropped after we extended store hours, so longer hours improved service quality.",
            "Fewer complaints may reflect fewer shoppers or changed reporting, not better service.",
            "The metric may track volume of feedback rather than quality.",
        ),
        (
            "Graduates from our program earn higher salaries, proving our curriculum is the best available.",
            "Self-selection and labor market conditions may explain earnings gaps.",
            "Outcome differences do not isolate curriculum effects.",
        ),
        (
            "Bike lane usage rose after installation, so every street should add lanes immediately.",
            "One corridor's results may not generalize to all street contexts.",
            "Infrastructure decisions need site-specific analysis.",
        ),
        (
            "Our newsletter open rate is high, so readers must agree with our editorial stance.",
            "Opening an email does not show agreement with its content.",
            "Engagement metrics differ from opinion metrics.",
        ),
        (
            "Defect rates fell after a new supplier was chosen, so the supplier caused the improvement.",
            "Process changes or inspection standards may have changed at the same time.",
            "Concurrent operational changes confound attribution.",
        ),
        (
            "Two executives who read the same book both increased revenue, so the book causes business success.",
            "A sample of two successes cannot establish a general causal rule.",
            "Anecdotes lack controls and sufficient size.",
        ),
        (
            "Wait times decreased after hiring more staff, so staffing alone fixed the bottleneck.",
            "Demand fluctuations or workflow changes could also explain shorter waits.",
            "Staffing is one variable among many in service systems.",
        ),
        (
            "Students who use our tutoring app score higher, so the app guarantees admission to top programs.",
            "Higher scores do not ensure admission outcomes.",
            "The argument jumps from test performance to admission certainty.",
        ),
        (
            "Energy use fell after thermostat setbacks, so aggressive setbacks should be mandated everywhere.",
            "Building types and climates differ; one policy may not fit all.",
            "Pilot savings do not justify universal mandates without analysis.",
        ),
        (
            "Social media mentions increased after the campaign, so brand loyalty must have improved.",
            "Mentions may reflect controversy or curiosity rather than loyalty.",
            "Visibility is not the same as positive attachment.",
        ),
        (
            "Returns fell after we added live chat support, so chat alone eliminated product dissatisfaction.",
            "Seasonal demand or product fixes may have changed at the same time.",
            "Support channels are one factor among many in satisfaction.",
        ),
        (
            "Volunteer sign-ups doubled after a celebrity endorsement, so celebrity endorsements always sustain nonprofits.",
            "Initial spikes may fade without ongoing engagement or mission fit.",
            "Short-term attention does not guarantee long-term volunteering.",
        ),
        (
            "Water quality scores improved after a filter was installed, so every household should use the same filter.",
            "Source water and plumbing conditions vary; one device may not fit all homes.",
            "Local conditions matter for infrastructure recommendations.",
        ),
    ]

    more_argument_prompts = [
        (
            "Website traffic doubled after the redesign, so the redesign caused the growth.",
            "Marketing spend or seasonality could also explain the traffic increase.",
            "Post hoc traffic gains do not isolate design effects.",
        ),
        (
            "Employee retention improved after flexible hours, so rigid schedules always harm retention.",
            "Other policy changes or labor market shifts may coincide with flexibility.",
            "One firm's experience does not prove a universal rule.",
        ),
        (
            "Town A lowered taxes and grew faster than Town B, so low taxes always drive growth.",
            "Industry mix, demographics, and investment differ between towns.",
            "Comparing two places ignores confounding local factors.",
        ),
        (
            "Patient satisfaction scores rose after free parking, so parking availability cures medical outcomes.",
            "Parking convenience may affect surveys without changing clinical care.",
            "Satisfaction scores measure experience, not treatment efficacy.",
        ),
        (
            "The team won every game after changing mascots, so the mascot change caused winning.",
            "Roster changes or schedule strength may explain the winning streak.",
            "Superstition and coincidence are weak causal evidence.",
        ),
        (
            "More people enrolled after tuition was cut, proving affordability alone drives quality education.",
            "Enrollment growth does not measure learning outcomes or completion.",
            "Quantity of students differs from quality of education.",
        ),
        (
            "Crime fell in districts with more cameras, so surveillance alone eliminates crime.",
            "Policing levels and reporting practices may change alongside cameras.",
            "Multiple safety policies often shift together.",
        ),
        (
            "Our product has five-star reviews online, so it must be the highest-quality option.",
            "Reviewers may be biased, incentivized, or unrepresentative of all buyers.",
            "Online ratings are not controlled comparisons.",
        ),
        (
            "Sales increased in stores that played classical music, so music choice drives all retail success.",
            "Store location, inventory, and promotions may differ from competitors.",
            "Ambient music is one variable in a complex retail environment.",
        ),
        (
            "Volunteer hours rose after recognition awards, so awards alone sustain long-term volunteering.",
            "Initial recognition may fade without meaningful roles or leadership.",
            "Motivation depends on more than one-time awards.",
        ),
        (
            "Defect complaints fell after a training video, so the video alone fixed manufacturing quality.",
            "Equipment upgrades or inspection changes may have occurred simultaneously.",
            "Training is one part of a quality system.",
        ),
        (
            "Public transit ridership rose after fare discounts, so fares are the only barrier to ridership.",
            "Service frequency and route coverage also affect ridership.",
            "Price is one factor in transportation choices.",
        ),
        (
            "The herb garden thrived after organic fertilizer was added, so organic fertilizer always beats other methods.",
            "Soil, weather, and watering may differ across gardens.",
            "One garden's success does not generalize to all conditions.",
        ),
        (
            "Customer churn fell after a loyalty program launch, so the program alone retained every at-risk customer.",
            "Competitor pricing or product improvements may have changed at the same time.",
            "Retention shifts rarely have a single cause.",
        ),
        (
            "Literacy rates improved after library hours extended, so longer hours alone solve literacy gaps.",
            "School programs and community outreach may have changed concurrently.",
            "Access to libraries helps but is not the only literacy intervention.",
        ),
        (
            "Energy bills dropped after insulation upgrades, so insulation alone explains every efficiency gain.",
            "Weather, usage habits, and rate changes also affect bills.",
            "Building efficiency has multiple drivers.",
        ),
        (
            "Our pilot clinic reduced readmissions, so the protocol should replace all care standards nationwide.",
            "Patient mix and staffing at the pilot may differ from other hospitals.",
            "Pilot success requires replication before universal adoption.",
        ),
        (
            "Applicants who submitted early were accepted more often, so earlier submission causes admission.",
            "Stronger applicants may simply apply earlier.",
            "Self-selection confounds timing and admission outcomes.",
        ),
        (
            "After the CEO appeared in ads, brand awareness rose, so executive visibility alone builds brand equity.",
            "Ad spend and product launches may have increased at the same time.",
            "Awareness campaigns bundle many tactics beyond one spokesperson.",
        ),
        (
            "Student attendance improved after free breakfast, so nutrition programs alone raise academic achievement.",
            "Other school reforms or family support may have changed simultaneously.",
            "Attendance gains do not prove learning gains.",
        ),
        (
            "Returns dropped after clearer sizing charts, so sizing information alone eliminates product dissatisfaction.",
            "Product quality or fit improvements may have occurred at the same time.",
            "Sizing clarity helps but does not address every return reason.",
        ),
        (
            "Fish populations recovered after a fishing ban, so bans alone restore every depleted fishery.",
            "Habitat restoration and climate conditions also affect recovery.",
            "Fishery management requires multiple interventions.",
        ),
        (
            "Productivity rose after standing desks were installed, so standing desks alone maximize worker output.",
            "Team changes or project mix may explain productivity shifts.",
            "Ergonomics is one factor in workplace performance.",
        ),
        (
            "Donations increased after a matching campaign, so matching gifts alone sustain nonprofit funding.",
            "Donor fatigue or economic conditions may change after campaigns end.",
            "Matching drives spikes but not guaranteed long-term revenue.",
        ),
        (
            "Noise complaints fell after sound barriers were built, so barriers alone solve all urban noise problems.",
            "Traffic patterns and zoning may also change over time.",
            "Noise reduction often needs multiple engineering and policy tools.",
        ),
    ]

    argument_prompts = argument_prompts + extra_argument_prompts + more_argument_prompts

    for i, (prompt, critique, expl) in enumerate(issue_prompts):
        out.append(
            q(
                f"gre-foundation-awa-issue-{i + 1:03d}",
                "gre::awa::issue",
                "awa",
                f'Issue prompt: "{prompt}" What is the strongest critique of this claim?',
                four_choices(
                    critique,
                    [
                        "The claim is obviously true in all cases.",
                        "Issues cannot be debated meaningfully.",
                        "Only experts may discuss this topic.",
                    ],
                ),
                critique,
                expl,
                fmt="essay_prompt",
                subtopic="analyze_issue",
                question_type="awa_issue",
                concepts=["awa", "issue_task", "critique"],
                difficulty=pick_diff(idx),
                time_secs=1800,
            )
        )
        idx += 1

    for i, (prompt, objection, expl) in enumerate(argument_prompts):
        out.append(
            q(
                f"gre-foundation-awa-arg-{i + 1:03d}",
                "gre::awa::argument",
                "awa",
                f'Argument prompt: "{prompt}" What is the strongest objection to this argument?',
                four_choices(
                    objection,
                    [
                        "Arguments cannot be evaluated logically.",
                        "The conclusion must be true regardless of evidence.",
                        "All surveys are perfectly representative.",
                    ],
                ),
                objection,
                expl,
                fmt="essay_prompt",
                subtopic="analyze_argument",
                question_type="awa_argument",
                concepts=["awa", "argument_task", "logical_flaw"],
                difficulty=pick_diff(idx),
                time_secs=1800,
            )
        )
        idx += 1

    assert len(out) >= MIN_FOUNDATION_AWA, len(out)
    return out


def validate_mcq_row(row: dict[str, Any]) -> None:
    if row.get("format") == "essay_prompt" or row.get("question_type", "").startswith(
        "awa_"
    ):
        return
    choices = row.get("answer_choices") or row.get("choices") or []
    if not choices:
        return
    correct = row["correct_answer"]
    normed = [norm_choice(choice) for choice in choices]
    assert len(set(normed)) == len(normed), (
        f"{row['id']}: duplicate choices {choices!r}"
    )
    matches = sum(1 for choice in choices if norm_choice(choice) == norm_choice(correct))
    assert matches == 1, (
        f"{row['id']}: expected one correct match, got {matches} for {correct!r} in {choices!r}"
    )


def validate_foundation_bank(
    quant: list[dict[str, Any]],
    verbal: list[dict[str, Any]],
    awa: list[dict[str, Any]],
) -> None:
    assert len(quant) >= MIN_FOUNDATION_QUANT, f"quant: {len(quant)}"
    assert len(verbal) >= MIN_FOUNDATION_VERBAL, f"verbal: {len(verbal)}"
    assert len(awa) >= MIN_FOUNDATION_AWA, f"awa: {len(awa)}"
    for section_name, rows in ("quant", quant), ("verbal", verbal), ("awa", awa):
        counts: dict[str, int] = {}
        for row in rows:
            counts[row["topic"]] = counts.get(row["topic"], 0) + 1
            validate_mcq_row(row)
        for topic, count in counts.items():
            assert count >= MIN_FOUNDATION_PER_TOPIC, (
                f"{section_name} {topic}: {count} < {MIN_FOUNDATION_PER_TOPIC}"
            )


def main() -> None:
    quant = gen_quant()
    verbal = gen_verbal()
    awa = gen_awa()
    validate_foundation_bank(quant, verbal, awa)
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    for name, data in [
        ("seed_gre_quant.json", quant),
        ("seed_gre_verbal.json", verbal),
        ("seed_gre_awa.json", awa),
    ]:
        path = OUT_DIR / name
        path.write_text(json.dumps(data, indent=4, ensure_ascii=False) + "\n")
        print(f"Wrote {path.name}: {len(data)} questions")
    print(f"Total: {len(quant) + len(verbal) + len(awa)}")


if __name__ == "__main__":
    main()
