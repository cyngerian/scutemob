# W3 LOW Session 3 Review -- Performance Micro-optimizations & Card-DB Schema

**Review Status**: REVIEWED (2026-03-20)
**Branch**: `w3-low-s3-perf-schema`
**Tests**: 2223 passing, 0 failures
**Clippy**: Clean (no warnings)

## Changes Reviewed

| File | Issue | Description |
|------|-------|-------------|
| `crates/engine/src/rules/sba.rs` | MR-M4-09 | Move `chars.name` instead of cloning in legendary rule |
| `crates/engine/src/rules/sba.rs` | MR-M4-10 | LazyLock statics for SubType Aura/Equipment/Fortification |
| `crates/card-db/src/schema.rs` | MR-M0-08 | ON DELETE CASCADE on card_faces FK |
| `tools/mcp-server/src/main.rs` | MR-M0-10 | Doc comment on broad LIKE matching |
| `tools/mcp-server/src/rules_db.rs` | MR-M0-15 | DELETE and UPDATE triggers for rulings_fts |
| `tools/scryfall-import/src/main.rs` | MR-M0-16 | Doc comment on empty oracle_id |
| `tools/replay-viewer/src/main.rs` | MR-M9.5-06 | Default bind 0.0.0.0 -> 127.0.0.1 |

## Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| W3S3-01 | **INFO** | `sba.rs:54-59` | **LazyLock statics are correct.** `SubType` is `Clone + Debug + PartialEq + Eq + Hash + Ord`; `&*SUBTYPE_AURA` dereferences `LazyLock<SubType>` to `&SubType`, and `OrdSet::contains(&SubType)` accepts `&SubType`. The `PartialEq` derive on `SubType(pub String)` compares by string content, so reference equality is not needed. Correct. | OK |
| W3S3-02 | **INFO** | `sba.rs:951-952` | **Legendary rule name move is safe.** `chars` is an owned `Characteristics` (from `calculate_characteristics` returning `Option<Characteristics>`, or from `.clone()`). After `chars.name` is moved into the key tuple, `chars` is not used again in the loop body -- the `if !chars.supertypes.contains(...)` check occurs before the move, and `push(*id)` only uses `id`. The move is a strict improvement over `.clone()` with identical semantics. | OK |
| W3S3-03 | **INFO** | `schema.rs:50` | **ON DELETE CASCADE syntax is correct for SQLite.** SQLite supports `ON DELETE CASCADE` on FK constraints (requires `PRAGMA foreign_keys = ON` at connection time). This is a schema-only change; existing data is unaffected until the next DB recreation. No behavioral risk. | OK |
| W3S3-04 | **LOW** | `schema.rs:50` | **PRAGMA foreign_keys not verified at connection time.** SQLite foreign key enforcement is off by default; `ON DELETE CASCADE` only works if `PRAGMA foreign_keys = ON` is executed on each connection. If the card-db crate does not set this pragma, the CASCADE is inert. **Fix:** Verify that the card-db connection setup runs `PRAGMA foreign_keys = ON`. If not, add it (separate issue, not a regression from this change). | OPEN |
| W3S3-05 | **INFO** | `rules_db.rs:70-79` | **Rulings FTS triggers correctly mirror the rules_fts pattern.** DELETE trigger uses the FTS5 delete command syntax (`INSERT INTO rulings_fts(rulings_fts, rowid, ...)`) with `old.id`/`old.oracle_id`/`old.comment`. UPDATE trigger deletes old row then inserts new row. Column list matches the `rulings_fts` virtual table definition (`oracle_id, comment` plus `rowid`). The `old.id` correctly maps to the `content_rowid='rowid'` since `id INTEGER PRIMARY KEY` is a rowid alias in SQLite. Correct. | OK |
| W3S3-06 | **INFO** | `replay-viewer/main.rs:45-47` | **Bind address default changed to 127.0.0.1.** The `--host` flag still accepts any address including `0.0.0.0`, so network-accessible mode is preserved via `--host 0.0.0.0`. Default is now localhost-only, which is the secure default. Correct. | OK |
| W3S3-07 | **INFO** | `mcp-server/main.rs` | **LIKE matching doc comment is accurate.** The comment correctly notes: results capped at 5, sorted exact-first, no minimum query length, LLM caller expected to supply meaningful names. Purely documentation. | OK |
| W3S3-08 | **INFO** | `scryfall-import/main.rs` | **Empty oracle_id doc comment is accurate.** The comment correctly explains that cards without oracle_id (tokens, art cards) get empty string, and these are filtered at query time by layout exclusion. Purely documentation. | OK |

## Summary

**Total findings**: 8 (0 HIGH, 0 MEDIUM, 1 LOW, 7 INFO)

All changes are correct and well-targeted. The LazyLock statics properly avoid per-SBA-pass String allocations. The legendary rule name move eliminates a clone without changing behavior. The FTS triggers follow the established pattern exactly. The bind address change is the right security default.

The one LOW finding (W3S3-04) is a pre-existing concern about foreign key pragma enforcement, not a regression introduced by this session. The ON DELETE CASCADE addition is syntactically correct and will work once the pragma is active.

**Fix phase needed**: No. The single LOW is pre-existing and can be addressed opportunistically.
