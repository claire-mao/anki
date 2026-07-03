// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import SwiftUI

enum GreTheme {
    static let cardRadius: CGFloat = 14
    static let sectionSpacing: CGFloat = 16
    static let pagePadding: CGFloat = 16

    static func cardBackground() -> some View {
        RoundedRectangle(cornerRadius: cardRadius, style: .continuous)
            .fill(Color(.secondarySystemBackground))
            .shadow(color: .black.opacity(0.06), radius: 10, y: 4)
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
        .padding(16)
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
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 160), spacing: 12)], spacing: 12) {
                ForEach(0..<4, id: \.self) { _ in
                    RoundedRectangle(cornerRadius: GreTheme.cardRadius, style: .continuous)
                        .fill(.quaternary)
                        .frame(height: 110)
                        .shimmering()
                }
            }
        }
        .frame(maxWidth: .infinity, minHeight: 220)
        .padding(.vertical, 8)
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

    var body: some View {
        VStack(alignment: .leading, spacing: GreTheme.sectionSpacing) {
            if isLoading && !hasData {
                GreLoadingShell(label: loadingLabel)
                    .transition(.opacity.combined(with: .scale(scale: 0.98)))
            } else if let error {
                GreErrorBanner(message: error)
            } else if !hasData && !isLoading {
                Text(emptyMessage)
                    .foregroundStyle(.secondary)
            }

            if hasData {
                content()
                    .transition(.move(edge: .bottom).combined(with: .opacity))
            }
        }
        .animation(.spring(response: 0.35, dampingFraction: 0.86), value: hasData)
        .animation(.easeInOut(duration: 0.2), value: isLoading)
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
        .padding(16)
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

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(task.title)
                .font(.subheadline.weight(.semibold))
            if !task.detail.isEmpty {
                Text(task.detail)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
            if task.targetCount > 0 {
                Text("Target · \(task.targetCount)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
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

struct DashboardView: View {
    @EnvironmentObject private var engine: AnkiMobileEngine

    private let columns = [GridItem(.adaptive(minimum: 280), spacing: 12)]

    var body: some View {
        NavigationStack {
            ScrollView {
                GrePageContent(
                    isLoading: engine.dashboardLoading,
                    error: engine.dashboardError,
                    emptyMessage: "Pull to refresh and load your GRE dashboard from the Rust backend.",
                    hasData: engine.dashboard != nil,
                    loadingLabel: "Loading dashboard…"
                ) {
                    if let view = engine.dashboard {
                        LazyVGrid(columns: columns, spacing: 12) {
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
                                detail: view.estimatedGrePreliminary ? "Preliminary estimate" : nil
                            )
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
                            GreMetricCard(
                                icon: "calendar",
                                title: "Today's plan",
                                value: view.dailyPlanHeadline,
                                detail: "\(view.dailyPlanTaskCount) tasks · \(view.studyPlanSummary)"
                            )
                            if let weakTopic = view.weakTopic {
                                GreMetricCard(
                                    icon: "exclamationmark.circle",
                                    title: "Weakest topic",
                                    value: weakTopic.displayName,
                                    detail: weakTopic.studyLabel.isEmpty ? weakTopic.reason : weakTopic.studyLabel
                                )
                            }
                            GreMetricCard(
                                icon: "rectangle.stack",
                                title: view.deckExists ? view.deckName : "Study deck",
                                value: view.deckExists
                                    ? "\(view.dueTotal) due · \(view.dueNew) new · \(view.dueLearn) learning · \(view.dueReview) review"
                                    : "Deck not found",
                                detail: view.deckExists
                                    ? nil
                                    : "Create \"\(view.deckName)\" with gre:: tags."
                            )
                        }

                        GreCoveragePanel(coverage: view.coverage)

                        if !view.dailyPlanTasks.isEmpty || !view.dailyPlanRationale.isEmpty {
                            GreSectionPanel(title: "Daily plan", icon: "checklist") {
                                if !view.dailyPlanRationale.isEmpty {
                                    Text(view.dailyPlanRationale)
                                        .font(.footnote)
                                        .foregroundStyle(.secondary)
                                }
                                ForEach(view.dailyPlanTasks, id: \.id) { task in
                                    GreDailyTaskRow(task: task)
                                }
                            }
                        }

                        if !view.recommendedTopics.isEmpty {
                            GreSectionPanel(title: "Recommended focus", icon: "lightbulb") {
                                ForEach(view.recommendedTopics, id: \.topicId) { topic in
                                    GreTopicInsightRow(topic: topic)
                                }
                            }
                        }

                        if !view.recentActivity.isEmpty {
                            GreSectionPanel(title: "Recent practice", icon: "clock.arrow.circlepath") {
                                GreTrendSummary(trend: view.recentAccuracyTrend)
                                ForEach(Array(view.recentActivity.enumerated()), id: \.offset) { _, attempt in
                                    GreActivityRow(attempt: attempt)
                                }
                            }
                        }
                    }
                }
                .padding(GreTheme.pagePadding)
            }
            .navigationTitle("Dashboard")
            .refreshable { await engine.refreshDashboard() }
            .task {
                if engine.dashboard == nil && !engine.dashboardLoading {
                    await engine.refreshDashboard()
                }
            }
        }
    }
}

struct GreProgressScreen: View {
    @EnvironmentObject private var engine: AnkiMobileEngine

    private let columns = [GridItem(.adaptive(minimum: 280), spacing: 12)]

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
                        LazyVGrid(columns: columns, spacing: 12) {
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
                            GreMetricCard(
                                icon: "chart.line.uptrend.xyaxis",
                                title: "Calibration",
                                value: view.calibrationAssessment.isEmpty
                                    ? "Building history"
                                    : view.calibrationAssessment,
                                detail: view.calibrationWellCalibrated ? "Well calibrated" : "Needs more data"
                            )
                        }

                        GreCoveragePanel(coverage: view.coverage)

                        if !view.topicMastery.isEmpty {
                            GreSectionPanel(title: "Topic mastery", icon: "chart.bar") {
                                Text("\(view.topicCount) topics · \(view.masteredCards) mastered cards")
                                    .font(.footnote)
                                    .foregroundStyle(.secondary)
                                ForEach(view.topicMastery.prefix(8), id: \.topicId) { topic in
                                    HStack {
                                        Text(topic.displayName)
                                        Spacer()
                                        Text(ScoreFormat.formatRatio(topic.avgRetrievability))
                                            .foregroundStyle(.secondary)
                                    }
                                    .font(.subheadline)
                                }
                            }
                        }

                        if !view.weakTopics.isEmpty {
                            GreSectionPanel(title: "Weak topics", icon: "exclamationmark.circle") {
                                ForEach(view.weakTopics, id: \.topicId) { topic in
                                    GreTopicInsightRow(topic: topic)
                                }
                            }
                        }

                        if !view.recentActivity.isEmpty {
                            GreSectionPanel(title: "Recent practice", icon: "clock.arrow.circlepath") {
                                GreTrendSummary(trend: view.practiceTrend)
                                ForEach(Array(view.recentActivity.enumerated()), id: \.offset) { _, attempt in
                                    GreActivityRow(attempt: attempt)
                                }
                            }
                        }
                    }
                }
                .padding(GreTheme.pagePadding)
            }
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
    @StateObject private var session = PracticeSession()

    var body: some View {
        NavigationStack {
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

                            if !session.sessionComplete, let scoreStrip = session.scoreStrip {
                                PracticeScoreStripView(scoreStrip: scoreStrip)
                            }

                            if session.sessionComplete {
                                practiceCompletePanel
                            } else if let question = session.currentQuestion {
                                practiceQuestionPanel(question)
                            } else {
                                Text("No questions match this filter.")
                                    .foregroundStyle(.secondary)
                            }
                        }
                        .onAppear {
                            session.syncBootstrap(bootstrap)
                        }
                        .onChange(of: bootstrap.sessionId) { _ in
                            session.syncBootstrap(bootstrap)
                        }
                    }
                }
                .padding(GreTheme.pagePadding)
            }
            .navigationTitle("Practice")
            .refreshable {
                await engine.refreshPractice()
                if let bootstrap = engine.practice {
                    session.restart(from: bootstrap)
                }
            }
            .task {
                if engine.practice == nil && !engine.practiceLoading {
                    await engine.refreshPractice()
                }
                if let bootstrap = engine.practice {
                    session.syncBootstrap(bootstrap)
                }
            }
        }
    }

    @ViewBuilder
    private var practiceToolbar: some View {
        VStack(alignment: .leading, spacing: 12) {
            if !session.queue.isEmpty {
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        Text(session.progressLabel)
                            .font(.subheadline.weight(.semibold))
                        Spacer()
                        Text("\(session.progressPercent)%")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }
                    ProgressView(value: Double(session.progressPercent), total: 100)
                        .tint(.accentColor)
                }
            }

            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    ForEach(PracticeSectionFilter.allCases) { filter in
                        Button(filter.label) {
                            if let bootstrap = engine.practice {
                                session.applySectionFilter(filter, from: bootstrap)
                            }
                        }
                        .buttonStyle(.bordered)
                        .tint(session.sectionFilter == filter ? .accentColor : .secondary)
                    }
                }
            }
        }
    }

    @ViewBuilder
    private var practiceCompletePanel: some View {
        GreSectionPanel(title: "Session complete", icon: "checkmark.circle") {
            if session.attemptsRecorded == 0 {
                Text("No questions were answered in this filter.")
                    .foregroundStyle(.secondary)
            } else {
                Text(
                    "You finished \(session.attemptsRecorded) question\(session.attemptsRecorded == 1 ? "" : "s") in this session."
                )
            }
            Button("Practice again") {
                if let bootstrap = engine.practice {
                    session.applySectionFilter(session.sectionFilter, from: bootstrap)
                }
            }
            .buttonStyle(.borderedProminent)
        }
    }

    @ViewBuilder
    private func practiceQuestionPanel(_ question: GreQuestionView) -> some View {
        GreSectionPanel(title: "Question", icon: "text.book.closed") {
            Text("\(question.section) · \(question.format)")
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)

            Text(question.stem)
                .font(.body)
                .frame(maxWidth: .infinity, alignment: .leading)

            if let result = session.attemptResult {
                VStack(alignment: .leading, spacing: 8) {
                    Text(
                        "\(result.correct ? "✓ Correct" : "✗ Incorrect") · \(ScoreFormat.formatResponseTimeMs(session.responseTimeMs))"
                    )
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(result.correct ? .green : .red)

                    Text(result.explanation)
                        .font(.body)

                    Text(result.topic)
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                .padding(12)
                .background(
                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                        .fill(result.correct ? Color.green.opacity(0.08) : Color.red.opacity(0.08))
                )

                Button("Continue") {
                    session.nextQuestion()
                }
                .buttonStyle(.borderedProminent)
            } else {
                VStack(alignment: .leading, spacing: 8) {
                    ForEach(question.choices, id: \.self) { choice in
                        Button {
                            session.selectedAnswer = choice
                        } label: {
                            HStack {
                                Image(systemName: session.selectedAnswer == choice ? "largecircle.fill.circle" : "circle")
                                Text(choice)
                                    .multilineTextAlignment(.leading)
                                Spacer()
                            }
                            .padding(12)
                            .background(
                                RoundedRectangle(cornerRadius: 12, style: .continuous)
                                    .fill(session.selectedAnswer == choice ? Color.accentColor.opacity(0.12) : Color.secondary.opacity(0.08))
                            )
                        }
                        .buttonStyle(.plain)
                    }
                }

                Button(session.submitting ? "Saving attempt…" : "Submit answer") {
                    Task { await session.submit(using: engine) }
                }
                .buttonStyle(.borderedProminent)
                .disabled(session.selectedAnswer.isEmpty || session.submitting)

                if let submitError = session.submitError {
                    Text(submitError)
                        .font(.footnote)
                        .foregroundStyle(.red)
                }
            }
        }
    }
}

struct PracticeScoreStripView: View {
    let scoreStrip: GrePracticeScoreStripView

    var body: some View {
        HStack(spacing: 12) {
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
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)
            Text(value)
                .font(.subheadline.weight(.semibold))
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(12)
        .background(GreTheme.cardBackground())
    }
}

struct StudyView: View {
    @EnvironmentObject private var engine: AnkiMobileEngine
    @StateObject private var session = StudySession()

    var body: some View {
        NavigationStack {
            ScrollView {
                if session.isReviewing, let card = session.review?.card {
                    studyReviewContent(card: card)
                } else {
                    studyLandingContent
                }
            }
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
            }
        }
    }

    @ViewBuilder
    private var studyLandingContent: some View {
        GrePageContent(
            isLoading: engine.studyLoading && engine.study == nil,
            error: engine.studyError ?? session.error,
            emptyMessage: "Pull to refresh to load study queue counts from the Rust backend.",
            hasData: engine.study != nil,
            loadingLabel: "Loading study queue…"
        ) {
            if let view = engine.study {
                VStack(alignment: .leading, spacing: GreTheme.sectionSpacing) {
                    Text("Flashcard review for the GRE Atlas deck.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)

                    if view.deckExists {
                        GreMetricCard(
                            icon: "rectangle.stack",
                            title: view.deckName,
                            value: "\(view.dueTotal) cards due",
                            detail: "\(view.dueNew) new · \(view.dueLearn) learning · \(view.dueReview) review"
                        )
                        HStack(spacing: 12) {
                            dueStat(count: view.dueNew, label: "New")
                            dueStat(count: view.dueLearn, label: "Learning")
                            dueStat(count: view.dueReview, label: "Review")
                        }

                        if let review = session.review, review.sessionComplete {
                            GreSectionPanel(title: "Session complete", icon: "checkmark.circle") {
                                Text("No cards are due right now in \(view.deckName).")
                                    .foregroundStyle(.secondary)
                            }
                        }

                        Button {
                            Task { await session.start(using: engine) }
                        } label: {
                            if session.starting {
                                ProgressView()
                                    .frame(maxWidth: .infinity)
                            } else {
                                Text(view.dueTotal > 0 ? "Start review" : "Check for cards")
                                    .frame(maxWidth: .infinity)
                            }
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(session.starting || view.dueTotal == 0)
                    } else {
                        GreMetricCard(
                            icon: "rectangle.stack.badge.plus",
                            title: "Study deck",
                            value: "Deck not found",
                            detail: "Create \"\(view.deckName)\" with gre:: tagged cards."
                        )
                    }
                }
            }
        }
        .padding(GreTheme.pagePadding)
    }

    @ViewBuilder
    private func studyReviewContent(card: GreStudyCardView) -> some View {
        VStack(alignment: .leading, spacing: GreTheme.sectionSpacing) {
            if let review = session.review {
                HStack(spacing: 12) {
                    dueStat(count: review.dueNew, label: "New")
                    dueStat(count: review.dueLearn, label: "Learning")
                    dueStat(count: review.dueReview, label: "Review")
                }
            }

            GreSectionPanel(title: "Card", icon: "rectangle.on.rectangle") {
                Text("\(card.queue.capitalized) card")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)

                StudyCardWebView(
                    html: StudyCardDocument.html(
                        css: card.css,
                        body: session.showingAnswer ? card.answerHtml : card.questionHtml
                    )
                )
                .frame(minHeight: 220)

                if !session.showingAnswer {
                    Button("Show answer") {
                        session.showAnswer()
                    }
                    .buttonStyle(.borderedProminent)
                } else {
                    LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 10) {
                        ForEach(card.buttons, id: \.rating) { button in
                            Button {
                                Task { await session.grade(rating: button.rating, using: engine) }
                            } label: {
                                VStack(spacing: 4) {
                                    Text(studyGradeTitle(for: button.rating))
                                        .font(.subheadline.weight(.semibold))
                                    Text(button.label)
                                        .font(.caption)
                                }
                                .frame(maxWidth: .infinity)
                                .padding(.vertical, 10)
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
        }
        .padding(GreTheme.pagePadding)
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

    private func dueStat(count: UInt, label: String) -> some View {
        VStack(spacing: 4) {
            Text("\(count)")
                .font(.title2.weight(.bold))
            Text(label)
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 12)
        .background(GreTheme.cardBackground())
    }
}

struct SettingsView: View {
    @EnvironmentObject private var engine: AnkiMobileEngine
    @StateObject private var syncSession = GREAtlasSyncSession()

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: GreTheme.sectionSpacing) {
                    Text("Study, practice, predictions, and GRE Atlas sync.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    GreSectionPanel(title: "Study", icon: "book.closed") {
                        Text("Daily review rhythm and what you see while studying flashcards.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                        if let study = engine.study {
                            GreMetricCard(
                                icon: "rectangle.stack",
                                title: study.deckName,
                                value: study.deckExists ? "\(study.dueTotal) cards due" : "Deck not found",
                                detail: study.deckExists
                                    ? "\(study.dueNew) new · \(study.dueLearn) learning · \(study.dueReview) review"
                                    : "Create \"\(study.deckName)\" with gre:: tagged cards."
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
                        Text("Session pacing for GRE practice questions.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                        if let practice = engine.practice {
                            GreMetricCard(
                                icon: "checkmark.circle",
                                title: "Question bank",
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

                    GreSectionPanel(title: "Prediction", icon: "chart.line.uptrend.xyaxis") {
                        Text("Scheduling and deck options that power GRE Atlas score predictions.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                        if let progress = engine.progress {
                            GreMetricCard(
                                icon: "brain.head.profile",
                                title: "Memory evidence",
                                value: progress.memorySufficient
                                    ? ScoreFormat.formatPercent(progress.memoryValue ?? 0)
                                    : progress.memoryDetail,
                                detail: "\(progress.studiedCards) studied cards"
                            )
                            GreMetricCard(
                                icon: "target",
                                title: "Performance evidence",
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

                    GreSectionPanel(title: "GRE Atlas sync", icon: "arrow.triangle.2.circlepath") {
                        Text("Practice attempt sync uses the shared GRE Atlas RPCs. Pull exports local changes; push merges remote changes with newer mtime wins.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)

                        if let status = syncSession.status {
                            GreMetricCard(
                                icon: "number",
                                title: "Sync status",
                                value: "USN \(status.currentUsn)",
                                detail: status.pendingUploadCount > 0
                                    ? "\(status.pendingUploadCount) pending upload · modified \(GREAtlasSyncFormat.relativeTime(secs: status.lastModifiedSecs))"
                                    : "Up to date · modified \(GREAtlasSyncFormat.relativeTime(secs: status.lastModifiedSecs))"
                            )
                        } else if syncSession.loadingStatus {
                            ProgressView("Loading sync status…")
                        }

                        HStack(spacing: 12) {
                            Button {
                                Task { await syncSession.pull(using: engine) }
                            } label: {
                                if syncSession.pulling {
                                    ProgressView()
                                        .frame(maxWidth: .infinity)
                                } else {
                                    Text("Pull")
                                        .frame(maxWidth: .infinity)
                                }
                            }
                            .buttonStyle(.borderedProminent)
                            .disabled(syncSession.pulling || syncSession.pushing)

                            Button {
                                Task { await syncSession.pushExported(using: engine) }
                            } label: {
                                if syncSession.pushing {
                                    ProgressView()
                                        .frame(maxWidth: .infinity)
                                } else {
                                    Text("Push")
                                        .frame(maxWidth: .infinity)
                                }
                            }
                            .buttonStyle(.bordered)
                            .disabled(
                                syncSession.pulling
                                    || syncSession.pushing
                                    || syncSession.exportedAttempts.isEmpty
                            )
                        }

                        if let pull = syncSession.lastPull {
                            Text("Last pull exported \(pull.attempts.count) attempt\(pull.attempts.count == 1 ? "" : "s") at USN \(pull.currentUsn).")
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                        }

                        if let push = syncSession.lastPush {
                            Text("Last push applied \(push.appliedCount) change\(push.appliedCount == 1 ? "" : "s") at USN \(push.currentUsn).")
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                        }

                        if let push = syncSession.lastPush, !push.conflicts.isEmpty {
                            VStack(alignment: .leading, spacing: 8) {
                                Text("Conflicts (\(push.conflicts.count))")
                                    .font(.subheadline.weight(.semibold))
                                ForEach(push.conflicts) { conflict in
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
                                    .padding(10)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                    .background(GreTheme.cardBackground())
                                }
                            }
                        }

                        if let error = syncSession.error {
                            Text(error)
                                .font(.footnote)
                                .foregroundStyle(.red)
                        }
                    }

                    GreSectionPanel(title: "Account", icon: "person.crop.circle") {
                        Text("Collection sync with AnkiWeb uses the desktop SyncService RPCs.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                        Text("AnkiWeb account controls remain in desktop Anki for now.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                }
                .padding(GreTheme.pagePadding)
            }
            .navigationTitle("Settings")
            .refreshable {
                await syncSession.refreshStatus(using: engine)
                await engine.refreshStudy()
                await engine.refreshPractice()
                await engine.refreshProgress()
            }
            .task {
                await syncSession.refreshStatus(using: engine)
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
