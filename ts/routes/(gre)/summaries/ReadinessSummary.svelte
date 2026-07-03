<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type {
        DashboardCoverage,
        DashboardTopicInsight,
        MemoryScore,
        PerformanceScore,
        ReadinessCalibrationStats,
        ReadinessScore,
    } from "@generated/anki/brainlift_pb";

    import { buildReadinessExplainability } from "../prediction-explainability";
    import { coverageAwareReadinessUnlocked } from "../coverage-presentation";
    import {
        readinessConfidenceLabel,
        readinessDetailRows,
        readinessEvidence,
        readinessNextAction,
        readinessWhy,
    } from "../prediction-presentation";
    import type { MetricChangePresentation } from "../metric-change-presentation";
    import { formatPercent, formatRange } from "../score-format";
    import { readinessHero } from "../summary-metrics";
    import GrePredictionBrief from "../ui/GrePredictionBrief.svelte";

    export let readiness: ReadinessScore;
    export let memory: MemoryScore | undefined = undefined;
    export let performance: PerformanceScore | undefined = undefined;
    export let coverage: DashboardCoverage | undefined = undefined;
    export let weakTopics: DashboardTopicInsight[] = [];
    export let calibration: ReadinessCalibrationStats | undefined = undefined;
    export let variant: "card" | "compact" | "inline" = "card";
    export let metricChange: MetricChangePresentation | null = null;
    export let confidenceChange: MetricChangePresentation | null = null;

    $: explainability = buildReadinessExplainability({
        memory,
        performance,
        readiness,
        weakTopics,
        calibration,
    });
</script>

<GrePredictionBrief
    title="Readiness score"
    score={readinessHero(readiness, formatPercent)}
    scoreRange={formatRange(readiness.projectedScoreLow, readiness.projectedScoreHigh)}
    unlocked={coverageAwareReadinessUnlocked(readiness, coverage)}
    confidence={readinessConfidenceLabel(readiness)}
    why={readinessWhy()}
    evidence={readinessEvidence(readiness)}
    nextAction={readinessNextAction(readiness)}
    {explainability}
    detailRows={readinessDetailRows(readiness)}
    requirements={readiness.abstentionRequirements}
    {variant}
    expandLabel="Inspect evidence"
    {metricChange}
    {confidenceChange}
/>
