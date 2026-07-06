// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import {
    generateBrainLiftAiEvalReport,
    generateBrainLiftEvalReport,
    getDashboard,
    getGreAtlasVerification,
    getMemoryEval,
    getPerformanceEval,
    getReadinessCalibration,
    getRecentAttempts,
    getScores,
} from "@generated/backend";
import {
    BrainLiftAiEvalReportResponse,
    DashboardCoverage,
    GetScoresResponse,
    GreAtlasClientPlatform,
    MemoryScore,
    PerformanceEvalResponse,
    PerformanceScore,
    ReadinessCalibrationResponse,
    ReadinessScore,
} from "@generated/anki/brainlift_pb";

import type { PageLoad } from "./$types";

const silent = { alertOnError: false } as const;

export const load = (async () => {
    const [
        performanceEval,
        memoryEval,
        evalReport,
        aiEval,
        verification,
        scores,
        dashboard,
        readinessCalibration,
        recentAttempts,
    ] = await Promise.all([
        getPerformanceEval({}, silent).catch(
            () => new PerformanceEvalResponse(),
        ),
        getMemoryEval({}, silent).catch(() => null),
        generateBrainLiftEvalReport({}, silent).catch(() => null),
        generateBrainLiftAiEvalReport({}, silent).catch(
            () => new BrainLiftAiEvalReportResponse(),
        ),
        getGreAtlasVerification(
            { client: GreAtlasClientPlatform.DESKTOP },
            silent,
        ).catch(() => null),
        getScores({}, silent).catch(() => new GetScoresResponse()),
        getDashboard(
            {
                recentActivityLimit: 8,
                topicInsightLimit: 3,
            },
            silent,
        ).catch(() => null),
        getReadinessCalibration({}, silent).catch(
            () => new ReadinessCalibrationResponse(),
        ),
        getRecentAttempts({ limit: 100 }, silent).catch(() => ({ attempts: [] })),
    ]);

    return {
        performanceEval,
        memoryEval,
        evalReportJson: evalReport?.json ?? null,
        aiEval: aiEval ?? { aiEnabled: false, json: "", markdown: "" },
        verification,
        honestReporting: {
            memory: scores.memory ?? new MemoryScore(),
            performance: scores.performance ?? new PerformanceScore(),
            readiness: scores.readiness ?? new ReadinessScore(),
            coverage: dashboard?.coverage ?? new DashboardCoverage(),
            calibration: readinessCalibration.calibration,
            aiEnabled:
                verification?.aiEnabled ?? (aiEval?.aiEnabled ? "Enabled" : "Disabled"),
            recentAttempts: recentAttempts.attempts,
        },
    };
}) satisfies PageLoad;
