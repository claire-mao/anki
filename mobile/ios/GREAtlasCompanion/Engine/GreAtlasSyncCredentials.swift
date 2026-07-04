// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

/// Stored sync-server credentials for GRE Atlas practice sync.
///
/// Persisted in `UserDefaults` under key `greAtlasSyncCredentials` as JSON
/// `{ "hkey", "endpoint", "ioTimeoutSecs" }`. The Settings tab writes these
/// fields; lldb can set the same key for ad-hoc testing.
struct GreAtlasSyncCredentials: Codable, Equatable {
    var hkey: String
    var endpoint: String?
    var ioTimeoutSecs: UInt32?

    static let missingCredentialsMessage =
        "Enter sync server URL and host key, then tap Save credentials (or Sync now)."

    private static let defaultsKey = "greAtlasSyncCredentials"

    static func load() -> GreAtlasSyncCredentials? {
        guard let data = UserDefaults.standard.data(forKey: defaultsKey) else {
            return nil
        }
        return try? JSONDecoder().decode(GreAtlasSyncCredentials.self, from: data)
    }

    static func save(_ credentials: GreAtlasSyncCredentials?) {
        if let credentials {
            if let data = try? JSONEncoder().encode(credentials) {
                UserDefaults.standard.set(data, forKey: defaultsKey)
            }
        } else {
            UserDefaults.standard.removeObject(forKey: defaultsKey)
        }
    }

    /// Build credentials from Settings text fields, falling back to saved values per field.
    static func resolve(endpoint: String, hkey: String) -> GreAtlasSyncCredentials? {
        let saved = load()
        let trimmedHkey = hkey.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedEndpoint = endpoint.trimmingCharacters(in: .whitespacesAndNewlines)

        let mergedHkey = trimmedHkey.isEmpty ? (saved?.hkey ?? "") : trimmedHkey
        guard !mergedHkey.isEmpty else {
            return nil
        }

        let mergedEndpointRaw = trimmedEndpoint.isEmpty
            ? (saved?.endpoint ?? "")
            : trimmedEndpoint

        var normalizedEndpoint = mergedEndpointRaw.trimmingCharacters(in: .whitespacesAndNewlines)
        if !normalizedEndpoint.isEmpty, !normalizedEndpoint.hasSuffix("/") {
            normalizedEndpoint += "/"
        }

        return GreAtlasSyncCredentials(
            hkey: mergedHkey,
            endpoint: normalizedEndpoint.isEmpty ? nil : normalizedEndpoint,
            ioTimeoutSecs: saved?.ioTimeoutSecs ?? 30
        )
    }
}
