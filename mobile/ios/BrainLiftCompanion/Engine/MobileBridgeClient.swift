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
        let code = collectionPath.withCString { collection in
            mediaFolderPath.withCString { media in
                mediaDbPath.withCString { mediaDb in
                    anki_mobile_open_collection(backend, collection, media, mediaDb)
                }
            }
        }
        try throwIfNeeded(code)
    }

    func loadGrePages() throws -> GrePageBundle {
        GrePageBundle(
            dashboard: try loadJSON(anki_mobile_gre_dashboard_json, as: GreDashboardView.self),
            progress: try loadJSON(anki_mobile_gre_progress_json, as: GreProgressView.self),
            practice: try loadJSON(
                anki_mobile_gre_practice_bootstrap_json,
                as: GrePracticeBootstrapView.self
            ),
            study: try loadJSON(anki_mobile_gre_study_json, as: GreStudyView.self)
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
            throw MobileBridgeError.decodeFailed(error.localizedDescription)
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
            throw MobileBridgeError.backendError("Backend error while opening collection.")
        default:
            throw MobileBridgeError.backendError("Unexpected mobile bridge status \(code).")
        }
    }

    private func bridgeFailure(code: Int32, bytes: UnsafeMutablePointer<UInt8>?, length: Int) throws -> MobileBridgeError {
        switch code {
        case ANKI_MOBILE_BACKEND_ERROR:
            let message = bytes.flatMap { ptr in
                String(data: Data(bytes: ptr, count: length), encoding: .utf8)
            } ?? "Backend error."
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
    _ mediaDbPath: UnsafePointer<CChar>?
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

@_silgen_name("anki_mobile_gre_study_json")
func anki_mobile_gre_study_json(
    _ backend: OpaquePointer?,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?,
    _ outLen: UnsafeMutablePointer<Int>?
) -> Int32

@_silgen_name("anki_mobile_bytes_free")
func anki_mobile_bytes_free(_ ptr: UnsafeMutablePointer<UInt8>?, _ len: Int)

@_silgen_name("anki_mobile_last_error")
func anki_mobile_last_error(_ out: UnsafeMutablePointer<UnsafePointer<CChar>?>?) -> Int32
