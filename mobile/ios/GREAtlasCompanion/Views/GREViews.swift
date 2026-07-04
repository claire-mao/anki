// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import SwiftUI

enum GreTheme {
    static let cardRadius: CGFloat = 12
    static let sectionSpacing: CGFloat = 12
    static let pagePadding: CGFloat = 12
    static let cardPadding: CGFloat = 12
    static let minTapTarget: CGFloat = 44
    static let scrollBottomInset: CGFloat = 8

    static func cardBackground() -> some View {
        RoundedRectangle(cornerRadius: cardRadius, style: .continuous)
            .fill(Color(.secondarySystemBackground))
            .shadow(color: .black.opacity(0.05), radius: 6, y: 2)
    }
}

private struct GreScrollContentMargins: ViewModifier {
    func body(content: Content) -> some View {
        if #available(iOS 17.0, *) {
            content
                .contentMargins(.bottom, GreTheme.scrollBottomInset, for: .scrollContent)
        } else {
            content.padding(.bottom, GreTheme.scrollBottomInset)
        }
    }
}

private extension View {
    func greScrollContentMargins() -> some View {
        modifier(GreScrollContentMargins())
    }

    func greMinTapTarget() -> some View {
        frame(minWidth: GreTheme.minTapTarget, minHeight: GreTheme.minTapTarget)
            .contentShape(Rectangle())
    }
}

struct GreMetricCard: View {
    let icon: String
    let title: String
    let value: String
    let detail: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Label(title, systemImage: icon)
                .font(.subheadline.weight(.semibold))
                .foregroundStyle(.secondary)
            Text(value)
                .font(.title2.weight(.bold))
                .foregroundStyle(.primary)
                .contentTransition(.numericText())
            if let detail, !detail.isEmpty {
                Text(detail)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(GreTheme.cardPadding)
        .background(GreTheme.cardBackground())
    }
}

struct GreLoadingShell: View {
    let label: String

    var body: some View {
        VStack(spacing: 16) {
            ProgressView()
                .controlSize(.large)
            Text(label)
                .font(.subheadline)
                .foregroundStyle(.secondary)
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 10) {
                ForEach(0..<4, id: \.self) { _ in
                    RoundedRectangle(cornerRadius: GreTheme.cardRadius, style: .continuous)
                        .fill(.quaternary)
                        .frame(height: 84)
                        .shimmering()
                }
            }
        }
        .frame(maxWidth: .infinity, minHeight: 160)
        .padding(.vertical, 4)
    }
}

struct GreErrorBanner: View {
    let message: String

    var body: some View {
        Label(message, systemImage: "exclamationmark.triangle.fill")
            .font(.footnote)
            .foregroundStyle(.red)
            .padding(12)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .fill(.red.opacity(0.08))
            )
    }
}

private struct ShimmerModifier: ViewModifier {
    @State private var phase: CGFloat = -1

    func body(content: Content) -> some View {
        content
            .overlay {
                LinearGradient(
                    colors: [.clear, .white.opacity(0.35), .clear],
                    startPoint: .leading,
                    endPoint: .trailing
                )
                .offset(x: phase * 200)
            }
            .clipped()
            .onAppear {
                withAnimation(.linear(duration: 1.2).repeatForever(autoreverses: false)) {
                    phase = 1
                }
            }
    }
}

private extension View {
    func shimmering() -> some View {
        modifier(ShimmerModifier())
    }
}

struct GrePageContent<Content: View>: View {
    let isLoading: Bool
    let error: String?
    let emptyMessage: String
    let hasData: Bool
    let loadingLabel: String
    @ViewBuilder let content: () -> Content

    private var contentAnimation: Animation {
        .easeInOut(duration: 0.18)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: GreTheme.sectionSpacing) {
            if isLoading && !hasData {
                GreLoadingShell(label: loadingLabel)
                    .transition(.opacity)
            } else if let error {
                GreErrorBanner(message: error)
            } else if !hasData && !isLoading {
                Text(emptyMessage)
                    .foregroundStyle(.secondary)
            }

            if hasData {
                content()
                    .transition(.opacity)
            }
        }
        .animation(contentAnimation, value: hasData)
        .animation(contentAnimation, value: isLoading)
    }
}

struct GreSectionPanel<Content: View>: View {
    let title: String
    let icon: String
    @ViewBuilder let content: () -> Content

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Label(title, systemImage: icon)
                .font(.headline)
            content()
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(GreTheme.cardPadding)
        .background(GreTheme.cardBackground())
    }
}

struct GreCoveragePanel: View {
    let coverage: GreCoverageView

    var body: some View {
        GreSectionPanel(title: "Coverage", icon: "chart.pie") {
            VStack(alignment: .leading, spacing: 10) {
                Text(ScoreFormat.formatRatio(coverage.weightedRatio))
                    .font(.title3.weight(.bold))
                Text(GreCoverageCopy.explanation)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                if !coverage.readinessAvailable, !coverage.readinessReason.isEmpty {
                    Text(coverage.readinessReason)
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                if !coverage.sections.isEmpty {
                    ForEach(coverage.sections, id: \.label) { section in
                        HStack {
                            Text(section.label)
                            Spacer()
                            Text("\(section.percent)%")
                                .foregroundStyle(.secondary)
                        }
                        .font(.subheadline)
                    }
                }
                if !coverage.uncoveredStudyLabels.isEmpty {
                    Text("Suggested next topics")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)
                    ForEach(coverage.uncoveredStudyLabels, id: \.self) { label in
                        Text(label)
                            .font(.footnote)
                    }
                }
            }
        }
    }
}

struct GreTopicInsightRow: View {
    let topic: GreTopicInsightView

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(topic.displayName)
                .font(.subheadline.weight(.semibold))
            HStack(spacing: 8) {
                if let memory = topic.memoryScore {
                    Text("Memory \(ScoreFormat.formatPercent(memory))")
                }
                if let accuracy = topic.practiceAccuracy {
                    Text("Practice \(ScoreFormat.formatPercent(accuracy))")
                }
                if !topic.studyLabel.isEmpty {
                    Text(topic.studyLabel)
                }
            }
            .font(.caption)
            .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

struct GreActivityRow: View {
    let attempt: GreAttemptView

    var body: some View {
        HStack {
            Text(attempt.topic)
                .font(.subheadline.weight(.medium))
            Spacer()
            Text(attempt.correct ? "Correct" : "Incorrect")
                .font(.caption.weight(.semibold))
                .foregroundStyle(attempt.correct ? .green : .red)
        }
    }
}

struct GreDailyTaskRow: View {
    let task: GreDailyTaskView
    var dueTotal: UInt = 0
    var showAction = false
    var onAction: (() -> Void)?

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(GreMissionCopy.title(for: task))
                .font(.subheadline.weight(.semibold))
            Text(GreMissionCopy.description(for: task))
                .font(.footnote)
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)
            HStack {
                Text(GreMissionCopy.progressLabel(for: task, dueTotal: dueTotal))
                    .font(.caption.weight(.semibold))
                if let detail = GreMissionCopy.progressDetail(for: task, dueTotal: dueTotal) {
                    Text("· \(detail)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            if showAction, let onAction {
                Button(GreMissionCopy.actionLabel(for: task), action: onAction)
                    .buttonStyle(.borderedProminent)
                    .frame(maxWidth: .infinity, minHeight: GreTheme.minTapTarget)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

struct GreTrendSummary: View {
    let trend: [Double]

    var body: some View {
        if trend.count >= 2 {
            Text("Recent accuracy · \(trend.map { ScoreFormat.formatPercent($0) }.joined(separator: " → "))")
                .font(.footnote)
                .foregroundStyle(.secondary)
        }
    }
}

/// Compact row for settings panels — avoids card-in-card nesting.
private struct GreInlineMetricRow: View {
    let label: String
    let value: String
    var detail: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(alignment: .firstTextBaseline) {
                Text(label)
                    .font(.subheadline.weight(.medium))
                Spacer(minLength: 8)
                Text(value)
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.trailing)
            }
            if let detail, !detail.isEmpty {
                Text(detail)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

private let practiceFilterColumns = [
    GridItem(.flexible(), spacing: 8),
    GridItem(.flexible(), spacing: 8),
]

/// Dashboard focused on four questions: predicted score, reliability, what to
/// study today, and whether the student is improving. Everything else is behind
/// an info sheet, expandable sections, and a details panel.
struct DashboardView: View {
    @EnvironmentObject private var engine: AnkiMobileEngine
    @EnvironmentObject private var tabRouter: GreTabRouter
    @Environment(\.dynamicTypeSize) private var dynamicTypeSize
    @State private var showReliabilityInfo = false
    @State private var showStudyPlan = false

    private var predictionScoreSize: CGFloat {
        dynamicTypeSize.isAccessibilitySize ? 32 : 36
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                GrePageContent(
                    isLoading: engine.dashboardLoading,
                    error: engine.dashboardError,
                    emptyMessage: "Pull down to load your dashboard.",
                    hasData: engine.dashboard != nil,
                    loadingLabel: "Loading dashboard…"
                ) {
                    if let view = engine.dashboard {
                        VStack(alignment: .leading, spacing: GreTheme.sectionSpacing) {
                            studyTodayCard(view)
                            predictionCard(view)
                            DisclosureGroup {
                                improvingCard(view)
                            } label: {
                                Label("Am I improving?", systemImage: "chart.line.uptrend.xyaxis")
                                    .font(.headline)
                            }
                            evidenceDetails(view)
                        }
                    }
                }
                .padding(.horizontal, GreTheme.pagePadding)
                .padding(.vertical, 8)
            }
            .greScrollContentMargins()
            .navigationTitle("Dashboard")
            .refreshable { await engine.refreshDashboard() }
            .task {
                if engine.dashboard == nil && !engine.dashboardLoading {
                    await engine.refreshDashboard()
                }
            }
            .onChange(of: tabRouter.openStudyPlanOnDashboard) { pending in
                if pending {
                    showStudyPlan = true
                    tabRouter.openStudyPlanOnDashboard = false
                }
            }
            .sheet(isPresented: $showReliabilityInfo) {
                reliabilityInfoSheet(engine.dashboard)
            }
            .sheet(isPresented: $showStudyPlan) {
                if let view = engine.dashboard {
                    StudyPlanSheet(view: view) { task in
                        showStudyPlan = false
                        handleDailyTask(task, for: view)
                    }
                }
            }
        }
    }

    private func handleDailyTask(_ task: GreDailyTaskView, for view: GreDashboardView) {
        switch task.id {
        case "review_cards":
            if task.targetCount > 0 {
                tabRouter.open(.study)
            } else {
                showStudyPlan = true
            }
        case "practice_questions":
            tabRouter.openPractice()
        default:
            tabRouter.openPractice(
                topicId: task.topicId,
                topicTitle: task.topicDisplayName ?? GreMissionCopy.title(for: task)
            )
        }
    }

    // MARK: Q1 + Q2 — predicted score and how reliable it is

    @ViewBuilder
    private func predictionCard(_ view: GreDashboardView) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(alignment: .top, spacing: 12) {
                estimatedGreMetric(view)
                Divider()
                readinessMetric(view)
            }

            HStack(alignment: .center) {
                Spacer(minLength: 8)
                Button {
                    showReliabilityInfo = true
                } label: {
                    Label("How reliable is this?", systemImage: "info.circle")
                        .font(.footnote.weight(.semibold))
                        .foregroundStyle(.secondary)
                }
                .buttonStyle(.plain)
                .greMinTapTarget()
                .accessibilityLabel("How this estimate and its reliability are calculated")
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(GreTheme.cardPadding)
        .background(GreTheme.cardBackground())
    }

    @ViewBuilder
    private func estimatedGreMetric(_ view: GreDashboardView) -> some View {
        let available = estimatedGreAvailable(view)
        VStack(alignment: .leading, spacing: 8) {
            Label("Estimated GRE", systemImage: "number")
                .font(.subheadline.weight(.semibold))
                .foregroundStyle(.secondary)
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(predictedScore(view))
                    .font(.system(size: predictionScoreSize, weight: .bold, design: .rounded))
                    .contentTransition(.numericText())
                if available && view.estimatedGrePreliminary {
                    Text("Preliminary")
                        .font(.caption2.weight(.semibold))
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(Capsule().fill(Color.secondary.opacity(0.15)))
                        .foregroundStyle(.secondary)
                }
            }
            if available,
               let range = ScoreFormat.formatGreScoreRange(low: view.estimatedGreLow, high: view.estimatedGreHigh) {
                Text(range)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
            Text(
                available
                    ? "Your projected GRE score (260–340)."
                    : (view.readinessSummary.isEmpty ? "Not enough evidence for a GRE score yet." : view.readinessSummary)
            )
                .font(.caption)
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    @ViewBuilder
    private func readinessMetric(_ view: GreDashboardView) -> some View {
        let unlocked = view.coverage.readinessAvailable && view.readinessSufficient
        VStack(alignment: .leading, spacing: 8) {
            Label("Readiness", systemImage: "flag.checkered")
                .font(.subheadline.weight(.semibold))
                .foregroundStyle(.secondary)
            Text(
                unlocked
                    ? ScoreFormat.formatPercent(view.readinessProjected ?? 0)
                    : (view.readinessSummary.isEmpty ? "Not enough evidence yet" : view.readinessSummary)
            )
            .font(.system(size: predictionScoreSize, weight: .bold, design: .rounded))
            .contentTransition(.numericText())
            if unlocked, let range = ScoreFormat.formatRange(low: view.readinessLow, high: view.readinessHigh) {
                Text(range)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
            Text("How much your study evidence supports that score.")
                .font(.caption)
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private func estimatedGreAvailable(_ view: GreDashboardView) -> Bool {
        view.coverage.readinessAvailable
            && view.readinessSufficient
            && view.estimatedGreCombined != nil
    }

    private func predictedScore(_ view: GreDashboardView) -> String {
        guard estimatedGreAvailable(view), let combined = view.estimatedGreCombined else { return "—" }
        return ScoreFormat.formatGreScore(combined)
    }

    private func reliabilitySummary(_ view: GreDashboardView) -> String {
        guard view.readinessSufficient, let projected = view.readinessProjected else {
            return "Not enough evidence yet"
        }
        let base = ScoreFormat.formatPercent(projected)
        if !view.readinessConfidenceLevel.isEmpty {
            return "\(base) · \(view.readinessConfidenceLevel.capitalized) confidence"
        }
        return base
    }

    // MARK: Today's study plan

    @ViewBuilder
    private func studyTodayCard(_ view: GreDashboardView) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(alignment: .center) {
                Label("Today's study plan", systemImage: "calendar")
                    .font(.headline)
                Spacer(minLength: 8)
                Button("Open study plan") {
                    showStudyPlan = true
                }
                .buttonStyle(.borderedProminent)
                .frame(minHeight: GreTheme.minTapTarget)
            }

            if view.dailyPlanTaskCount > 0 {
                Text(GreMissionCopy.intro(taskCount: view.dailyPlanTaskCount))
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            } else if !view.dailyPlanHeadline.isEmpty {
                Text(view.dailyPlanHeadline)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }

            ForEach(view.dailyPlanTasks.prefix(3), id: \.id) { task in
                GreDailyTaskRow(
                    task: task,
                    dueTotal: view.dueTotal,
                    showAction: true
                ) {
                    handleDailyTask(task, for: view)
                }
                .padding(12)
                .background(
                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                        .fill(Color.accentColor.opacity(0.10))
                )
            }

            if view.deckExists && view.dueTotal > 0 {
                Text("\(view.dueTotal) cards due for review")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }

            if !view.dailyPlanRationale.isEmpty || view.weakTopic != nil || !view.recommendedTopics.isEmpty {
                DisclosureGroup("See full plan") {
                    VStack(alignment: .leading, spacing: 12) {
                        if !view.dailyPlanRationale.isEmpty {
                            Text(view.dailyPlanRationale)
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                                .fixedSize(horizontal: false, vertical: true)
                        }
                        if let weakTopic = view.weakTopic {
                            Text("Weakest topic")
                                .font(.caption.weight(.semibold))
                                .foregroundStyle(.secondary)
                            GreTopicInsightRow(topic: weakTopic)
                        }
                        if !view.recommendedTopics.isEmpty {
                            Text("Recommended focus")
                                .font(.caption.weight(.semibold))
                                .foregroundStyle(.secondary)
                            ForEach(view.recommendedTopics, id: \.topicId) { topic in
                                GreTopicInsightRow(topic: topic)
                            }
                        }
                    }
                    .padding(.top, 6)
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .font(.subheadline.weight(.semibold))
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(GreTheme.cardPadding)
        .background(GreTheme.cardBackground())
    }

    // MARK: Q4 — am I improving

    @ViewBuilder
    private func improvingCard(_ view: GreDashboardView) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            if view.recentAccuracyTrend.count >= 2 {
                Text("Recent accuracy \(view.recentAccuracyTrend.map { ScoreFormat.formatPercent($0) }.joined(separator: " → "))")
                    .font(.subheadline)
                    .fixedSize(horizontal: false, vertical: true)
            } else {
                Text("Keep studying and practicing — trends appear after your next sessions.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }

            if !view.recentActivity.isEmpty {
                DisclosureGroup("Recent activity") {
                    VStack(alignment: .leading, spacing: 8) {
                        ForEach(Array(view.recentActivity.enumerated()), id: \.offset) { _, attempt in
                            GreActivityRow(attempt: attempt)
                        }
                    }
                    .padding(.top, 6)
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .font(.subheadline.weight(.semibold))
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(GreTheme.cardPadding)
        .background(GreTheme.cardBackground())
    }

    // MARK: Progressive disclosure — full evidence and metrics

    @ViewBuilder
    private func evidenceDetails(_ view: GreDashboardView) -> some View {
        DisclosureGroup {
            VStack(spacing: 12) {
                GreMetricCard(
                    icon: "brain.head.profile",
                    title: "Memory",
                    value: ScoreFormat.scoreSummary(
                        value: view.memoryValue,
                        low: view.memoryLow,
                        high: view.memoryHigh,
                        sufficient: view.memorySufficient,
                        abstainReason: view.memoryDetail
                    ),
                    detail: "\(view.memoryStudiedCards) studied cards"
                )
                GreMetricCard(
                    icon: "target",
                    title: "Performance",
                    value: ScoreFormat.scoreSummary(
                        value: view.performanceValue,
                        low: view.performanceLow,
                        high: view.performanceHigh,
                        sufficient: view.performanceSufficient,
                        abstainReason: view.performanceDetail
                    ),
                    detail: "\(view.performanceAttemptCount) attempts"
                )
                GreCoveragePanel(coverage: view.coverage)
                GreMetricCard(
                    icon: view.deckExists ? "rectangle.stack" : "rectangle.stack.badge.plus",
                    title: view.deckExists ? view.deckName : "Study deck",
                    value: view.deckExists
                        ? "\(view.dueTotal) due · \(view.dueNew) new · \(view.dueLearn) learning · \(view.dueReview) review"
                        : "Deck not found",
                    detail: view.deckExists
                        ? nil
                        : "Built-in flashcards load when you open Study."
                )
            }
            .padding(.top, 12)
        } label: {
            Label("Evidence & metrics", systemImage: "chart.bar.doc.horizontal")
                .font(.headline)
        }
    }

    // MARK: ⓘ reliability detail sheet

    @ViewBuilder
    private func reliabilityInfoSheet(_ view: GreDashboardView?) -> some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    Text("Your Estimated GRE combines three signals from your own study data. Reliability reflects how much evidence supports it.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .fixedSize(horizontal: false, vertical: true)

                    if let view {
                        infoRow(
                            title: "Reliability",
                            value: reliabilitySummary(view),
                            detail: view.readinessSummary.isEmpty ? nil : view.readinessSummary
                        )
                        if !view.coverage.readinessAvailable && !view.coverage.readinessReason.isEmpty {
                            infoRow(title: "Why it's limited", value: view.coverage.readinessReason, detail: nil)
                        }
                        infoRow(
                            title: "Memory",
                            value: ScoreFormat.scoreSummary(
                                value: view.memoryValue,
                                low: view.memoryLow,
                                high: view.memoryHigh,
                                sufficient: view.memorySufficient,
                                abstainReason: view.memoryDetail
                            ),
                            detail: "\(view.memoryStudiedCards) studied cards"
                        )
                        infoRow(
                            title: "Performance",
                            value: ScoreFormat.scoreSummary(
                                value: view.performanceValue,
                                low: view.performanceLow,
                                high: view.performanceHigh,
                                sufficient: view.performanceSufficient,
                                abstainReason: view.performanceDetail
                            ),
                            detail: "\(view.performanceAttemptCount) attempts"
                        )
                        infoRow(
                            title: "Coverage",
                            value: ScoreFormat.formatRatio(view.coverage.weightedRatio),
                            detail: "\(view.coverage.coveredLeafCount)/\(view.coverage.catalogLeafCount) topics"
                        )
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(GreTheme.pagePadding)
            }
            .greScrollContentMargins()
            .navigationTitle("How reliable is this?")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") { showReliabilityInfo = false }
                }
            }
        }
        .presentationDetents([.medium, .large])
    }

    private func infoRow(title: String, value: String, detail: String?) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)
            Text(value)
                .font(.subheadline.weight(.semibold))
                .fixedSize(horizontal: false, vertical: true)
            if let detail, !detail.isEmpty {
                Text(detail)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(12)
        .background(GreTheme.cardBackground())
        .accessibilityElement(children: .combine)
    }
}

struct GreProgressScreen: View {
    @EnvironmentObject private var engine: AnkiMobileEngine

    private let columns = [GridItem(.flexible())]

    var body: some View {
        NavigationStack {
            ScrollView {
                GrePageContent(
                    isLoading: engine.progressLoading,
                    error: engine.progressError,
                    emptyMessage: "Pull to refresh once your collection has GRE study data.",
                    hasData: engine.progress != nil,
                    loadingLabel: "Loading progress…"
                ) {
                    if let view = engine.progress {
                        VStack(alignment: .leading, spacing: GreTheme.sectionSpacing) {
                            LazyVGrid(columns: columns, spacing: 10) {
                                GreMetricCard(
                                    icon: "brain.head.profile",
                                    title: "Memory",
                                    value: ScoreFormat.scoreSummary(
                                        value: view.memoryValue,
                                        low: view.memoryLow,
                                        high: view.memoryHigh,
                                        sufficient: view.memorySufficient,
                                        abstainReason: view.memoryDetail
                                    ),
                                    detail: view.memoryDetail.isEmpty
                                        ? ScoreFormat.formatRange(low: view.memoryLow, high: view.memoryHigh)
                                        : view.memoryDetail
                                )
                                GreMetricCard(
                                    icon: "target",
                                    title: "Performance",
                                    value: ScoreFormat.scoreSummary(
                                        value: view.performanceValue,
                                        low: view.performanceLow,
                                        high: view.performanceHigh,
                                        sufficient: view.performanceSufficient,
                                        abstainReason: view.performanceDetail
                                    ),
                                    detail: "\(view.performanceAttemptCount) attempts"
                                )
                                GreMetricCard(
                                    icon: "flag.checkered",
                                    title: "Readiness",
                                    value: ScoreFormat.scoreSummary(
                                        value: view.readinessProjected,
                                        low: view.readinessLow,
                                        high: view.readinessHigh,
                                        sufficient: view.readinessSufficient,
                                        abstainReason: view.readinessSummary
                                    ),
                                    detail: view.readinessSummary.isEmpty
                                        ? view.readinessConfidenceLevel
                                        : view.readinessSummary
                                )
                                GreMetricCard(
                                    icon: "number",
                                    title: "Estimated GRE",
                                    value: ScoreFormat.estimatedGreSummary(
                                        combined: view.estimatedGreCombined,
                                        low: view.estimatedGreLow,
                                        high: view.estimatedGreHigh,
                                        preliminary: view.estimatedGrePreliminary,
                                        fallback: "Estimate unavailable"
                                    ),
                                    detail: view.estimatedGreConfidence.isEmpty
                                        ? ScoreFormat.formatGreScoreRange(
                                            low: view.estimatedGreLow,
                                            high: view.estimatedGreHigh
                                        )
                                        : view.estimatedGreConfidence
                                )
                                GreMetricCard(
                                    icon: "chart.pie",
                                    title: "Coverage",
                                    value: ScoreFormat.formatRatio(view.weightedCoverage),
                                    detail: "\(view.coveredLeafCount)/\(view.catalogLeafCount) topics · \(view.studiedCards) studied cards"
                                )
                            }

                            DisclosureGroup {
                                GreCoveragePanel(coverage: view.coverage)
                                    .padding(.top, 8)
                            } label: {
                                Label("Coverage details", systemImage: "chart.pie")
                                    .font(.subheadline.weight(.semibold))
                            }

                            if !view.topicMastery.isEmpty {
                                DisclosureGroup {
                                    VStack(alignment: .leading, spacing: 8) {
                                        Text("\(view.topicCount) topics · \(view.masteredCards) mastered cards")
                                            .font(.footnote)
                                            .foregroundStyle(.secondary)
                                        ForEach(view.topicMastery.prefix(8), id: \.topicId) { topic in
                                            HStack {
                                                Text(topic.displayName)
                                                Spacer(minLength: 8)
                                                Text(ScoreFormat.formatRatio(topic.avgRetrievability))
                                                    .foregroundStyle(.secondary)
                                            }
                                            .font(.subheadline)
                                        }
                                    }
                                    .padding(.top, 8)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                } label: {
                                    Label("Topic mastery", systemImage: "chart.bar")
                                        .font(.subheadline.weight(.semibold))
                                }
                            }

                            if !view.weakTopics.isEmpty {
                                DisclosureGroup {
                                    VStack(alignment: .leading, spacing: 8) {
                                        ForEach(view.weakTopics, id: \.topicId) { topic in
                                            GreTopicInsightRow(topic: topic)
                                        }
                                    }
                                    .padding(.top, 8)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                } label: {
                                    Label("Weak topics", systemImage: "exclamationmark.circle")
                                        .font(.subheadline.weight(.semibold))
                                }
                            }

                            if !view.recentActivity.isEmpty {
                                DisclosureGroup {
                                    VStack(alignment: .leading, spacing: 8) {
                                        GreTrendSummary(trend: view.practiceTrend)
                                        ForEach(Array(view.recentActivity.enumerated()), id: \.offset) { _, attempt in
                                            GreActivityRow(attempt: attempt)
                                        }
                                    }
                                    .padding(.top, 8)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                } label: {
                                    Label("Recent practice", systemImage: "clock.arrow.circlepath")
                                        .font(.subheadline.weight(.semibold))
                                }
                            }
                        }
                    }
                }
                .padding(.horizontal, GreTheme.pagePadding)
                .padding(.vertical, 8)
            }
            .greScrollContentMargins()
            .navigationTitle("Progress")
            .refreshable { await engine.refreshProgress() }
            .task {
                if engine.progress == nil && !engine.progressLoading {
                    await engine.refreshProgress()
                }
            }
        }
    }
}

struct PracticeView: View {
    @EnvironmentObject private var engine: AnkiMobileEngine
    @EnvironmentObject private var tabRouter: GreTabRouter
    @StateObject private var session = PracticeSession()

    private var practiceNavigationTitle: String {
        if !session.topicTitle.isEmpty {
            return session.topicTitle
        }
        if !session.topicFilter.isEmpty {
            return GreMissionCopy.practiceTopicTitle(for: session.topicFilter)
        }
        return "Practice"
    }

    var body: some View {
        NavigationStack {
            Group {
                if let bootstrap = engine.practice,
                   !session.sessionComplete,
                   session.currentQuestion != nil {
                    practiceFocusedLayout(bootstrap)
                } else {
                    practiceScrollLayout
                }
            }
            .navigationTitle(practiceNavigationTitle)
            .refreshable {
                await engine.refreshPractice()
                if let bootstrap = engine.practice {
                    session.restart(from: bootstrap)
                    applyPendingPracticeTopic(from: bootstrap)
                }
            }
            .task {
                if engine.practice == nil && !engine.practiceLoading {
                    await engine.refreshPractice()
                }
                if let bootstrap = engine.practice {
                    session.syncBootstrap(bootstrap)
                    applyPendingPracticeTopic(from: bootstrap)
                }
            }
            .onChange(of: tabRouter.pendingPracticeTopicId) { _ in
                guard let bootstrap = engine.practice else { return }
                applyPendingPracticeTopic(from: bootstrap)
            }
        }
    }

    private func applyPendingPracticeTopic(from bootstrap: GrePracticeBootstrapView) {
        guard let topicId = tabRouter.pendingPracticeTopicId, !topicId.isEmpty else { return }
        session.applyTopicFocus(
            topicId: topicId,
            topicTitle: tabRouter.pendingPracticeTopicTitle,
            from: bootstrap
        )
        tabRouter.pendingPracticeTopicId = nil
        tabRouter.pendingPracticeTopicTitle = nil
    }

    @ViewBuilder
    private var practiceScrollLayout: some View {
        ScrollView {
            GrePageContent(
                isLoading: engine.practiceLoading && engine.practice == nil,
                error: engine.practiceError,
                emptyMessage: "Pull to refresh to start a practice session from the Rust backend.",
                hasData: engine.practice != nil,
                loadingLabel: "Preparing practice session…"
            ) {
                if let bootstrap = engine.practice {
                    VStack(alignment: .leading, spacing: GreTheme.sectionSpacing) {
                        practiceToolbar

                        if session.sessionComplete {
                            practiceCompletePanel
                        } else if session.currentQuestion == nil {
                            Text("No questions here. Try another section filter.")
                                .foregroundStyle(.secondary)
                        }
                    }
                    .onAppear { session.syncBootstrap(bootstrap) }
                    .onChange(of: bootstrap.sessionId) { _ in session.syncBootstrap(bootstrap) }
                }
            }
            .padding(.horizontal, GreTheme.pagePadding)
            .padding(.vertical, 8)
        }
        .greScrollContentMargins()
    }

    @ViewBuilder
    private func practiceFocusedLayout(_ bootstrap: GrePracticeBootstrapView) -> some View {
        VStack(spacing: 0) {
            practiceToolbar
                .padding(.horizontal, GreTheme.pagePadding)
                .padding(.vertical, 8)
                .background(.bar)

            ScrollView {
                if let question = session.currentQuestion {
                    practiceQuestionContent(question)
                        .id("\(session.questionIndex)-\(question.id)")
                        .padding(.horizontal, GreTheme.pagePadding)
                        .padding(.vertical, 8)
                }
            }
            .greScrollContentMargins()
        }
        .onAppear { session.syncBootstrap(bootstrap) }
        .onChange(of: bootstrap.sessionId) { _ in session.syncBootstrap(bootstrap) }
    }

    @ViewBuilder
    private var practiceToolbar: some View {
        VStack(alignment: .leading, spacing: 10) {
            if !session.queue.isEmpty {
                VStack(alignment: .leading, spacing: 6) {
                    HStack {
                        Text(session.progressLabel)
                            .font(.subheadline.weight(.semibold))
                        Spacer()
                        Text("\(session.progressPercent)%")
                            .font(.caption.weight(.semibold))
                            .foregroundStyle(.secondary)
                    }
                    ProgressView(value: Double(session.progressPercent), total: 100)
                        .tint(.accentColor)
                }
            }

            LazyVGrid(columns: practiceFilterColumns, spacing: 8) {
                if session.topicFilter.isEmpty {
                    ForEach(PracticeSectionFilter.allCases) { filter in
                        Button(filter.label) {
                            if let bootstrap = engine.practice {
                                session.applySectionFilter(filter, from: bootstrap)
                            }
                        }
                        .buttonStyle(.bordered)
                        .tint(session.sectionFilter == filter ? .accentColor : .secondary)
                        .frame(maxWidth: .infinity)
                        .frame(minHeight: GreTheme.minTapTarget)
                    }
                }
            }

            if !session.sessionComplete, let scoreStrip = session.scoreStrip {
                PracticeScoreStripView(scoreStrip: scoreStrip)
            }
        }
    }

    @ViewBuilder
    private var practiceCompletePanel: some View {
        if session.attemptsRecorded == 0 {
            GreSectionPanel(title: "Session complete", icon: "checkmark.circle") {
                Text("No questions here. Try another section filter.")
                    .foregroundStyle(.secondary)
                Button("Show all sections") {
                    if let bootstrap = engine.practice {
                        session.applySectionFilter(.all, from: bootstrap)
                    }
                }
                .buttonStyle(.borderedProminent)
                .frame(maxWidth: .infinity, minHeight: GreTheme.minTapTarget)
            }
        } else {
            let summary = SessionCompletionBuilder.practiceSummary(from: session.sessionAttempts)
            SessionCompletePanel(
                summary: summary,
                onPrimary: {
                    handleSessionCompletionPrimary(summary)
                },
                onSecondary: {
                    handleSessionCompletionSecondary(summary, restartPractice: {
                        if let bootstrap = engine.practice {
                            session.applySectionFilter(session.sectionFilter, from: bootstrap)
                        }
                    })
                }
            )
            .padding(GreTheme.cardPadding)
            .background(GreTheme.cardBackground())
        }
    }

    private func handleSessionCompletionPrimary(_ summary: SessionCompletionSummary) {
        if summary.nextActionLabel == "View study plan" {
            tabRouter.openStudyPlan()
            return
        }
        if let tab = summary.nextActionTab {
            tabRouter.open(tab)
        }
    }

    private func handleSessionCompletionSecondary(
        _ summary: SessionCompletionSummary,
        restartPractice: @escaping () -> Void
    ) {
        if summary.secondaryActionLabel == "Practice again" {
            restartPractice()
            return
        }
        tabRouter.openStudyPlan()
    }

    @ViewBuilder
    private func practiceQuestionContent(_ question: GreQuestionView) -> some View {
        VStack(alignment: .leading, spacing: GreTheme.sectionSpacing) {
            VStack(alignment: .leading, spacing: 4) {
                Text(GreMissionCopy.practiceTopicTitle(for: question.topic))
                    .font(.subheadline.weight(.semibold))
                Text(GreMissionCopy.questionTypeLabel(for: question.format))
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }

            Text(question.stem)
                .font(.title3)
                .frame(maxWidth: .infinity, alignment: .leading)
                .fixedSize(horizontal: false, vertical: true)

            if let result = session.attemptResult {
                VStack(alignment: .leading, spacing: 8) {
                    Text(
                        "\(result.correct ? "✓ Correct" : "✗ Incorrect") · \(ScoreFormat.formatResponseTimeMs(session.responseTimeMs))"
                    )
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(result.correct ? .green : .red)

                    Text(result.explanation)
                        .font(.body)
                        .fixedSize(horizontal: false, vertical: true)

                    Text("\(question.section) · \(question.format) · \(result.topic)")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                .padding(GreTheme.cardPadding)
                .background(
                    RoundedRectangle(cornerRadius: GreTheme.cardRadius, style: .continuous)
                        .fill(result.correct ? Color.green.opacity(0.08) : Color.red.opacity(0.08))
                )

                Button(session.questionIndex + 1 >= session.queue.count ? "Finish session" : "Next question") {
                    session.nextQuestion()
                }
                .buttonStyle(.borderedProminent)
                .frame(maxWidth: .infinity, minHeight: GreTheme.minTapTarget)
            } else {
                VStack(alignment: .leading, spacing: 8) {
                    ForEach(question.choices, id: \.self) { choice in
                        Button {
                            session.selectedAnswer = choice
                        } label: {
                            HStack(alignment: .top, spacing: 10) {
                                Image(systemName: session.selectedAnswer == choice ? "largecircle.fill.circle" : "circle")
                                    .padding(.top, 2)
                                Text(choice)
                                    .multilineTextAlignment(.leading)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                            .padding(GreTheme.cardPadding)
                            .frame(maxWidth: .infinity, minHeight: GreTheme.minTapTarget, alignment: .leading)
                            .background(
                                RoundedRectangle(cornerRadius: GreTheme.cardRadius, style: .continuous)
                                    .fill(session.selectedAnswer == choice ? Color.accentColor.opacity(0.12) : Color.secondary.opacity(0.08))
                            )
                        }
                        .buttonStyle(.plain)
                    }
                }

                Button(session.submitting ? "Checking…" : "Confirm answer") {
                    Task { await session.submit(using: engine) }
                }
                .buttonStyle(.borderedProminent)
                .frame(maxWidth: .infinity, minHeight: GreTheme.minTapTarget)
                .disabled(session.selectedAnswer.isEmpty || session.submitting)

                if let submitError = session.submitError {
                    Text(submitError)
                        .font(.footnote)
                        .foregroundStyle(.red)
                }
            }
        }
        .padding(GreTheme.cardPadding)
        .background(GreTheme.cardBackground())
    }
}

struct SessionCompletePanel: View {
    let summary: SessionCompletionSummary
    let onPrimary: () -> Void
    let onSecondary: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(summary.headline)
                .font(.title3.weight(.semibold))
            Text(summary.subline)
                .font(.footnote)
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)

            VStack(spacing: 8) {
                ForEach(summary.rows, id: \.label) { row in
                    HStack(alignment: .firstTextBaseline) {
                        Text(row.label)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        Spacer(minLength: 12)
                        Text(row.value)
                            .font(.subheadline.weight(.semibold))
                            .multilineTextAlignment(.trailing)
                    }
                }
            }
            .padding(GreTheme.cardPadding)
            .background(
                RoundedRectangle(cornerRadius: GreTheme.cardRadius, style: .continuous)
                    .fill(Color(.tertiarySystemBackground))
            )

            Text(summary.nextActionDetail)
                .font(.footnote)
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)

            Button(summary.nextActionLabel, action: onPrimary)
                .buttonStyle(.borderedProminent)
                .frame(maxWidth: .infinity, minHeight: GreTheme.minTapTarget)

            Button(summary.secondaryActionLabel, action: onSecondary)
                .buttonStyle(.bordered)
                .frame(maxWidth: .infinity, minHeight: GreTheme.minTapTarget)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

struct PracticeScoreStripView: View {
    let scoreStrip: GrePracticeScoreStripView

    var body: some View {
        HStack(spacing: 8) {
            scoreItem(
                title: "Memory",
                value: ScoreFormat.scoreSummary(
                    value: scoreStrip.memoryValue,
                    low: scoreStrip.memoryLow,
                    high: scoreStrip.memoryHigh,
                    sufficient: scoreStrip.memorySufficient,
                    abstainReason: scoreStrip.memoryDetail
                )
            )
            scoreItem(
                title: "Performance",
                value: ScoreFormat.scoreSummary(
                    value: scoreStrip.performanceValue,
                    low: scoreStrip.performanceLow,
                    high: scoreStrip.performanceHigh,
                    sufficient: scoreStrip.performanceSufficient,
                    abstainReason: scoreStrip.performanceDetail
                )
            )
        }
    }

    private func scoreItem(title: String, value: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title)
                .font(.caption2.weight(.semibold))
                .foregroundStyle(.secondary)
            Text(value)
                .font(.caption.weight(.semibold))
                .lineLimit(2)
                .minimumScaleFactor(0.85)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        .background(
            RoundedRectangle(cornerRadius: GreTheme.cardRadius, style: .continuous)
                .fill(Color(.tertiarySystemBackground))
        )
    }
}

struct StudyView: View {
    @EnvironmentObject private var engine: AnkiMobileEngine
    @EnvironmentObject private var tabRouter: GreTabRouter
    @StateObject private var session = StudySession()
    @Environment(\.verticalSizeClass) private var verticalSizeClass

    private var cardMinHeight: CGFloat {
        verticalSizeClass == .compact ? 140 : 168
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                if session.isReviewing, let card = session.review?.card {
                    studyReviewContent(card: card)
                } else {
                    studyLandingContent
                }
            }
            .greScrollContentMargins()
            .navigationTitle("Study")
            .refreshable {
                await engine.refreshStudy()
                if !session.isReviewing {
                    session.reset()
                }
            }
            .task {
                if engine.study == nil && !engine.studyLoading {
                    await engine.refreshStudy()
                }
                if engine.dashboard == nil && !engine.dashboardLoading {
                    await engine.refreshDashboard()
                }
            }
        }
    }

    @ViewBuilder
    private var studyLandingContent: some View {
        GrePageContent(
            isLoading: engine.studyLoading && engine.study == nil,
            error: engine.studyError ?? session.error,
            emptyMessage: "Pull to refresh to load your study queue.",
            hasData: engine.study != nil,
            loadingLabel: "Loading study queue…"
        ) {
            if let view = engine.study {
                VStack(alignment: .leading, spacing: GreTheme.sectionSpacing) {
                    if !view.deckExists {
                        GreSectionPanel(title: "Your GRE flashcards are ready", icon: "rectangle.stack") {
                            Text(
                                "GRE Atlas includes built-in flashcards — no import needed. Pull to refresh, then start reviewing."
                            )
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .fixedSize(horizontal: false, vertical: true)
                            Button("Refresh study queue") {
                                Task {
                                    await engine.prepareDemoCollection()
                                    await engine.refreshStudy()
                                    await engine.refreshDashboard()
                                }
                            }
                            .buttonStyle(.borderedProminent)
                            .frame(maxWidth: .infinity, minHeight: GreTheme.minTapTarget)
                        }
                    } else if view.dueTotal > 0 {
                        GreSectionPanel(title: "Ready to review", icon: "rectangle.stack") {
                            Text(
                                "You have \(view.dueTotal) card\(view.dueTotal == 1 ? "" : "s") ready. A few minutes now keeps this material fresh for test day."
                            )
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .fixedSize(horizontal: false, vertical: true)

                            Button {
                                Task { await session.start(using: engine) }
                            } label: {
                                if session.starting {
                                    ProgressView()
                                        .frame(maxWidth: .infinity)
                                } else {
                                    Text("Review flashcards")
                                        .frame(maxWidth: .infinity)
                                }
                            }
                            .buttonStyle(.borderedProminent)
                            .frame(minHeight: GreTheme.minTapTarget)
                            .disabled(session.starting)

                            Text("\(view.dueNew) new · \(view.dueLearn) learning · \(view.dueReview) review")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    } else {
                        studyCaughtUpPanel(view)
                    }
                }
            }
        }
        .padding(.horizontal, GreTheme.pagePadding)
        .padding(.vertical, 8)
    }

    @ViewBuilder
    private func studyCaughtUpPanel(_ view: GreStudyView) -> some View {
        let summary = SessionCompletionBuilder.studyCaughtUpSummary(
            weakTopic: engine.dashboard?.weakTopic,
            recommendedTopics: engine.dashboard?.recommendedTopics ?? [],
            dueTotal: view.dueTotal,
            studiedCards: engine.dashboard?.memoryStudiedCards ?? 0
        )
        SessionCompletePanel(
            summary: summary,
            onPrimary: {
                if summary.nextActionLabel == "View study plan" {
                    tabRouter.openStudyPlan()
                } else {
                    tabRouter.open(.practice)
                }
            },
            onSecondary: {
                tabRouter.openStudyPlan()
            }
        )
        .padding(GreTheme.cardPadding)
        .background(GreTheme.cardBackground())
    }

    @ViewBuilder
    private func studyReviewContent(card: GreStudyCardView) -> some View {
        VStack(alignment: .leading, spacing: GreTheme.sectionSpacing) {
            if let review = session.review {
                Text("\(review.dueNew) new · \(review.dueLearn) learning · \(review.dueReview) review")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
            }

            VStack(alignment: .leading, spacing: 10) {
                Text("\(card.queue.capitalized) card")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)

                StudyCardWebView(
                    html: StudyCardDocument.html(
                        css: card.css,
                        body: session.showingAnswer ? card.answerHtml : card.questionHtml
                    )
                )
                .frame(minHeight: cardMinHeight)

                if !session.showingAnswer {
                    Button("Show answer") {
                        session.showAnswer()
                    }
                    .buttonStyle(.borderedProminent)
                    .frame(maxWidth: .infinity, minHeight: GreTheme.minTapTarget)
                } else {
                    LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 8) {
                        ForEach(card.buttons, id: \.rating) { button in
                            Button {
                                Task { await session.grade(rating: button.rating, using: engine) }
                            } label: {
                                VStack(spacing: 2) {
                                    Text(studyGradeTitle(for: button.rating))
                                        .font(.subheadline.weight(.semibold))
                                    Text(button.label)
                                        .font(.caption2)
                                }
                                .frame(maxWidth: .infinity)
                                .frame(minHeight: GreTheme.minTapTarget)
                            }
                            .buttonStyle(.bordered)
                            .disabled(session.grading)
                        }
                    }

                    if session.grading {
                        ProgressView("Saving review…")
                    }

                    if let error = session.error {
                        Text(error)
                            .font(.footnote)
                            .foregroundStyle(.red)
                    }
                }
            }
            .padding(GreTheme.cardPadding)
            .background(GreTheme.cardBackground())
        }
        .padding(.horizontal, GreTheme.pagePadding)
        .padding(.vertical, 8)
    }

    private func studyGradeTitle(for rating: UInt) -> String {
        switch rating {
        case 0: "Again"
        case 1: "Hard"
        case 2: "Good"
        case 3: "Easy"
        default: "Grade"
        }
    }
}

struct SettingsView: View {
    @EnvironmentObject private var engine: AnkiMobileEngine
    @EnvironmentObject private var syncSession: GREAtlasSyncSession
    @State private var syncEndpoint = ""
    @State private var syncHkey = ""
    @State private var credentialsMessage: String?

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: GreTheme.sectionSpacing) {
                    GreSectionPanel(title: "Study", icon: "book.closed") {
                        if let study = engine.study {
                            GreInlineMetricRow(
                                label: study.deckName,
                                value: study.deckExists ? "\(study.dueTotal) due" : "Not found",
                                detail: study.deckExists
                                    ? "\(study.dueNew) new · \(study.dueLearn) learning · \(study.dueReview) review"
                                    : "Built-in flashcards load when you open Study."
                            )
                        } else if engine.studyLoading {
                            ProgressView()
                        } else {
                            Text("Pull to refresh to load study settings context.")
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                        }
                    }

                    GreSectionPanel(title: "Practice", icon: "checkmark.rectangle") {
                        if let practice = engine.practice {
                            GreInlineMetricRow(
                                label: "Question bank",
                                value: "\(practice.questionCount) questions",
                                detail: practice.questionCount == 0
                                    ? "No practice questions found."
                                    : "Quant \(practice.quantCount) · Verbal \(practice.verbalCount) · AWA \(practice.awaCount)"
                            )
                        } else if engine.practiceLoading {
                            ProgressView()
                        } else {
                            Text("Open the Practice tab to bootstrap a session.")
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                        }
                    }

                    GreSectionPanel(title: "GRE Atlas sync", icon: "arrow.triangle.2.circlepath") {
                        Text("Practice sync uses a self-hosted Anki sync server (AnkiWeb is not supported). Use the same sync URL and hkey as desktop after sign-in.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)

                        VStack(alignment: .leading, spacing: 8) {
                            Text("Sync server URL")
                                .font(.caption.weight(.semibold))
                                .foregroundStyle(.secondary)
                            TextField("http://127.0.0.1:8080/", text: $syncEndpoint)
                                .textInputAutocapitalization(.never)
                                .autocorrectionDisabled()
                                .keyboardType(.URL)
                                .textContentType(.URL)
                                .padding(10)
                                .background(Color(.tertiarySystemBackground))
                                .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))

                            Text("Host key (hkey)")
                                .font(.caption.weight(.semibold))
                                .foregroundStyle(.secondary)
                            TextField("From desktop sign-in", text: $syncHkey)
                                .textInputAutocapitalization(.never)
                                .autocorrectionDisabled()
                                .font(.caption.monospaced())
                                .padding(10)
                                .background(Color(.tertiarySystemBackground))
                                .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
                        }

                        HStack(spacing: 8) {
                            Button("Save credentials") {
                                saveSyncCredentials()
                            }
                            .buttonStyle(.bordered)
                            .frame(minHeight: GreTheme.minTapTarget)

                            if GreAtlasSyncCredentials.load() != nil {
                                Button("Clear") {
                                    clearSyncCredentials()
                                }
                                .buttonStyle(.bordered)
                                .frame(minHeight: GreTheme.minTapTarget)
                            }
                        }

                        if let credentialsMessage {
                            Text(credentialsMessage)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }

                        if let status = syncSession.status {
                            GreInlineMetricRow(
                                label: "Sync status",
                                value: "USN \(status.currentUsn)",
                                detail: status.pendingUploadCount > 0
                                    ? "\(status.pendingUploadCount) pending upload · modified \(GREAtlasSyncFormat.relativeTime(secs: status.lastModifiedSecs))"
                                    : "Up to date · modified \(GREAtlasSyncFormat.relativeTime(secs: status.lastModifiedSecs))"
                            )
                        } else if syncSession.loadingStatus {
                            ProgressView("Loading sync status…")
                        }

                        Button {
                            triggerSyncNow(persistCredentials: true)
                        } label: {
                            if syncSession.syncing {
                                ProgressView()
                                    .frame(maxWidth: .infinity)
                            } else {
                                Text("Sync now")
                                    .frame(maxWidth: .infinity)
                            }
                        }
                        .buttonStyle(.borderedProminent)
                        .frame(minHeight: GreTheme.minTapTarget)
                        .disabled(syncSession.syncing)

                        if let result = syncSession.lastResult {
                            Text(result.message.isEmpty
                                ? "Synced \(result.appliedCount) change\(result.appliedCount == 1 ? "" : "s") · uploaded \(result.uploadedCount) · downloaded \(result.downloadedCount)."
                                : result.message)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }

                        if let result = syncSession.lastResult, !result.conflicts.isEmpty {
                            DisclosureGroup("Conflicts (\(result.conflicts.count))") {
                                VStack(alignment: .leading, spacing: 8) {
                                    ForEach(result.conflicts) { conflict in
                                        VStack(alignment: .leading, spacing: 4) {
                                            Text("Attempt \(conflict.attemptId)")
                                                .font(.caption.weight(.semibold))
                                            Text(conflict.reason)
                                                .font(.caption)
                                                .foregroundStyle(.secondary)
                                            Text("Kept: \(conflict.kept.answer) · Rejected: \(conflict.rejected.answer)")
                                                .font(.caption2)
                                                .foregroundStyle(.secondary)
                                        }
                                        .padding(8)
                                        .frame(maxWidth: .infinity, alignment: .leading)
                                        .background(Color(.tertiarySystemBackground))
                                        .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
                                    }
                                }
                                .padding(.top, 6)
                            }
                            .font(.subheadline.weight(.semibold))
                        }

                        if let error = syncSession.error {
                            Text(error)
                                .font(.footnote)
                                .foregroundStyle(.red)
                        }
                    }

                    DisclosureGroup {
                        VStack(alignment: .leading, spacing: 10) {
                            if let progress = engine.progress {
                                GreInlineMetricRow(
                                    label: "Memory evidence",
                                    value: progress.memorySufficient
                                        ? ScoreFormat.formatPercent(progress.memoryValue ?? 0)
                                        : progress.memoryDetail,
                                    detail: "\(progress.studiedCards) studied cards"
                                )
                                GreInlineMetricRow(
                                    label: "Performance evidence",
                                    value: progress.performanceSufficient
                                        ? ScoreFormat.formatPercent(progress.performanceValue ?? 0)
                                        : progress.performanceDetail,
                                    detail: "\(progress.performanceAttemptCount) attempts"
                                )
                            } else if engine.progressLoading {
                                ProgressView()
                            } else {
                                Text("Open the Progress tab to load prediction evidence.")
                                    .font(.footnote)
                                    .foregroundStyle(.secondary)
                            }
                        }
                        .padding(.top, 8)
                    } label: {
                        Label("Prediction evidence", systemImage: "chart.line.uptrend.xyaxis")
                            .font(.subheadline.weight(.semibold))
                    }

                    DisclosureGroup {
                        VStack(alignment: .leading, spacing: 6) {
                            Text("Cloud sync keeps your GRE Atlas collection up to date across devices.")
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                            Text("Account sign-in and sync details are managed on desktop for now.")
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                        }
                        .padding(.top, 8)
                    } label: {
                        Label("Account", systemImage: "person.crop.circle")
                            .font(.subheadline.weight(.semibold))
                    }
                }
                .padding(.horizontal, GreTheme.pagePadding)
                .padding(.vertical, 8)
            }
            .greScrollContentMargins()
            .navigationTitle("Settings")
            .refreshable {
                await syncSession.refreshStatus(using: engine)
                await engine.refreshStudy()
                await engine.refreshPractice()
                await engine.refreshProgress()
            }
            .task {
                loadSyncCredentials()
                await syncSession.refreshStatus(using: engine)
                await syncSession.autoSyncIfConfigured(using: engine)
                if engine.study == nil && !engine.studyLoading {
                    await engine.refreshStudy()
                }
                if engine.practice == nil && !engine.practiceLoading {
                    await engine.refreshPractice()
                }
                if engine.progress == nil && !engine.progressLoading {
                    await engine.refreshProgress()
                }
            }
        }
    }

    private func loadSyncCredentials() {
        guard let credentials = GreAtlasSyncCredentials.load() else { return }
        syncEndpoint = credentials.endpoint ?? ""
        syncHkey = credentials.hkey
    }

    private func resolvedSyncCredentials() -> GreAtlasSyncCredentials? {
        GreAtlasSyncCredentials.resolve(endpoint: syncEndpoint, hkey: syncHkey)
    }

    private func triggerSyncNow(persistCredentials: Bool) {
        let credentials = resolvedSyncCredentials()

        if persistCredentials, let credentials {
            GreAtlasSyncCredentials.save(credentials)
            syncEndpoint = credentials.endpoint ?? ""
            syncHkey = credentials.hkey
            credentialsMessage = "Credentials saved."
        }

        Task { @MainActor in
            await syncSession.syncNow(using: engine, credentials: credentials)
        }
    }

    private func saveSyncCredentials() {
        guard let credentials = resolvedSyncCredentials(), !credentials.hkey.isEmpty else {
            credentialsMessage = "Enter a host key from desktop sign-in."
            return
        }
        guard let endpoint = credentials.endpoint, !endpoint.isEmpty else {
            credentialsMessage = "Enter sync server URL (e.g. http://127.0.0.1:8080/)."
            return
        }
        GreAtlasSyncCredentials.save(credentials)
        syncEndpoint = credentials.endpoint ?? ""
        syncHkey = credentials.hkey
        credentialsMessage = "Credentials saved."
    }

    private func clearSyncCredentials() {
        GreAtlasSyncCredentials.save(nil)
        syncEndpoint = ""
        syncHkey = ""
        credentialsMessage = "Credentials cleared."
    }
}

struct StudyPlanSheet: View {
    let view: GreDashboardView
    let onTaskAction: (GreDailyTaskView) -> Void
    @Environment(\.dismiss) private var dismiss

    private var planSubtitle: String {
        "\(view.coverage.coveredLeafCount) of \(view.coverage.catalogLeafCount) GRE topics covered. Here's what to focus on next."
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: GreTheme.sectionSpacing) {
                    Text(planSubtitle)
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .fixedSize(horizontal: false, vertical: true)

                    if !view.studyPlanSummary.isEmpty {
                        Text(view.studyPlanSummary)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .fixedSize(horizontal: false, vertical: true)
                    }

                    if view.dailyPlanTaskCount > 0 {
                        Text(GreMissionCopy.intro(taskCount: view.dailyPlanTaskCount))
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }

                    ForEach(view.dailyPlanTasks, id: \.id) { task in
                        GreDailyTaskRow(
                            task: task,
                            dueTotal: view.dueTotal,
                            showAction: true
                        ) {
                            dismiss()
                            onTaskAction(task)
                        }
                        .padding(GreTheme.cardPadding)
                        .background(GreTheme.cardBackground())
                    }

                    DisclosureGroup("Topic coverage breakdown") {
                        GreCoveragePanel(coverage: view.coverage)
                            .padding(.top, 8)
                    }
                    .font(.subheadline.weight(.semibold))

                    DisclosureGroup {
                        if !view.recommendedTopics.isEmpty {
                            VStack(alignment: .leading, spacing: 8) {
                                ForEach(view.recommendedTopics, id: \.topicId) { topic in
                                    GreTopicInsightRow(topic: topic)
                                }
                            }
                            .padding(.top, 8)
                        } else {
                            Text("Keep reviewing and practicing to unlock ranked topic recommendations.")
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                                .padding(.top, 8)
                        }
                    } label: {
                        Label("Recommended focus areas", systemImage: "star")
                            .font(.subheadline.weight(.semibold))
                    }
                }
                .padding(.horizontal, GreTheme.pagePadding)
                .padding(.vertical, 8)
            }
            .greScrollContentMargins()
            .navigationTitle("Study plan")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") { dismiss() }
                }
            }
        }
    }
}

enum GREAtlasSyncFormat {
    static func relativeTime(secs: Int64) -> String {
        guard secs > 0 else { return "never" }
        let date = Date(timeIntervalSince1970: TimeInterval(secs))
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .short
        return formatter.localizedString(for: date, relativeTo: Date())
    }
}
