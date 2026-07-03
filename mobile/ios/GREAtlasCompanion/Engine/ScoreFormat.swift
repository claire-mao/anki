// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

enum ScoreFormat {
    static func formatPercent(_ value: Double) -> String {
        "\(Int(value.rounded()))%"
    }

    static func formatRatio(_ ratio: Double) -> String {
        formatPercent(ratio * 100)
    }

    static func formatRange(low: Double?, high: Double?) -> String? {
        guard let low, let high else { return nil }
        return "\(formatPercent(low))–\(formatPercent(high))"
    }

    static func formatGreScore(_ score: UInt) -> String {
        String(score)
    }

    static func formatGreScoreRange(low: UInt?, high: UInt?) -> String? {
        guard let low, let high else { return nil }
        if low == high { return nil }
        return "\(low)–\(high)"
    }

    static func scoreSummary(
        value: Double?,
        low: Double?,
        high: Double?,
        sufficient: Bool,
        abstainReason: String
    ) -> String {
        guard sufficient, let value else { return abstainReason }
        if let range = formatRange(low: low, high: high) {
            return "\(formatPercent(value)) (\(range))"
        }
        return formatPercent(value)
    }

    static func estimatedGreSummary(
        combined: UInt?,
        low: UInt?,
        high: UInt?,
        preliminary: Bool,
        fallback: String
    ) -> String {
        guard let combined else { return fallback }
        if let range = formatGreScoreRange(low: low, high: high) {
            let prefix = preliminary ? "~" : ""
            return "\(prefix)\(formatGreScore(combined)) (\(range))"
        }
        let prefix = preliminary ? "~" : ""
        return "\(prefix)\(formatGreScore(combined))"
    }

    static func formatResponseTimeMs(_ ms: UInt) -> String {
        String(format: "%.1fs", Double(ms) / 1000.0)
    }
}
