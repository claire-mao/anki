// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Writes a pre-seeded demo collection for the iOS app bundle.
//!
//! Usage:
//!   cargo run -p mobile_bridge --bin generate_ios_demo_bundle -- <output_dir>

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anki::collection::CollectionBuilder;
use anki::prelude::*;

fn main() -> Result<()> {
    let output_dir = env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ios/GREAtlasCompanion/Resources/DemoBundle")
    });

    fs::create_dir_all(&output_dir).expect("create output directory");

    let collection_path = output_dir.join("collection.anki2");
    remove_if_exists(&collection_path);
    remove_if_exists(&output_dir.join("collection.mdb"));
    remove_if_exists(&output_dir.join("greatlas.db"));
    remove_if_exists(&output_dir.join("brainlift.db"));
    remove_dir_if_exists(&output_dir.join("collection.media"));

    let mut col = CollectionBuilder::new(&collection_path).build()?;
    let response = col.gre_atlas_prepare_demo_collection()?;
    col.close(None)?;

    if !response.deck_created {
        eprintln!("warning: expected a fresh demo deck but deck already existed");
    }
    if response.cards_added != 8 {
        eprintln!(
            "warning: expected 8 demo flashcards, got {}",
            response.cards_added
        );
    }
    if response.practice_attempts_added != 4 {
        eprintln!(
            "warning: expected 4 demo practice attempts, got {}",
            response.practice_attempts_added
        );
    }

    let gre_atlas_db = output_dir.join("greatlas.db");
    if !gre_atlas_db.is_file() {
        eprintln!(
            "error: greatlas.db missing after seeding at {}",
            gre_atlas_db.display()
        );
        std::process::exit(1);
    }

    let media_dir = output_dir.join("collection.media");
    if !media_dir.is_dir() {
        fs::create_dir_all(&media_dir).expect("create collection.media directory");
    }

    let mdb = output_dir.join("collection.mdb");
    if !mdb.is_file() {
        eprintln!(
            "note: collection.mdb not created during seeding; iOS will create it on first open"
        );
    }

    println!(
        "Wrote iOS demo bundle to {} (collection + greatlas.db)",
        output_dir.display()
    );
    Ok(())
}

fn remove_if_exists(path: &Path) {
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}

fn remove_dir_if_exists(path: &Path) {
    if path.exists() {
        let _ = fs::remove_dir_all(path);
    }
}
