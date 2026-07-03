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

    static func decodeBackendErrorMessage(_ data: Data) -> String? {
        var offset = 0
        var message: String?
        var context: String?

        while offset < data.count, let key = readVarint(from: data, offset: &offset) {
            let fieldNumber = Int(key >> 3)
            let wireType = Int(key & 0x7)

            switch (fieldNumber, wireType) {
            case (1, 2):
                message = readLengthDelimitedString(from: data, offset: &offset)
            case (4, 2):
                context = readLengthDelimitedString(from: data, offset: &offset)
            default:
                guard skipField(wireType: wireType, data: data, offset: &offset) else {
                    return message
                }
            }
        }

        if let message, let context, !context.isEmpty {
            return "\(message) \(context)"
        }
        return message
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

    private static func readLengthDelimitedString(from data: Data, offset: inout Int) -> String? {
        guard let length = readVarint(from: data, offset: &offset) else { return nil }
        let end = offset + Int(length)
        guard end <= data.count else { return nil }
        defer { offset = end }
        return String(data: data[offset..<end], encoding: .utf8)
    }

    private static func readVarint(from data: Data, offset: inout Int) -> UInt64? {
        var result: UInt64 = 0
        var shift: UInt64 = 0

        while offset < data.count, shift < 64 {
            let byte = data[offset]
            offset += 1
            result |= UInt64(byte & 0x7F) << shift
            if byte & 0x80 == 0 {
                return result
            }
            shift += 7
        }

        return nil
    }

    private static func skipField(wireType: Int, data: Data, offset: inout Int) -> Bool {
        switch wireType {
        case 0:
            return readVarint(from: data, offset: &offset) != nil
        case 1:
            offset += 8
            return offset <= data.count
        case 2:
            guard let length = readVarint(from: data, offset: &offset) else { return false }
            offset += Int(length)
            return offset <= data.count
        case 5:
            offset += 4
            return offset <= data.count
        default:
            return false
        }
    }
}
