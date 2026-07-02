// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import SwiftUI

enum GreTheme {
    static let cardRadius: CGFloat = 14
    static let sectionSpacing: CGFloat = 16
    static let pagePadding: CGFloat = 16

    static func cardBackground() -> some View {
        RoundedRectangle(cornerRadius: cardRadius, style: .continuous)
            .fill(.background.secondary)
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
            } else if !hasData {
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

struct DashboardView: View {
    @EnvironmentObject private var engine: AnkiMobileEngine

    private let columns = [GridItem(.adaptive(minimum: 280), spacing: 12)]

    var body: some View {
        NavigationStack {
            ScrollView {
                GrePageContent(
                    isLoading: engine.isLoading,
                    error: engine.lastError,
                    emptyMessage: "Open a collection to load your GRE dashboard.",
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
                                detail: view.readinessSufficient ? view.readinessSummary : nil
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
                                icon: "calendar",
                                title: "Today's plan",
                                value: view.dailyPlanHeadline,
                                detail: "\(view.dailyPlanTaskCount) tasks · \(view.studyPlanSummary)"
                            )
                            if let weakTopic = view.weakTopicName {
                                GreMetricCard(
                                    icon: "exclamationmark.circle",
                                    title: "Weakest topic",
                                    value: weakTopic,
                                    detail: nil
                                )
                            }
                            GreMetricCard(
                                icon: "rectangle.stack",
                                title: view.deckExists ? view.deckName : "Study deck",
                                value: view.deckExists
                                    ? "\(view.dueNew) new · \(view.dueLearn) learning · \(view.dueReview) review"
                                    : "Deck not found",
                                detail: view.deckExists
                                    ? nil
                                    : "Create \"\(view.deckName)\" with gre:: tags."
                            )
                        }
                    }
                }
                .padding(GreTheme.pagePadding)
            }
            .navigationTitle("Dashboard")
            .refreshable { await engine.refreshAllPages() }
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
                    isLoading: engine.isLoading,
                    error: engine.lastError,
                    emptyMessage: "Progress appears after your collection has GRE study data.",
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
                                    abstainReason: "Insufficient memory evidence"
                                ),
                                detail: nil
                            )
                            GreMetricCard(
                                icon: "target",
                                title: "Performance",
                                value: ScoreFormat.scoreSummary(
                                    value: view.performanceValue,
                                    low: view.performanceLow,
                                    high: view.performanceHigh,
                                    sufficient: view.performanceSufficient,
                                    abstainReason: "Insufficient practice evidence"
                                ),
                                detail: nil
                            )
                            GreMetricCard(
                                icon: "flag.checkered",
                                title: "Readiness",
                                value: ScoreFormat.scoreSummary(
                                    value: view.readinessProjected,
                                    low: view.readinessLow,
                                    high: view.readinessHigh,
                                    sufficient: view.readinessSufficient,
                                    abstainReason: "More evidence needed"
                                ),
                                detail: nil
                            )
                            GreMetricCard(
                                icon: "number",
                                title: "Estimated GRE",
                                value: ScoreFormat.estimatedGreSummary(
                                    combined: view.estimatedGreCombined,
                                    low: view.estimatedGreLow,
                                    high: view.estimatedGreHigh,
                                    preliminary: false,
                                    fallback: "Estimate unavailable"
                                ),
                                detail: nil
                            )
                            GreMetricCard(
                                icon: "chart.pie",
                                title: "Coverage",
                                value: ScoreFormat.formatRatio(view.weightedCoverage),
                                detail: "\(view.studiedTopics) studied cards"
                            )
                            GreMetricCard(
                                icon: "chart.line.uptrend.xyaxis",
                                title: "Calibration",
                                value: view.calibrationAssessment,
                                detail: nil
                            )
                        }
                    }
                }
                .padding(GreTheme.pagePadding)
            }
            .navigationTitle("Progress")
            .refreshable { await engine.refreshAllPages() }
        }
    }
}

struct PracticeView: View {
    @EnvironmentObject private var engine: AnkiMobileEngine

    var body: some View {
        NavigationStack {
            ScrollView {
                GrePageContent(
                    isLoading: engine.isLoading,
                    error: engine.lastError,
                    emptyMessage: "Practice loads after the GRE question bank is available.",
                    hasData: engine.practice != nil,
                    loadingLabel: "Preparing practice session…"
                ) {
                    if let view = engine.practice {
                        VStack(alignment: .leading, spacing: 12) {
                            GreMetricCard(
                                icon: "checkmark.circle",
                                title: "Question bank",
                                value: "\(view.questionCount) questions",
                                detail: view.questionCount == 0
                                    ? "No practice questions found."
                                    : "Ready for a new session."
                            )
                            HStack(spacing: 12) {
                                GreMetricCard(
                                    icon: "brain.head.profile",
                                    title: "Memory",
                                    value: view.memoryValue.map(ScoreFormat.formatPercent)
                                        ?? "Unavailable",
                                    detail: nil
                                )
                                GreMetricCard(
                                    icon: "target",
                                    title: "Performance",
                                    value: view.performanceValue.map(ScoreFormat.formatPercent)
                                        ?? "Unavailable",
                                    detail: view.performanceSufficient
                                        ? "Sufficient data"
                                        : "More practice needed"
                                )
                            }
                        }
                    }
                }
                .padding(GreTheme.pagePadding)
            }
            .navigationTitle("Practice")
            .refreshable { await engine.refreshAllPages() }
        }
    }
}

struct StudyView: View {
    @EnvironmentObject private var engine: AnkiMobileEngine

    var body: some View {
        NavigationStack {
            ScrollView {
                GrePageContent(
                    isLoading: engine.isLoading,
                    error: engine.lastError,
                    emptyMessage: "Study counts load from your GRE deck.",
                    hasData: engine.study != nil,
                    loadingLabel: "Loading study queue…"
                ) {
                    if let view = engine.study {
                        if view.deckExists {
                            VStack(alignment: .leading, spacing: 12) {
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
                            }
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
                .padding(GreTheme.pagePadding)
            }
            .navigationTitle("Study")
            .refreshable { await engine.refreshAllPages() }
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
