<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type {
        AbstentionRequirement,
        DashboardTopicInsight,
        EstimatedGreScore,
        MemoryScore,
        PerformanceScore,
        ReadinessCalibrationStats,
        ReadinessScore,
    } from "@generated/anki/brainlift_pb";

    import { buildEstimatedGreExplainability } from "../prediction-explainability";
    import {
        estimatedGreConfidenceLabel,
        estimatedGreDetailRows,
        estimatedGreEvidence,
        estimatedGreNextAction,
        estimatedGreUnlocked,
        estimatedGreWhy,
    } from "../prediction-presentation";
    import type { MetricChangePresentation } from "../metric-change-presentation";
    import { formatGreScoreRange } from "../score-format";
    import { estimatedGreHero } from "../summary-metrics";
    import GrePredictionBrief from "../ui/GrePredictionBrief.svelte";

    export let estimate: EstimatedGreScore;
    export let readiness: ReadinessScore | undefined = undefined;
    export let memory: MemoryScore | undefined = undefined;
    export let performance: PerformanceScore | undefined = undefined;
    export let weakTopics: DashboardTopicInsight[] = [];
    export let checklistRequirements: AbstentionRequirement[] = [];
    export let calibration: ReadinessCalibrationStats | undefined = undefined;
    export let variant: "card" | "compact" | "inline" = "card";
    export let metricChange: MetricChangePresentation | null = null;

    $: explainability = buildEstimatedGreExplainability({
        memory,
        performance,
        readiness,
        weakTopics,
        requirements: checklistRequirements,
        calibration,
    });
</script>

<GrePredictionBrief
    title="Estimated GRE"
    score={estimatedGreHero(estimate)}
    scoreRange={estimatedGreUnlocked(estimate)
        ? formatGreScoreRange(
            estimate.combinedScoreLow,
            estimate.combinedScoreHigh,
        )
        : null}
    unlocked={estimatedGreUnlocked(estimate)}
    showScoreHeader={estimatedGreUnlocked(estimate)}
    confidence={estimatedGreConfidenceLabel(estimate, readiness)}
    why={estimatedGreWhy(estimate)}
    evidence={estimatedGreEvidence(estimate, memory, performance)}
    nextAction={estimatedGreNextAction(estimate, checklistRequirements)}
    {explainability}
    detailRows={estimatedGreDetailRows(estimate)}
    requirements={checklistRequirements}
    {variant}
    expandLabel="Inspect evidence"
    {metricChange}
/>
