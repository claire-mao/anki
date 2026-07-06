// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

enum MobileBridgeError: LocalizedError {
    case invalidInput
    case backendError(String)
    case panic(String)
    case decodeFailed(String)

    var errorDescription: String? {
        switch self {
        case .invalidInput:
            return "Invalid input to mobile bridge."
        case .backendError(let message):
            return message
        case .panic(let message):
            return message
        case .decodeFailed(let message):
            return message
        }
    }
}

/// Thin Swift wrapper around `mobile/mobile_bridge/include/anki_mobile.h`.
final class MobileBridgeClient {
    private var backend: OpaquePointer?
    private let decoder = JSONDecoder()

    deinit {
        if let backend {
            anki_mobile_backend_destroy(backend)
        }
    }

    func createBackend(preferredLangs: [String] = ["en"]) throws {
        if backend != nil { return }
        let initBytes = ProtobufEncoding.encodeBackendInit(preferredLangs: preferredLangs)
        var handle: OpaquePointer?
        let code = initBytes.withUnsafeBytes { raw in
            anki_mobile_backend_create(
                raw.baseAddress?.assumingMemoryBound(to: UInt8.self),
                initBytes.count,
                &handle
            )
        }
        try throwIfNeeded(code)
        guard let handle else { throw MobileBridgeError.invalidInput }
        backend = handle
    }

    func openCollection(
        collectionPath: String,
        mediaFolderPath: String,
        mediaDbPath: String
    ) throws {
        guard let backend else { throw MobileBridgeError.invalidInput }
        var outBytes: UnsafeMutablePointer<UInt8>?
        var outLen: Int = 0
        let code = collectionPath.withCString { collection in
            mediaFolderPath.withCString { media in
                mediaDbPath.withCString { mediaDb in
                    anki_mobile_open_collection(backend, collection, media, mediaDb, &outBytes, &outLen)
                }
            }
        }
        defer { anki_mobile_bytes_free(outBytes, outLen) }
        if code != ANKI_MOBILE_OK {
            throw try bridgeFailure(code: code, bytes: outBytes, length: outLen)
        }
    }

    func loadDashboard() throws -> GreDashboardView {
        try loadJSON(anki_mobile_gre_dashboard_json, as: GreDashboardView.self)
    }

    func loadProgress() throws -> GreProgressView {
        try loadJSON(anki_mobile_gre_progress_json, as: GreProgressView.self)
    }

    func loadPracticeBootstrap() throws -> GrePracticeBootstrapView {
        try loadJSON(anki_mobile_gre_practice_bootstrap_json, as: GrePracticeBootstrapView.self)
    }

    func recordPracticeAttempt(_ input: GreRecordAttemptInput) throws -> GreRecordAttemptResultView {
        let payload = try JSONEncoder().encode(input)
        return try payload.withUnsafeBytes { raw in
            try loadJSONWithInput(
                raw.baseAddress?.assumingMemoryBound(to: UInt8.self),
                payload.count,
                anki_mobile_gre_record_attempt_json,
                as: GreRecordAttemptResultView.self
            )
        }
    }

    func explainAnswer(_ input: GreExplainAnswerInput) throws -> GreAnswerExplanationView {
        let payload = try JSONEncoder().encode(input)
        return try payload.withUnsafeBytes { raw in
            try loadJSONWithInput(
                raw.baseAddress?.assumingMemoryBound(to: UInt8.self),
                payload.count,
                anki_mobile_gre_explain_answer_json,
                as: GreAnswerExplanationView.self
            )
        }
    }

    func loadPracticeScoreStrip() throws -> GrePracticeScoreStripView {
        try loadJSON(anki_mobile_gre_practice_scores_json, as: GrePracticeScoreStripView.self)
    }

    func loadStudy() throws -> GreStudyView {
        try loadJSON(anki_mobile_gre_study_json, as: GreStudyView.self)
    }

    func loadVerification() throws -> GreVerificationView {
        try loadJSON(anki_mobile_gre_verification_json, as: GreVerificationView.self)
    }

    func loadStudyReview() throws -> GreStudyReviewView {
        try loadJSON(anki_mobile_gre_study_review_json, as: GreStudyReviewView.self)
    }

    func loadStudyExtraReview() throws -> GreStudyReviewView {
        try loadJSON(anki_mobile_gre_study_extra_review_json, as: GreStudyReviewView.self)
    }

    func answerStudyCard(_ input: GreStudyAnswerInput) throws -> GreStudyReviewView {
        let payload = try JSONEncoder().encode(input)
        return try payload.withUnsafeBytes { raw in
            try loadJSONWithInput(
                raw.baseAddress?.assumingMemoryBound(to: UInt8.self),
                payload.count,
                anki_mobile_gre_study_answer_json,
                as: GreStudyReviewView.self
            )
        }
    }

    func loadGREAtlasSyncStatus() throws -> GREAtlasSyncStatusView {
        try loadJSON(anki_mobile_brainlift_sync_status_json, as: GREAtlasSyncStatusView.self)
    }

    func pullGREAtlasChanges(_ input: GREAtlasSyncPullInput) throws -> GREAtlasSyncPullView {
        let payload = try JSONEncoder().encode(input)
        return try payload.withUnsafeBytes { raw in
            try loadJSONWithInput(
                raw.baseAddress?.assumingMemoryBound(to: UInt8.self),
                payload.count,
                anki_mobile_brainlift_sync_pull_json,
                as: GREAtlasSyncPullView.self
            )
        }
    }

    func pushGREAtlasChanges(_ input: GREAtlasSyncPushInput) throws -> GREAtlasSyncPushView {
        let payload = try JSONEncoder().encode(input)
        return try payload.withUnsafeBytes { raw in
            try loadJSONWithInput(
                raw.baseAddress?.assumingMemoryBound(to: UInt8.self),
                payload.count,
                anki_mobile_brainlift_sync_push_json,
                as: GREAtlasSyncPushView.self
            )
        }
    }

    func performGREAtlasSync(_ input: GREAtlasPerformSyncInput) throws -> GREAtlasPerformSyncView {
        let payload = try JSONEncoder().encode(input)
        return try payload.withUnsafeBytes { raw in
            try loadJSONWithInput(
                raw.baseAddress?.assumingMemoryBound(to: UInt8.self),
                payload.count,
                anki_mobile_brainlift_sync_perform_json,
                as: GREAtlasPerformSyncView.self
            )
        }
    }

    func syncCollection(_ input: GRECollectionSyncInput) throws -> GRECollectionSyncView {
        let payload = try JSONEncoder().encode(input)
        return try payload.withUnsafeBytes { raw in
            try loadJSONWithInput(
                raw.baseAddress?.assumingMemoryBound(to: UInt8.self),
                payload.count,
                anki_mobile_sync_collection_json,
                as: GRECollectionSyncView.self
            )
        }
    }

    func prepareDemoCollection() throws -> GreDemoCollectionView {
        try loadJSON(anki_mobile_prepare_demo_json, as: GreDemoCollectionView.self)
    }

    func loadGrePages() throws -> GrePageBundle {
        GrePageBundle(
            dashboard: try loadDashboard(),
            progress: try loadProgress(),
            practice: try loadPracticeBootstrap(),
            study: try loadStudy()
        )
    }

    private func loadJSON<T: Decodable>(
        _ loader: @escaping (OpaquePointer?, UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>, UnsafeMutablePointer<Int>?) -> Int32,
        as type: T.Type
    ) throws -> T {
        guard let backend else { throw MobileBridgeError.invalidInput }
        var outBytes: UnsafeMutablePointer<UInt8>?
        var outLen: Int = 0
        let code = loader(backend, &outBytes, &outLen)
        defer { anki_mobile_bytes_free(outBytes, outLen) }
        guard code == ANKI_MOBILE_OK else {
            throw try bridgeFailure(code: code, bytes: outBytes, length: outLen)
        }
        guard let outBytes else { throw MobileBridgeError.decodeFailed("Empty response.") }
        let data = Data(bytes: outBytes, count: outLen)
        do {
            return try decoder.decode(T.self, from: data)
        } catch {
            throw MobileBridgeError.decodeFailed("\(T.self): \(error.localizedDescription)")
        }
    }

    private func loadJSONWithInput<T: Decodable>(
        _ input: UnsafePointer<UInt8>?,
        _ inputLen: Int,
        _ loader: @escaping (OpaquePointer?, UnsafePointer<UInt8>?, Int, UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>, UnsafeMutablePointer<Int>?) -> Int32,
        as type: T.Type
    ) throws -> T {
        guard let backend else { throw MobileBridgeError.invalidInput }
        var outBytes: UnsafeMutablePointer<UInt8>?
        var outLen: Int = 0
        let code = loader(backend, input, inputLen, &outBytes, &outLen)
        defer { anki_mobile_bytes_free(outBytes, outLen) }
        guard code == ANKI_MOBILE_OK else {
            throw try bridgeFailure(code: code, bytes: outBytes, length: outLen)
        }
        guard let outBytes else { throw MobileBridgeError.decodeFailed("Empty response.") }
        let data = Data(bytes: outBytes, count: outLen)
        do {
            return try decoder.decode(T.self, from: data)
        } catch {
            throw MobileBridgeError.decodeFailed("\(T.self): \(error.localizedDescription)")
        }
    }

    private func throwIfNeeded(_ code: Int32) throws {
        switch code {
        case ANKI_MOBILE_OK:
            return
        case ANKI_MOBILE_INVALID_INPUT:
            throw MobileBridgeError.invalidInput
        case ANKI_MOBILE_PANIC:
            throw MobileBridgeError.panic(readLastPanicMessage())
        case ANKI_MOBILE_BACKEND_ERROR:
            throw MobileBridgeError.backendError("Backend error while creating backend.")
        default:
            throw MobileBridgeError.backendError("Unexpected mobile bridge status \(code).")
        }
    }

    private func bridgeFailure(code: Int32, bytes: UnsafeMutablePointer<UInt8>?, length: Int) throws -> MobileBridgeError {
        switch code {
        case ANKI_MOBILE_BACKEND_ERROR:
            let data = bytes.map { Data(bytes: $0, count: length) }
            let message = data.flatMap(ProtobufEncoding.decodeBackendErrorMessage)
                ?? data.flatMap { String(data: $0, encoding: .utf8) }
                ?? "Backend error."
            return .backendError(message)
        case ANKI_MOBILE_PANIC:
            return .panic(readLastPanicMessage())
        case ANKI_MOBILE_INVALID_INPUT:
            return .invalidInput
        default:
            return .backendError("Unexpected mobile bridge status \(code).")
        }
    }

    private func readLastPanicMessage() -> String {
        var messagePtr: UnsafePointer<CChar>?
        let code = anki_mobile_last_error(&messagePtr)
        if code == ANKI_MOBILE_PANIC, let messagePtr {
            return String(cString: messagePtr)
        }
        return "Panic in mobile bridge."
    }
}

// MARK: - C ABI (mobile/mobile_bridge/include/anki_mobile.h)

let ANKI_MOBILE_OK: Int32 = 0
let ANKI_MOBILE_BACKEND_ERROR: Int32 = 1
let ANKI_MOBILE_INVALID_INPUT: Int32 = 2
let ANKI_MOBILE_PANIC: Int32 = 3

@_silgen_name("anki_mobile_backend_create")
func anki_mobile_backend_create(
    _ initMsg: UnsafePointer<UInt8>?,
    _ initLen: Int,
    _ outBackend: UnsafeMutablePointer<OpaquePointer?>?
) -> Int32

@_silgen_name("anki_mobile_backend_destroy")
func anki_mobile_backend_destroy(_ backend: OpaquePointer?)

@_silgen_name("anki_mobile_open_collection")
func anki_mobile_open_collection(
    _ backend: OpaquePointer?,
    _ collectionPath: UnsafePointer<CChar>?,
    _ mediaFolderPath: UnsafePointer<CChar>?,
    _ mediaDbPath: UnsafePointer<CChar>?,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_gre_dashboard_json")
func anki_mobile_gre_dashboard_json(
    _ backend: OpaquePointer?,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_gre_progress_json")
func anki_mobile_gre_progress_json(
    _ backend: OpaquePointer?,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_gre_practice_bootstrap_json")
func anki_mobile_gre_practice_bootstrap_json(
    _ backend: OpaquePointer?,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_gre_record_attempt_json")
func anki_mobile_gre_record_attempt_json(
    _ backend: OpaquePointer?,
    _ input: UnsafePointer<UInt8>?,
    _ inputLen: Int,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_gre_explain_answer_json")
func anki_mobile_gre_explain_answer_json(
    _ backend: OpaquePointer?,
    _ input: UnsafePointer<UInt8>?,
    _ inputLen: Int,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_gre_practice_scores_json")
func anki_mobile_gre_practice_scores_json(
    _ backend: OpaquePointer?,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_gre_study_json")
func anki_mobile_gre_study_json(
    _ backend: OpaquePointer?,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_gre_verification_json")
func anki_mobile_gre_verification_json(
    _ backend: OpaquePointer?,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_gre_study_review_json")
func anki_mobile_gre_study_review_json(
    _ backend: OpaquePointer?,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_gre_study_answer_json")
func anki_mobile_gre_study_answer_json(
    _ backend: OpaquePointer?,
    _ input: UnsafePointer<UInt8>?,
    _ inputLen: Int,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_prepare_demo_json")
func anki_mobile_prepare_demo_json(
    _ backend: OpaquePointer?,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_brainlift_sync_status_json")
func anki_mobile_brainlift_sync_status_json(
    _ backend: OpaquePointer?,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_brainlift_sync_pull_json")
func anki_mobile_brainlift_sync_pull_json(
    _ backend: OpaquePointer?,
    _ input: UnsafePointer<UInt8>?,
    _ inputLen: Int,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_brainlift_sync_push_json")
func anki_mobile_brainlift_sync_push_json(
    _ backend: OpaquePointer?,
    _ input: UnsafePointer<UInt8>?,
    _ inputLen: Int,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_brainlift_sync_perform_json")
func anki_mobile_brainlift_sync_perform_json(
    _ backend: OpaquePointer?,
    _ input: UnsafePointer<UInt8>?,
    _ inputLen: Int,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_sync_collection_json")
func anki_mobile_sync_collection_json(
    _ backend: OpaquePointer?,
    _ input: UnsafePointer<UInt8>?,
    _ inputLen: Int,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_bytes_free")
func anki_mobile_bytes_free(_ ptr: UnsafeMutablePointer<UInt8>?, _ len: Int)

@_silgen_name("anki_mobile_last_error")
func anki_mobile_last_error(_ out: UnsafeMutablePointer<UnsafePointer<CChar>?>?) -> Int32
