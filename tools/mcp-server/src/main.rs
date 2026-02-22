//! MCP server for MTG Comprehensive Rules and card data.
//!
//! Exposes four tools over stdio transport:
//! - search_rules: Full-text search across the Comprehensive Rules
//! - get_rule: Look up a specific rule by number
//! - lookup_card: Look up a card by name
//! - search_rulings: Full-text search across all card rulings
//!
//! Usage: mtg-mcp-server --db <path-to-cards.sqlite> --rules <path-to-CompRules.txt>
//!
//! On first run (or with --import), imports the CR text into the database.
//! Subsequent runs skip import if rules are already present.

mod rules_db;

use std::future::Future;
use std::sync::Arc;

/// Escapes a user-supplied query string for safe use in an FTS5 MATCH expression.
///
/// Wraps the query in double-quotes so FTS5 treats it as a literal phrase,
/// preventing injection of FTS5 boolean operators (AND, OR, NOT, NEAR, parentheses,
/// asterisk prefix queries, etc.). Any embedded double-quotes in the query are
/// doubled to escape them within the quoted phrase.
///
/// Fixes MR-M0-02: FTS5 MATCH operator injection.
fn escape_fts_query(query: &str) -> String {
    format!("\"{}\"", query.replace('"', "\"\""))
}

use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};
use rusqlite::Connection;
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::sync::Mutex;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchRulesRequest {
    /// Search query — keywords or concepts to find in the Comprehensive Rules.
    /// Examples: "commander damage", "layer system", "state-based actions"
    pub query: String,
    /// Maximum number of results to return (default: 10)
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRuleRequest {
    /// Rule number to look up, e.g., "704.5f", "613.8", "903"
    pub rule_number: String,
    /// If true, also return child rules (subrules). Default: true
    pub include_children: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LookupCardRequest {
    /// Card name to search for. Partial matches supported.
    /// Examples: "Lightning Bolt", "Sol Ring", "Humility"
    pub name: String,
    /// If true, include rulings for the card. Default: true
    pub include_rulings: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchRulingsRequest {
    /// Search query — keywords or concepts to find in card rulings.
    /// Examples: "copy effect double-faced", "commander zone change"
    pub query: String,
    /// Maximum number of results to return (default: 10)
    pub limit: Option<u32>,
}

#[derive(Clone)]
pub struct MtgServer {
    db: Arc<Mutex<Connection>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl MtgServer {
    pub fn new(conn: Connection) -> Self {
        Self {
            db: Arc::new(Mutex::new(conn)),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Search the MTG Comprehensive Rules by keyword or concept. Returns matching rules with their rule numbers and section context. Use this to find rules about specific mechanics, interactions, or game concepts."
    )]
    async fn search_rules(
        &self,
        Parameters(req): Parameters<SearchRulesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let db = self.db.lock().await;
        let limit = req.limit.unwrap_or(10).min(50);

        let fts_query = escape_fts_query(&req.query);
        let mut stmt = db
            .prepare(
                "SELECT r.rule_number, r.rule_text, r.section_title
                 FROM rules_fts fts
                 JOIN rules r ON r.rowid = fts.rowid
                 WHERE rules_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2",
            )
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let results: Vec<String> = stmt
            .query_map(rusqlite::params![fts_query, limit], |row| {
                let number: String = row.get(0)?;
                let text: String = row.get(1)?;
                let section: String = row.get(2)?;
                Ok(format!("[{}] {}\n  Section: {}", number, text, section))
            })
            .map_err(|e| McpError::internal_error(e.to_string(), None))?
            .filter_map(|r| r.ok())
            .collect();

        if results.is_empty() {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "No rules found matching '{}'",
                req.query
            ))]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(
                results.join("\n\n"),
            )]))
        }
    }

    #[tool(
        description = "Look up a specific MTG Comprehensive Rule by its number (e.g., '704.5f', '613.8', '903'). Returns the rule text and optionally all child/sub-rules. Use this when you know the exact rule number."
    )]
    async fn get_rule(
        &self,
        Parameters(req): Parameters<GetRuleRequest>,
    ) -> Result<CallToolResult, McpError> {
        let db = self.db.lock().await;
        let include_children = req.include_children.unwrap_or(true);

        // Get the exact rule
        let main_rule: Option<String> = db
            .query_row(
                "SELECT rule_text FROM rules WHERE rule_number = ?1",
                rusqlite::params![req.rule_number],
                |row| row.get(0),
            )
            .ok();

        let mut output = Vec::new();

        if let Some(text) = main_rule {
            output.push(text);
        } else {
            // Try prefix match for top-level sections like "903"
            let mut stmt = db
                .prepare(
                    "SELECT rule_number, rule_text FROM rules
                     WHERE rule_number LIKE ?1 || '.%'
                     ORDER BY rule_number
                     LIMIT 50",
                )
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;

            let results: Vec<String> = stmt
                .query_map(rusqlite::params![req.rule_number], |row| {
                    let number: String = row.get(0)?;
                    let text: String = row.get(1)?;
                    Ok(format!("[{}] {}", number, text))
                })
                .map_err(|e| McpError::internal_error(e.to_string(), None))?
                .filter_map(|r| r.ok())
                .collect();

            if results.is_empty() {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Rule '{}' not found",
                    req.rule_number
                ))]));
            }
            output.extend(results);
            return Ok(CallToolResult::success(vec![Content::text(
                output.join("\n\n"),
            )]));
        }

        // Get child rules if requested
        if include_children {
            let mut stmt = db
                .prepare(
                    "SELECT rule_number, rule_text FROM rules
                     WHERE parent_number = ?1
                     ORDER BY rule_number",
                )
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;

            let children: Vec<String> = stmt
                .query_map(rusqlite::params![req.rule_number], |row| {
                    let number: String = row.get(0)?;
                    let text: String = row.get(1)?;
                    Ok(format!("  [{}] {}", number, text))
                })
                .map_err(|e| McpError::internal_error(e.to_string(), None))?
                .filter_map(|r| r.ok())
                .collect();

            output.extend(children);
        }

        Ok(CallToolResult::success(vec![Content::text(
            output.join("\n\n"),
        )]))
    }

    #[tool(
        description = "Look up an MTG card by name. Returns oracle text, type line, mana cost, and optionally all rulings. Supports partial name matching. Use this to check card text, find interactions, or verify card behavior."
    )]
    async fn lookup_card(
        &self,
        Parameters(req): Parameters<LookupCardRequest>,
    ) -> Result<CallToolResult, McpError> {
        let db = self.db.lock().await;
        let include_rulings = req.include_rulings.unwrap_or(true);

        // Try exact match first, then LIKE. Exclude non-game layouts
        // (art_series, token, double_faced_token, emblem, etc.)
        let mut stmt = db
            .prepare(
                "SELECT id, oracle_id, name, mana_cost, type_line, oracle_text,
                        power, toughness, loyalty, color_identity, keywords
                 FROM cards
                 WHERE (name = ?1 OR name LIKE '%' || ?1 || '%')
                   AND layout NOT IN ('art_series', 'token', 'double_faced_token', 'emblem')
                 ORDER BY CASE WHEN name = ?1 THEN 0 ELSE 1 END
                 LIMIT 5",
            )
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let cards: Vec<(String, String)> = stmt
            .query_map(rusqlite::params![req.name], |row| {
                let oracle_id: String = row.get(1)?;
                let name: String = row.get(2)?;
                let mana_cost: Option<String> = row.get(3)?;
                let type_line: String = row.get(4)?;
                let oracle_text: Option<String> = row.get(5)?;
                let power: Option<String> = row.get(6)?;
                let toughness: Option<String> = row.get(7)?;
                let loyalty: Option<String> = row.get(8)?;
                let color_identity: String = row.get(9)?;
                let keywords: String = row.get(10)?;

                let mut parts = vec![format!("**{}**", name)];
                if let Some(mc) = mana_cost {
                    parts.push(format!("Mana Cost: {}", mc));
                }
                parts.push(format!("Type: {}", type_line));
                if let Some(text) = oracle_text {
                    parts.push(format!("Oracle Text:\n{}", text));
                }
                if let (Some(p), Some(t)) = (power, toughness) {
                    parts.push(format!("P/T: {}/{}", p, t));
                }
                if let Some(l) = loyalty {
                    parts.push(format!("Loyalty: {}", l));
                }
                parts.push(format!("Color Identity: {}", color_identity));
                parts.push(format!("Keywords: {}", keywords));

                Ok((oracle_id, parts.join("\n")))
            })
            .map_err(|e| McpError::internal_error(e.to_string(), None))?
            .filter_map(|r| r.ok())
            .collect();

        if cards.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "No cards found matching '{}'",
                req.name
            ))]));
        }

        let mut output: Vec<String> = cards.iter().map(|(_, text)| text.clone()).collect();

        // Add rulings for the first (best) match
        if include_rulings {
            let oracle_id = &cards[0].0;
            let mut ruling_stmt = db
                .prepare(
                    "SELECT published_at, comment FROM rulings
                     WHERE oracle_id = ?1
                     ORDER BY published_at",
                )
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;

            let rulings: Vec<String> = ruling_stmt
                .query_map(rusqlite::params![oracle_id], |row| {
                    let date: String = row.get(0)?;
                    let comment: String = row.get(1)?;
                    Ok(format!("  [{}] {}", date, comment))
                })
                .map_err(|e| McpError::internal_error(e.to_string(), None))?
                .filter_map(|r| r.ok())
                .collect();

            if !rulings.is_empty() {
                output.push(format!("\nRulings ({}):", rulings.len()));
                output.extend(rulings);
            }
        }

        Ok(CallToolResult::success(vec![Content::text(
            output.join("\n"),
        )]))
    }

    #[tool(
        description = "Search across all MTG card rulings by keyword or concept. Returns matching rulings with their associated card names. Use this to find rulings about specific interactions, mechanics, or edge cases."
    )]
    async fn search_rulings(
        &self,
        Parameters(req): Parameters<SearchRulingsRequest>,
    ) -> Result<CallToolResult, McpError> {
        let db = self.db.lock().await;
        let limit = req.limit.unwrap_or(10).min(50);

        let fts_query = escape_fts_query(&req.query);
        let mut stmt = db
            .prepare(
                "SELECT r.oracle_id, r.comment, r.published_at, c.name
                 FROM rulings_fts fts
                 JOIN rulings r ON r.id = fts.rowid
                 LEFT JOIN cards c ON c.oracle_id = r.oracle_id
                 WHERE rulings_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2",
            )
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let results: Vec<String> = stmt
            .query_map(rusqlite::params![fts_query, limit], |row| {
                let comment: String = row.get(1)?;
                let date: String = row.get(2)?;
                let card_name: Option<String> = row.get(3)?;
                let name = card_name.unwrap_or_else(|| "Unknown card".to_string());
                Ok(format!("[{}] {} ({})", name, comment, date))
            })
            .map_err(|e| McpError::internal_error(e.to_string(), None))?
            .filter_map(|r| r.ok())
            .collect();

        if results.is_empty() {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "No rulings found matching '{}'",
                req.query
            ))]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(
                results.join("\n\n"),
            )]))
        }
    }
}

#[tool_handler]
impl ServerHandler for MtgServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "mtg-rules-server".into(),
                version: "0.1.0".into(),
            },
            instructions: Some(
                "MTG Comprehensive Rules and card data server. \
                 Search rules by keyword, look up specific rules by number, \
                 look up cards by name, and search card rulings."
                    .into(),
            ),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let db_path = args
        .iter()
        .position(|a| a == "--db")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
        .unwrap_or("cards.sqlite");

    let rules_path = args
        .iter()
        .position(|a| a == "--rules")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str);

    let force_import = args.iter().any(|a| a == "--import");

    // Open database
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    // Create FTS tables
    rules_db::create_fts_tables(&conn)?;

    // Check if rules are already imported
    let rule_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM rules", [], |row: &rusqlite::Row| {
            row.get(0)
        })?;

    if rule_count == 0 || force_import {
        if let Some(path) = rules_path {
            eprintln!("Importing Comprehensive Rules from {}...", path);
            let cr_text = std::fs::read_to_string(path)?;
            // MR-M0-04: log the CR version/date if present in the header.
            if let Some(version) = rules_db::extract_cr_version(&cr_text) {
                eprintln!("CR version: {}", version);
            }
            let entries = rules_db::parse_rules(&cr_text);
            // MR-M0-04: sanity check — the full CR has 2000+ rules; warn if far fewer.
            const MIN_EXPECTED_RULES: usize = 500;
            if entries.len() < MIN_EXPECTED_RULES {
                eprintln!(
                    "Warning: only {} rules parsed (expected {}+). \
                     The CR file format may have changed.",
                    entries.len(),
                    MIN_EXPECTED_RULES
                );
            }
            rules_db::import_rules(&conn, &entries)?;
            eprintln!("Imported {} rules.", entries.len());
        } else if rule_count == 0 {
            eprintln!(
                "Warning: No rules in database and no --rules path provided. \
                 search_rules and get_rule will return no results."
            );
        }
    } else {
        eprintln!(
            "Rules already imported ({} entries). Use --import to reimport.",
            rule_count
        );
    }

    // Build rulings FTS index.
    // External content FTS5 tables (content='rulings') always reflect the
    // content table's row count, even when the FTS index is empty. We can't
    // use COUNT(*) to detect an unbuilt index. Instead, try a probe search —
    // if it returns nothing despite rulings existing, the index needs rebuilding.
    let rulings_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM rulings", [], |row: &rusqlite::Row| {
            row.get(0)
        })
        .unwrap_or(0);

    let fts_needs_rebuild = if rulings_count > 0 {
        // Probe the FTS index with several very common MTG ruling words — if the
        // index is populated this will match something. Using multiple OR terms
        // avoids the fragility of relying on any single word (MR-M0-05).
        let probe_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM rulings_fts WHERE rulings_fts MATCH 'card OR the OR ability OR effect OR player'",
                [],
                |row: &rusqlite::Row| row.get(0),
            )
            .unwrap_or(0);
        probe_count == 0
    } else {
        false
    };

    if fts_needs_rebuild {
        eprintln!("Building rulings FTS index ({} rulings)...", rulings_count);
        rules_db::rebuild_rulings_fts(&conn)?;
        eprintln!("Rulings FTS index built.");
    }

    eprintln!("MTG MCP server starting on stdio...");
    let service = MtgServer::new(conn).serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
