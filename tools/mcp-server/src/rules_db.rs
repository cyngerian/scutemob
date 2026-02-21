//! Comprehensive Rules parser and FTS5 storage.
//!
//! Parses the CR text file into individual rules, each identified by its
//! rule number (e.g., "704.5f"). Rules are stored in a SQLite FTS5 table
//! for full-text search.

use rusqlite::Connection;

/// A parsed rule entry from the Comprehensive Rules.
#[derive(Debug)]
pub struct RuleEntry {
    /// Rule number, e.g., "704.5f"
    pub number: String,
    /// The full text of this rule.
    pub text: String,
    /// The section title this rule belongs to, e.g., "State-Based Actions"
    pub section_title: String,
    /// Parent rule number, e.g., "704.5" for rule "704.5f"
    pub parent_number: Option<String>,
}

/// Creates the FTS5 tables for rules and rulings search.
pub fn create_fts_tables(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS rules (
            rule_number TEXT PRIMARY KEY,
            rule_text TEXT NOT NULL,
            section_title TEXT NOT NULL,
            parent_number TEXT
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS rules_fts USING fts5(
            rule_number,
            rule_text,
            section_title,
            content='rules',
            content_rowid='rowid'
        );

        -- Triggers to keep FTS in sync
        CREATE TRIGGER IF NOT EXISTS rules_ai AFTER INSERT ON rules BEGIN
            INSERT INTO rules_fts(rowid, rule_number, rule_text, section_title)
            VALUES (new.rowid, new.rule_number, new.rule_text, new.section_title);
        END;
        CREATE TRIGGER IF NOT EXISTS rules_ad AFTER DELETE ON rules BEGIN
            INSERT INTO rules_fts(rules_fts, rowid, rule_number, rule_text, section_title)
            VALUES('delete', old.rowid, old.rule_number, old.rule_text, old.section_title);
        END;
        CREATE TRIGGER IF NOT EXISTS rules_au AFTER UPDATE ON rules BEGIN
            INSERT INTO rules_fts(rules_fts, rowid, rule_number, rule_text, section_title)
            VALUES('delete', old.rowid, old.rule_number, old.rule_text, old.section_title);
            INSERT INTO rules_fts(rowid, rule_number, rule_text, section_title)
            VALUES (new.rowid, new.rule_number, new.rule_text, new.section_title);
        END;

        -- FTS5 for rulings (using existing rulings table)
        CREATE VIRTUAL TABLE IF NOT EXISTS rulings_fts USING fts5(
            oracle_id,
            comment,
            content='rulings',
            content_rowid='rowid'
        );

        CREATE TRIGGER IF NOT EXISTS rulings_fts_ai AFTER INSERT ON rulings BEGIN
            INSERT INTO rulings_fts(rowid, oracle_id, comment)
            VALUES (new.id, new.oracle_id, new.comment);
        END;
        ",
    )?;
    Ok(())
}

/// Parses the CR text file into individual rule entries.
///
/// The CR format (after normalizing line endings):
/// - Section headers: lines like "704. State-Based Actions"
/// - Rules: lines starting with a rule number like "704.5f"
/// - Glossary entries and other non-rule content are skipped
pub fn parse_rules(cr_text: &str) -> Vec<RuleEntry> {
    let mut entries = Vec::new();
    let mut current_section_title = String::new();
    let mut section_titles: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    // Normalize line endings: \r\n -> \n, then \r -> \n
    let normalized = cr_text.replace("\r\n", "\n").replace('\r', "\n");

    // Track whether we've hit the glossary (stop parsing rules there).
    // The glossary marker only counts AFTER we've seen actual rules —
    // "Glossary" also appears in the table of contents near the top.
    let mut in_glossary = false;
    let mut seen_rules = false;

    for line in normalized.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Only detect glossary after we've parsed some rules
        // (it also appears in the table of contents)
        if line == "Glossary" && seen_rules {
            in_glossary = true;
            continue;
        }

        // Detect end of glossary (Credits section)
        // Only break after rules have been seen — "Credits" also appears
        // in the table of contents near the top of the file.
        if line == "Credits" && seen_rules {
            break;
        }

        if in_glossary {
            continue;
        }

        // Detect section headers: "704. State-Based Actions"
        // Pattern: number followed by period, space, then title
        if let Some(captures) = parse_section_header(line) {
            current_section_title = captures.1.clone();
            section_titles.insert(captures.0.clone(), captures.1);
            continue;
        }

        // Detect rule entries: lines starting with a rule number
        if let Some((rule_number, rule_text)) = parse_rule_line(line) {
            seen_rules = true;
            // Determine parent: "704.5f" -> "704.5", "704.5" -> "704"
            let parent = compute_parent(&rule_number);

            // Try to find a better section title from the top-level section number
            let section_num = rule_number.split('.').next().unwrap_or("").to_string();
            let section = section_titles
                .get(&section_num)
                .cloned()
                .unwrap_or_else(|| current_section_title.clone());

            entries.push(RuleEntry {
                number: rule_number,
                text: rule_text,
                section_title: section,
                parent_number: parent,
            });
        }
    }

    entries
}

/// Attempts to parse a line as a section header like "704. State-Based Actions"
/// Returns (section_number, section_title) if successful.
fn parse_section_header(line: &str) -> Option<(String, String)> {
    // Match: digits followed by ". " then the title
    // But NOT sub-rules like "704.5"
    let bytes = line.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_digit() {
        return None;
    }

    // Find the first period
    let dot_pos = line.find('.')?;
    let number_part = &line[..dot_pos];

    // Must be all digits (section number, not a sub-rule)
    if !number_part.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    // Must have text after ". "
    let rest = line[dot_pos + 1..].trim_start();
    if rest.is_empty() {
        return None;
    }

    // Sub-rules start with digits after the dot: "704.5" — these are NOT headers
    if rest.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }

    Some((number_part.to_string(), rest.to_string()))
}

/// Attempts to parse a line as a rule entry.
/// Returns (rule_number, full_text) if successful.
///
/// Rule number formats in the CR:
/// - "100.1." with trailing period and space before text
/// - "100.1a" subrule with letter, space before text
/// - "704.5f" subrule with letter
fn parse_rule_line(line: &str) -> Option<(String, String)> {
    let bytes = line.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_digit() {
        return None;
    }

    // Find where the rule number ends (space or end of string)
    let space_pos = line.find(' ').unwrap_or(line.len());
    let mut candidate = &line[..space_pos];

    // Must contain a dot
    if !candidate.contains('.') {
        return None;
    }

    // Strip trailing period if present (e.g., "704.5." -> "704.5")
    if candidate.ends_with('.') && candidate.matches('.').count() > 1 {
        candidate = &candidate[..candidate.len() - 1];
    }

    // Validate: before first dot is digits, after first dot is digits+optional letters
    let parts: Vec<&str> = candidate.splitn(2, '.').collect();
    if parts.len() != 2 {
        return None;
    }

    if !parts[0].chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    // After dot: digits, then optional subrule letters
    let after_dot = parts[1];
    if after_dot.is_empty() {
        return None;
    }

    let digit_end = after_dot
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(after_dot.len());
    if digit_end == 0 {
        return None;
    }

    // Any remaining chars should be lowercase letters (subrule indicators)
    let suffix = &after_dot[digit_end..];
    if !suffix.chars().all(|c| c.is_ascii_lowercase()) {
        return None;
    }

    Some((candidate.to_string(), line.to_string()))
}

/// Compute the parent rule number.
/// "704.5f" -> Some("704.5"), "704.5" -> Some("704"), "704" -> None
fn compute_parent(rule_number: &str) -> Option<String> {
    let parts: Vec<&str> = rule_number.splitn(2, '.').collect();
    if parts.len() != 2 {
        return None;
    }

    let after_dot = parts[1];

    // If it ends with letters, parent is the same number without the last letter
    if after_dot.ends_with(|c: char| c.is_ascii_lowercase()) {
        // "5f" -> "5", "5ab" -> "5a" (strip last letter)
        let parent_suffix = &after_dot[..after_dot.len() - 1];
        if parent_suffix.is_empty() {
            return Some(parts[0].to_string());
        }
        return Some(format!("{}.{}", parts[0], parent_suffix));
    }

    // Pure number subrule: "704.5" -> parent is "704"
    Some(parts[0].to_string())
}

/// Imports parsed rules into the database.
pub fn import_rules(conn: &Connection, entries: &[RuleEntry]) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM rules", [])?;
    // Rebuild FTS index
    conn.execute("INSERT INTO rules_fts(rules_fts) VALUES('rebuild')", [])?;

    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO rules (rule_number, rule_text, section_title, parent_number)
         VALUES (?1, ?2, ?3, ?4)",
    )?;

    for entry in entries {
        stmt.execute(rusqlite::params![
            entry.number,
            entry.text,
            entry.section_title,
            entry.parent_number,
        ])?;
    }

    Ok(())
}

/// Rebuilds the rulings FTS index from the existing rulings table.
pub fn rebuild_rulings_fts(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("INSERT INTO rulings_fts(rulings_fts) VALUES('rebuild')", [])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rule_line_subrule_letter() {
        let (num, text) = parse_rule_line(
            "704.5f If a creature has toughness 0 or less, it's put into its owner's graveyard.",
        )
        .unwrap();
        assert_eq!(num, "704.5f");
        assert!(text.contains("toughness 0 or less"));
    }

    #[test]
    fn test_parse_rule_line_with_trailing_period() {
        // Actual CR format: "704.5. The state-based actions are as follows:"
        let (num, _) = parse_rule_line("704.5. The state-based actions are as follows:").unwrap();
        assert_eq!(num, "704.5");
    }

    #[test]
    fn test_parse_rule_line_no_trailing_period() {
        let (num, _) = parse_rule_line("704.5 The state-based actions are as follows:").unwrap();
        assert_eq!(num, "704.5");
    }

    #[test]
    fn test_parse_section_header() {
        let (num, title) = parse_section_header("704. State-Based Actions").unwrap();
        assert_eq!(num, "704");
        assert_eq!(title, "State-Based Actions");
    }

    #[test]
    fn test_section_header_rejects_subrule() {
        // "704.5. ..." starts with digits after the first dot, so it's not a header
        assert!(parse_section_header("704.5. The state-based actions are as follows:").is_none());
    }

    #[test]
    fn test_compute_parent() {
        assert_eq!(compute_parent("704.5f"), Some("704.5".to_string()));
        assert_eq!(compute_parent("704.5"), Some("704".to_string()));
        assert_eq!(compute_parent("704"), None);
        assert_eq!(compute_parent("100.1a"), Some("100.1".to_string()));
    }

    #[test]
    fn test_parse_rules_sample_cr_format() {
        // Simulate actual CR format with \r\r separators and trailing periods
        let sample = "704. State-Based Actions\r\r\
            704.1. State-based actions are game actions that happen automatically.\r\r\
            704.5. The state-based actions are as follows:\r\r\
            704.5a If a player has 0 or less life, that player loses the game.\r\r\
            704.5f If a creature has toughness 0 or less, it's put into its owner's graveyard.\r\r";

        let entries = parse_rules(sample);
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].number, "704.1");
        assert_eq!(entries[0].section_title, "State-Based Actions");
        assert_eq!(entries[3].number, "704.5f");
        assert_eq!(entries[3].parent_number, Some("704.5".to_string()));
    }

    #[test]
    fn test_parse_rules_credits_and_glossary_in_toc() {
        // The CR file has a table of contents that lists "Glossary" and "Credits"
        // before the actual rules. These must NOT trigger the stop conditions
        // until after actual rules have been parsed.
        let sample = "\
            1. Game Concepts\r\
            100. General\r\
            Glossary\r\
            Credits\r\
            \r\
            1. Game Concepts\r\
            100. General\r\
            100.1. These Magic rules apply to any Magic game with two or more players.\r\
            100.1a A two-player game is a game that begins with only two players.\r\
            Glossary\r\
            Some glossary entry\r\
            Credits\r";

        let entries = parse_rules(sample);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].number, "100.1");
        assert_eq!(entries[1].number, "100.1a");
    }
}
