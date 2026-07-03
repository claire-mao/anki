// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

enum StudyCardDocument {
    static func html(css: String, body: String) -> String {
        """
        <!DOCTYPE html>
        <html>
        <head>
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <style>
        body { font: -apple-system-body; margin: 16px; color: #111; }
        \(css)
        </style>
        </head>
        <body>\(body)</body>
        </html>
        """
    }
}

@MainActor
final class StudySession: ObservableObject {
    @Published private(set) var review: GreStudyReviewView?
    @Published private(set) var showingAnswer = false
    @Published private(set) var starting = false
    @Published private(set) var grading = false
    @Published var error: String?

    private var cardStartedAt = Date()

    var isReviewing: Bool {
        review?.sessionActive == true && review?.card != nil && review?.sessionComplete == false
    }

    func reset() {
        review = nil
        showingAnswer = false
        starting = false
        grading = false
        error = nil
    }

    func start(using engine: AnkiMobileEngine) async {
        starting = true
        error = nil
        showingAnswer = false
        defer { starting = false }

        do {
            let next = try await engine.startStudyReview()
            review = next
            cardStartedAt = Date()
        } catch {
            self.error = error.localizedDescription
        }
    }

    func showAnswer() {
        showingAnswer = true
    }

    func grade(rating: UInt, using engine: AnkiMobileEngine) async {
        guard let card = review?.card else { return }
        grading = true
        error = nil
        defer { grading = false }

        let elapsedMs = UInt(max(0, Date().timeIntervalSince(cardStartedAt) * 1000))
        do {
            let next = try await engine.answerStudyCard(
                cardId: card.cardId,
                rating: rating,
                millisecondsTaken: elapsedMs
            )
            review = next
            showingAnswer = false
            cardStartedAt = Date()
            await engine.refreshAfterStudyGrade()
        } catch {
            self.error = error.localizedDescription
        }
    }
}
