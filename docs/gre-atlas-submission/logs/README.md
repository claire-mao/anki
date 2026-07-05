# Wednesday submission logs

Raw command output captured **2026-07-05** at commit `1323b37859cc9baaa5a8a1a850a20fe76d3c0e8f`.

| Log                                  | Command                                         |
| ------------------------------------ | ----------------------------------------------- |
| `wednesday-build.log`                | `just build`                                    |
| `wednesday-cargo-test.log`           | `cargo test -p anki`                            |
| `wednesday-pytest-direct.log`        | `pytest pylib/tests` (direct re-run for counts) |
| `wednesday-pytest-qt.log`            | `pytest qt/tests` (direct re-run for counts)    |
| `wednesday-pytest-vitest-full.log`   | `./ninja check:pytest check:vitest`             |
| `wednesday-vitest-direct.log`        | `vitest run` from `ts/`                         |
| `wednesday-mobile-bridge.log`        | `mobile/ios/scripts/build-mobile-bridge.sh`     |
| `wednesday-xcodebuild-simulator.log` | `xcodebuild` GREAtlasCompanion (iPhone 17 sim)  |

Summaries: [build.md](../build.md), [tests.md](../tests.md), [artifacts.md](../artifacts.md).
