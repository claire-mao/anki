# Bundled iOS demo collection

Pre-seeded files copied into `Application Support/GRE Atlas/` on first launch:

| File                | Purpose                                                       |
| ------------------- | ------------------------------------------------------------- |
| `collection.anki2`  | Anki collection with **GRE Atlas** deck and 8 demo flashcards |
| `greatlas.db`       | Practice questions + 4 sample attempts                        |
| `collection.media/` | Media folder (empty for demo cards)                           |

Regenerate after changing `PrepareDemoCollection` or seed JSON:

```bash
mobile/ios/scripts/generate-bundled-demo-collection.sh
```

Or from the repo root:

```bash
cargo run -p mobile_bridge --bin generate_ios_demo_bundle -- mobile/ios/GREAtlasCompanion/Resources/DemoBundle
```

Verification: `cargo test -p mobile_bridge ios_demo_bundle`
