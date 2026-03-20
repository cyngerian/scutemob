//! Scryfall bulk data importer.
//!
//! Downloads the Oracle Cards and Rulings bulk data files from Scryfall
//! and populates a SQLite database with the card-db schema.
//!
//! Usage: scryfall-import [--db PATH] [--skip-download]
//!
//! The tool downloads bulk JSON files to a cache directory, then streams
//! them into SQLite. Re-running will replace existing data.

use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::params;
use serde::Deserialize;

/// Scryfall bulk data API response.
#[derive(Deserialize)]
struct BulkDataInfo {
    download_uri: String,
    /// Expected file size in bytes as reported by Scryfall's bulk-data API.
    /// Used for download integrity validation (MR-M0-07).
    #[serde(default)]
    size: u64,
}

/// A card object from Scryfall's oracle cards bulk data.
#[derive(Deserialize)]
struct ScryfallCard {
    id: String,
    oracle_id: Option<String>,
    name: String,
    mana_cost: Option<String>,
    cmc: f64,
    type_line: Option<String>,
    oracle_text: Option<String>,
    power: Option<String>,
    toughness: Option<String>,
    loyalty: Option<String>,
    colors: Option<Vec<String>>,
    color_identity: Vec<String>,
    keywords: Vec<String>,
    legalities: serde_json::Value,
    set: String,
    collector_number: String,
    rarity: Option<String>,
    layout: String,
    card_faces: Option<Vec<ScryfallCardFace>>,
}

/// A card face from multi-faced cards.
#[derive(Deserialize)]
struct ScryfallCardFace {
    name: String,
    mana_cost: Option<String>,
    type_line: Option<String>,
    oracle_text: Option<String>,
    power: Option<String>,
    toughness: Option<String>,
    colors: Option<Vec<String>>,
}

/// A ruling from Scryfall's rulings bulk data.
#[derive(Deserialize)]
struct ScryfallRuling {
    oracle_id: String,
    published_at: String,
    comment: String,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let db_path = args
        .iter()
        .position(|a| a == "--db")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
        .unwrap_or("cards.sqlite");

    let skip_download = args.iter().any(|a| a == "--skip-download");

    let cache_dir = PathBuf::from(".scryfall-cache");
    fs::create_dir_all(&cache_dir)?;

    let oracle_path = cache_dir.join("oracle-cards.json");
    let rulings_path = cache_dir.join("rulings.json");

    if !skip_download {
        download_bulk_file("oracle_cards", &oracle_path)?;
        download_bulk_file("rulings", &rulings_path)?;
    }

    println!("Opening database: {}", db_path);
    let mut conn = mtg_card_db::open_database(db_path)?;

    import_cards(&mut conn, &oracle_path)?;
    import_rulings(&mut conn, &rulings_path)?;

    print_stats(&conn)?;

    println!("Import complete.");
    Ok(())
}

fn download_bulk_file(data_type: &str, dest: &Path) -> Result<()> {
    println!("Fetching {} bulk data info...", data_type);
    let url = format!("https://api.scryfall.com/bulk-data/{}", data_type);

    let body: String = ureq::get(&url)
        .call()
        .context("failed to fetch bulk data info")?
        .body_mut()
        .read_to_string()
        .context("failed to read bulk data info response")?;

    let info: BulkDataInfo =
        serde_json::from_str(&body).context("failed to parse bulk data info")?;

    println!("Downloading {} to {}...", data_type, dest.display());
    let mut response = ureq::get(&info.download_uri)
        .call()
        .context("failed to download bulk data file")?;

    let mut reader = response.body_mut().as_reader();
    let mut file = fs::File::create(dest)?;
    std::io::copy(&mut reader, &mut file)?;
    drop(file); // flush and close before stat

    // MR-M0-07: Verify the download produced a non-empty file and matches the
    // expected size from the Scryfall bulk-data API. Mismatch is a warning (not
    // an error) because transparent decompression can shift the byte count.
    let file_size = fs::metadata(dest)
        .context("failed to stat downloaded file")?
        .len();
    if file_size == 0 {
        anyhow::bail!(
            "Downloaded file for '{}' is empty — download may have failed or the URL has changed",
            data_type
        );
    }
    if info.size > 0 && file_size != info.size {
        println!(
            "Warning: {} size mismatch: Scryfall expected {} bytes, file is {} bytes \
             (may differ if response was compressed or Scryfall updated the file).",
            data_type, info.size, file_size
        );
    } else if info.size > 0 {
        println!("Size verified: {} bytes.", file_size);
    }

    println!("Downloaded {}.", data_type);
    Ok(())
}

fn import_cards(conn: &mut rusqlite::Connection, path: &Path) -> Result<()> {
    println!("Importing cards from {}...", path.display());

    let file = fs::File::open(path).context("failed to open oracle cards file")?;
    let reader = BufReader::new(file);

    // MR-M0-06/14: Parse the JSON array as RawValue elements to avoid holding all
    // ScryfallCard structs in memory simultaneously, and to report the card index
    // on any parse error. Each RawValue holds only the raw JSON bytes for one card;
    // we deserialize and insert them one at a time.
    let raw_cards: Vec<Box<serde_json::value::RawValue>> = serde_json::from_reader(reader)
        .context("failed to parse oracle cards JSON (expected a JSON array)")?;

    let total = raw_cards.len();
    println!("Found {} cards, inserting into database...", total);

    let tx = conn.transaction()?;

    // Clear existing data for clean reimport
    tx.execute_batch("DELETE FROM card_faces; DELETE FROM cards;")?;

    {
        let mut card_stmt = tx.prepare(
            "INSERT INTO cards (
                id, oracle_id, name, mana_cost, cmc, type_line, oracle_text,
                power, toughness, loyalty, colors, color_identity,
                keywords, legalities, set_code, collector_number, rarity, layout
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        )?;

        let mut face_stmt = tx.prepare(
            "INSERT INTO card_faces (
                card_id, face_index, name, mana_cost, type_line,
                oracle_text, power, toughness, colors
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )?;

        for (idx, raw) in raw_cards.iter().enumerate() {
            let card: ScryfallCard = serde_json::from_str(raw.get())
                .with_context(|| format!("failed to parse card at index {}", idx))?;

            // MR-M0-16: Cards without oracle_id (tokens, art cards) store empty string.
            // These are filtered out at query time by layout exclusion (art_series, token, etc.).
            // NULL would be semantically cleaner but empty string is safe given the layout filter.
            let oracle_id = card.oracle_id.as_deref().unwrap_or("");
            let type_line = card.type_line.as_deref().unwrap_or("");

            card_stmt.execute(params![
                card.id,
                oracle_id,
                card.name,
                card.mana_cost,
                card.cmc,
                type_line,
                card.oracle_text,
                card.power,
                card.toughness,
                card.loyalty,
                serde_json::to_string(&card.colors)?,
                serde_json::to_string(&card.color_identity)?,
                serde_json::to_string(&card.keywords)?,
                card.legalities.to_string(),
                card.set,
                card.collector_number,
                card.rarity,
                card.layout,
            ])?;

            // Insert card faces for multi-faced cards
            if let Some(faces) = &card.card_faces {
                for (i, face) in faces.iter().enumerate() {
                    let face_type_line = face.type_line.as_deref().unwrap_or("");
                    face_stmt.execute(params![
                        card.id,
                        i as i32,
                        face.name,
                        face.mana_cost,
                        face_type_line,
                        face.oracle_text,
                        face.power,
                        face.toughness,
                        serde_json::to_string(&face.colors)?,
                    ])?;
                }
            }

            if (idx + 1) % 5000 == 0 {
                println!("  {}/{} cards...", idx + 1, total);
            }
        }
    }

    tx.commit()?;
    println!("Inserted {} cards.", total);
    Ok(())
}

fn import_rulings(conn: &mut rusqlite::Connection, path: &Path) -> Result<()> {
    println!("Importing rulings from {}...", path.display());

    let file = fs::File::open(path).context("failed to open rulings file")?;
    let reader = BufReader::new(file);

    // MR-M0-06/14: Same streaming approach as import_cards — parse one ruling at
    // a time from raw JSON to report the index on parse error and avoid holding all
    // rulings as Rust structs simultaneously.
    let raw_rulings: Vec<Box<serde_json::value::RawValue>> = serde_json::from_reader(reader)
        .context("failed to parse rulings JSON (expected a JSON array)")?;

    let total = raw_rulings.len();
    println!("Found {} rulings, inserting into database...", total);

    let tx = conn.transaction()?;

    tx.execute("DELETE FROM rulings", [])?;

    {
        let mut stmt = tx.prepare(
            "INSERT INTO rulings (oracle_id, published_at, comment) VALUES (?1, ?2, ?3)",
        )?;

        for (idx, raw) in raw_rulings.iter().enumerate() {
            let ruling: ScryfallRuling = serde_json::from_str(raw.get())
                .with_context(|| format!("failed to parse ruling at index {}", idx))?;
            stmt.execute(params![
                ruling.oracle_id,
                ruling.published_at,
                ruling.comment,
            ])?;
        }
    }

    tx.commit()?;
    println!("Inserted {} rulings.", total);
    Ok(())
}

fn print_stats(conn: &rusqlite::Connection) -> Result<()> {
    let card_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM cards", [], |row: &rusqlite::Row| {
            row.get(0)
        })?;
    let face_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM card_faces",
        [],
        |row: &rusqlite::Row| row.get(0),
    )?;
    let ruling_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM rulings", [], |row: &rusqlite::Row| {
            row.get(0)
        })?;

    println!("\nDatabase statistics:");
    println!("  Cards:      {}", card_count);
    println!("  Card faces: {}", face_count);
    println!("  Rulings:    {}", ruling_count);

    // Show commander-legal card count
    let commander_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM cards WHERE json_extract(legalities, '$.commander') = 'legal'",
        [],
        |row: &rusqlite::Row| row.get(0),
    )?;
    println!("  Commander-legal: {}", commander_count);

    Ok(())
}
