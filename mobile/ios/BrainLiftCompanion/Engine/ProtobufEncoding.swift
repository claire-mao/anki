// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

enum ProtobufEncoding {
    /// Encodes `anki.backend.BackendInit` for `anki_mobile_backend_create`.
    static func encodeBackendInit(preferredLangs: [String], server: Bool = false) -> Data {
        var data = Data()
        for lang in preferredLangs {
            appendStringField(tag: 1, value: lang, to: &data)
        }
        appendStringField(tag: 2, value: "", to: &data)
        appendVarintField(tag: 3, value: server ? 1 : 0, to: &data)
        return data
    }

    private static func appendStringField(tag: Int, value: String, to data: inout Data) {
        guard let encoded = value.data(using: .utf8) else { return }
        appendTag(tag, wireType: 2, to: &data)
        appendVarint(UInt64(encoded.count), to: &data)
        data.append(encoded)
    }

    private static func appendVarintField(tag: Int, value: Int, to data: inout Data) {
        appendTag(tag, wireType: 0, to: &data)
        appendVarint(UInt64(value), to: &data)
    }

    private static func appendTag(_ fieldNumber: Int, wireType: Int, to data: inout Data) {
        appendVarint(UInt64((fieldNumber << 3) | wireType), to: &data)
    }

    private static func appendVarint(_ value: UInt64, to data: inout Data) {
        var remaining = value
        repeat {
            var byte = UInt8(remaining & 0x7F)
            remaining >>= 7
            if remaining != 0 {
                byte |= 0x80
            }
            data.append(byte)
        } while remaining != 0
    }
}
