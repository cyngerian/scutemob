//! Scryfall bulk data importer.
//!
//! Downloads the Oracle Cards and Rulings bulk data files from Scryfall
//! and populates a SQLite database with the card-db schema.
//!
//! Usage: scryfall-import [--db PATH] [--skip-download]
//!
//! Bulk files are fetched as Scryfall's gzipped JSON-Lines (`jsonl_download_uri`, the
//! API shape since 2026) into `.scryfall-cache/*.jsonl.gz`; the pre-2026 uncompressed
//! JSON-array shape (`download_uri`, `*.json`) is still accepted for `--skip-download`
//! runs against an old cache. `tools/data-freshness.py refresh` drives this tool.
//!
//! The tool downloads bulk JSON files to a cache directory, then streams
//! them into SQLite. Re-running will replace existing data.

use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::params;
use serde::Deserialize;

/// Scryfall bulk data API response.
///
/// The current API (2026-09) publishes `jsonl_download_uri` + `compressed_size`; the
/// legacy fields `download_uri` + `size` are kept as a fallback so an API rollback
/// would not break the importer.
#[derive(Deserialize)]
struct BulkDataInfo {
    #[serde(default)]
    jsonl_download_uri: Option<String>,
    #[serde(default)]
    download_uri: Option<String>,
    /// Expected size in bytes of the gzipped JSON-Lines file (MR-M0-07 integrity check).
    #[serde(default)]
    compressed_size: u64,
    /// Expected size in bytes of the legacy JSON-array file.
    #[serde(default)]
    size: u64,
}

/// Which download the bulk-data record offers, and the expected byte size of it.
enum BulkDownload {
    /// Gzipped JSON-Lines: one card / ruling object per line.
    JsonlGz { uri: String, expected_size: u64 },
    /// Legacy uncompressed JSON array.
    JsonArray { uri: String, expected_size: u64 },
}

impl BulkDataInfo {
    fn download(&self) -> Result<BulkDownload> {
        if let Some(uri) = &self.jsonl_download_uri {
            return Ok(BulkDownload::JsonlGz {
                uri: uri.clone(),
                expected_size: self.compressed_size,
            });
        }
        if let Some(uri) = &self.download_uri {
            return Ok(BulkDownload::JsonArray {
                uri: uri.clone(),
                expected_size: self.size,
            });
        }
        anyhow::bail!("bulk-data record offers neither jsonl_download_uri nor download_uri")
    }
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

    let (oracle_path, rulings_path) = if skip_download {
        (
            existing_cache_file(&cache_dir, "oracle-cards")?,
            existing_cache_file(&cache_dir, "rulings")?,
        )
    } else {
        (
            download_bulk_file("oracle_cards", &cache_dir, "oracle-cards")?,
            download_bulk_file("rulings", &cache_dir, "rulings")?,
        )
    };

    println!("Opening database: {}", db_path);
    let mut conn = mtg_card_db::open_database(db_path)?;

    import_cards(&mut conn, &oracle_path)?;
    import_rulings(&mut conn, &rulings_path)?;

    print_stats(&conn)?;

    println!("Import complete.");
    Ok(())
}

/// With `--skip-download`, prefer the current `.jsonl.gz` cache file and fall back to a
/// legacy `.json` one.
fn existing_cache_file(cache_dir: &Path, stem: &str) -> Result<PathBuf> {
    for ext in ["jsonl.gz", "json"] {
        let p = cache_dir.join(format!("{stem}.{ext}"));
        if p.exists() {
            return Ok(p);
        }
    }
    anyhow::bail!(
        "--skip-download but neither {stem}.jsonl.gz nor {stem}.json exists in {}",
        cache_dir.display()
    )
}

/// Downloads one bulk file into the cache and returns the path written
/// (`<stem>.jsonl.gz` for the current API shape, `<stem>.json` for the legacy one).
fn download_bulk_file(data_type: &str, cache_dir: &Path, stem: &str) -> Result<PathBuf> {
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

    let (uri, expected_size, dest) = match info.download()? {
        BulkDownload::JsonlGz { uri, expected_size } => (
            uri,
            expected_size,
            cache_dir.join(format!("{stem}.jsonl.gz")),
        ),
        BulkDownload::JsonArray { uri, expected_size } => {
            (uri, expected_size, cache_dir.join(format!("{stem}.json")))
        }
    };

    println!("Downloading {} to {}...", data_type, dest.display());
    let mut response = ureq::get(&uri)
        .call()
        .context("failed to download bulk data file")?;

    let mut reader = response.body_mut().as_reader();
    let mut file = fs::File::create(&dest)?;
    std::io::copy(&mut reader, &mut file)?;
    drop(file); // flush and close before stat

    // MR-M0-07: Verify the download produced a non-empty file and matches the
    // expected size from the Scryfall bulk-data API. Mismatch is a warning (not
    // an error) because transparent decompression can shift the byte count.
    let file_size = fs::metadata(&dest)
        .context("failed to stat downloaded file")?
        .len();
    if file_size == 0 {
        anyhow::bail!(
            "Downloaded file for '{}' is empty — download may have failed or the URL has changed",
            data_type
        );
    }
    if expected_size > 0 && file_size != expected_size {
        println!(
            "Warning: {} size mismatch: Scryfall expected {} bytes, file is {} bytes \
             (may differ if response was compressed or Scryfall updated the file).",
            data_type, expected_size, file_size
        );
    } else if expected_size > 0 {
        println!("Size verified: {} bytes.", file_size);
    }

    // A stale sibling in the other format would shadow nothing (the newest download is
    // what main() passes on), but it wastes 170 MB; drop it.
    let other = if dest.extension().is_some_and(|e| e == "gz") {
        cache_dir.join(format!("{stem}.json"))
    } else {
        cache_dir.join(format!("{stem}.jsonl.gz"))
    };
    if other.exists() {
        fs::remove_file(&other)
            .with_context(|| format!("failed to remove superseded {}", other.display()))?;
        println!("Removed superseded {}.", other.display());
    }

    println!("Downloaded {}.", data_type);
    Ok(dest)
}

/// Yields every record of a bulk file as its raw JSON text, whatever the container:
/// gzipped JSON-Lines (`.jsonl.gz`), plain JSON-Lines (`.jsonl`), or a JSON array
/// (`.json`). Records come out one at a time so the caller never holds every parsed
/// struct in memory (MR-M0-06/14); the array form still buffers the raw text once.
fn bulk_records(path: &Path) -> Result<Box<dyn Iterator<Item = Result<String>>>> {
    let file = fs::File::open(path)
        .with_context(|| format!("failed to open bulk file {}", path.display()))?;
    let name = path.to_string_lossy();
    if name.ends_with(".json") {
        let raw: Vec<Box<serde_json::value::RawValue>> =
            serde_json::from_reader(BufReader::new(file))
                .context("failed to parse bulk JSON (expected a JSON array)")?;
        return Ok(Box::new(raw.into_iter().map(|r| Ok(r.get().to_string()))));
    }
    let reader: Box<dyn Read> = if name.ends_with(".gz") {
        Box::new(flate2::read::GzDecoder::new(file))
    } else {
        Box::new(file)
    };
    let lines = BufReader::new(reader)
        .lines()
        .map(|l| l.context("failed to read bulk JSON-Lines"))
        .filter(|l| !matches!(l, Ok(s) if s.trim().is_empty()));
    Ok(Box::new(lines))
}

fn import_cards(conn: &mut rusqlite::Connection, path: &Path) -> Result<()> {
    println!("Importing cards from {}...", path.display());

    // MR-M0-06/14: records are deserialized and inserted one at a time so the card
    // index can be reported on any parse error and no full Vec<ScryfallCard> is held.
    let records = bulk_records(path)?;
    let mut total = 0usize;

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

        for (idx, raw) in records.enumerate() {
            let raw = raw?;
            let card: ScryfallCard = serde_json::from_str(&raw)
                .with_context(|| format!("failed to parse card at index {}", idx))?;
            total = idx + 1;

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
                println!("  {} cards...", idx + 1);
            }
        }
    }

    tx.commit()?;
    println!("Inserted {} cards.", total);
    Ok(())
}

fn import_rulings(conn: &mut rusqlite::Connection, path: &Path) -> Result<()> {
    println!("Importing rulings from {}...", path.display());

    // MR-M0-06/14: same streaming approach as import_cards.
    let records = bulk_records(path)?;
    let mut total = 0usize;

    let tx = conn.transaction()?;

    tx.execute("DELETE FROM rulings", [])?;

    {
        let mut stmt = tx.prepare(
            "INSERT INTO rulings (oracle_id, published_at, comment) VALUES (?1, ?2, ?3)",
        )?;

        for (idx, raw) in records.enumerate() {
            let raw = raw?;
            let ruling: ScryfallRuling = serde_json::from_str(&raw)
                .with_context(|| format!("failed to parse ruling at index {}", idx))?;
            total = idx + 1;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("scryfall-import-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn bulk_info_prefers_current_jsonl_shape() {
        // The bulk-data record as served on 2026-09-14: no download_uri / size at all.
        let info: BulkDataInfo = serde_json::from_str(
            r#"{"object":"bulk_data","type":"oracle_cards",
                "updated_at":"2026-09-14T21:01:53.341+00:00",
                "jsonl_download_uri":"https://data.scryfall.io/oracle-cards/x.jsonl.gz",
                "compressed_size":24615290}"#,
        )
        .unwrap();
        match info.download().unwrap() {
            BulkDownload::JsonlGz { uri, expected_size } => {
                assert!(uri.ends_with(".jsonl.gz"));
                assert_eq!(expected_size, 24_615_290);
            }
            BulkDownload::JsonArray { .. } => panic!("expected the JSON-Lines download"),
        }
    }

    #[test]
    fn bulk_info_falls_back_to_legacy_array_shape() {
        let info: BulkDataInfo = serde_json::from_str(
            r#"{"download_uri":"https://data.scryfall.io/oracle-cards/x.json","size":42}"#,
        )
        .unwrap();
        match info.download().unwrap() {
            BulkDownload::JsonArray { uri, expected_size } => {
                assert!(uri.ends_with(".json"));
                assert_eq!(expected_size, 42);
            }
            BulkDownload::JsonlGz { .. } => panic!("expected the legacy download"),
        }
        let neither: BulkDataInfo = serde_json::from_str("{}").unwrap();
        assert!(neither.download().is_err());
    }

    #[test]
    fn bulk_records_reads_gzipped_jsonl_and_json_array_alike() {
        let a = r#"{"id":"a","name":"Alpha"}"#;
        let b = r#"{"id":"b","name":"Beta"}"#;

        let gz = scratch("cards.jsonl.gz");
        {
            let mut enc = flate2::write::GzEncoder::new(
                fs::File::create(&gz).unwrap(),
                flate2::Compression::fast(),
            );
            // Blank line in the middle: must be skipped, not parsed.
            write!(enc, "{a}\n\n{b}\n").unwrap();
            enc.finish().unwrap();
        }
        let from_gz: Vec<String> = bulk_records(&gz).unwrap().map(Result::unwrap).collect();
        assert_eq!(from_gz, vec![a.to_string(), b.to_string()]);

        let arr = scratch("cards.json");
        fs::write(&arr, format!("[{a},\n {b}]")).unwrap();
        let from_arr: Vec<String> = bulk_records(&arr).unwrap().map(Result::unwrap).collect();
        assert_eq!(from_arr.len(), 2);
        let parsed: serde_json::Value = serde_json::from_str(&from_arr[1]).unwrap();
        assert_eq!(parsed["name"], "Beta");
    }

    #[test]
    fn existing_cache_file_prefers_jsonl_gz() {
        let dir = scratch("cache-pref");
        fs::create_dir_all(&dir).unwrap();
        assert!(existing_cache_file(&dir, "rulings").is_err());
        fs::write(dir.join("rulings.json"), "[]").unwrap();
        assert!(existing_cache_file(&dir, "rulings")
            .unwrap()
            .ends_with("rulings.json"));
        fs::write(dir.join("rulings.jsonl.gz"), "").unwrap();
        assert!(existing_cache_file(&dir, "rulings")
            .unwrap()
            .ends_with("rulings.jsonl.gz"));
    }
}
