// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::path::PathBuf;

use anki::collection::CollectionBuilder;
use anki::prelude::*;

const IOS_DEMO_BUNDLE_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../ios/GREAtlasCompanion/Resources/DemoBundle"
);

#[test]
fn ios_demo_bundle_contains_required_files() {
    let bundle_dir = PathBuf::from(IOS_DEMO_BUNDLE_DIR);
    assert!(
        bundle_dir.join("collection.anki2").is_file(),
        "missing bundled collection.anki2 — run mobile/ios/scripts/generate-bundled-demo-collection.sh"
    );
    assert!(
        bundle_dir.join("greatlas.db").is_file(),
        "missing bundled greatlas.db — run mobile/ios/scripts/generate-bundled-demo-collection.sh"
    );
}

#[test]
fn ios_demo_bundle_collection_is_fully_seeded() -> Result<()> {
    let collection_path = PathBuf::from(IOS_DEMO_BUNDLE_DIR).join("collection.anki2");
    let mut col = CollectionBuilder::new(&collection_path).build()?;
    let response = col.gre_atlas_prepare_demo_collection()?;
    assert!(!response.deck_created);
    assert_eq!(response.cards_added, 0);
    assert_eq!(response.practice_attempts_added, 0);
    assert!(response.due_new >= 8);
    Ok(())
}

#[test]
fn generate_ios_demo_bundle_writes_required_files() -> Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let output_dir = dir.path().join("DemoBundle");
    std::fs::create_dir_all(&output_dir)?;

    let collection_path = output_dir.join("collection.anki2");
    let mut col = CollectionBuilder::new(&collection_path).build()?;
    let _ = col.gre_atlas_prepare_demo_collection()?;
    col.close(None)?;

    assert!(collection_path.is_file());
    assert!(output_dir.join("greatlas.db").is_file());
    Ok(())
}
