//! SR-7 — the machine gates that keep the `PendingTrigger` → `TriggerData` cutover
//! from rotting back.
//!
//! Before SR-7, `PendingTrigger` carried 13 per-keyword `Option` fields
//! (`poisonous_n`, `enlist_enlisted_creature`, `recover_cost`, `cipher_encoded_card_id`,
//! `haunt_source_object_id`, …) alongside the unified `data: Option<TriggerData>`
//! payload that had already replaced them. Every one was unconditionally `None` and
//! read nowhere; they survived only because 32 construction sites hand-spelled the
//! full field list, so each new field propagated itself by copy-paste.
//!
//! Two failure modes, two gates:
//!
//! * `pending_trigger_has_no_per_keyword_payload_fields` — the struct's field set is
//!   pinned. Re-adding a keyword-specific `Option` field fails here, forcing the
//!   author to add a `TriggerData` variant instead.
//! * `every_pending_trigger_literal_uses_blank` — every `PendingTrigger { .. }`
//!   literal in the scanned tree must build on `..PendingTrigger::blank(..)`. This is
//!   what makes the first gate cheap to satisfy: with no hand-rolled literals, adding
//!   a field costs one line in `blank()` rather than 32 copy-pasted `None`s.
//!
//! Per SR-5's lesson, every derived set here carries a non-vacuity guard: a scanner
//! that silently finds nothing would otherwise report `pass` forever.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Source trees the literal scan walks, workspace-relative.
///
/// `crates/card-defs/` is not scanned: a card definition never constructs a
/// `PendingTrigger` (it has no access to `GameState`), and after SR-6 it sits below
/// the engine in the dependency graph. Being outside every scan root is a stronger
/// exclusion than a path filter — `literal_scan_is_not_vacuous` asserts each root
/// still contributes files.
const SCAN_ROOTS: &[&str] = &[
    "crates/engine/src",
    "crates/card-types/src",
    "crates/engine/tests",
];

/// `stubs.rs` declares `PendingTrigger` and defines `blank()`, whose body *is* the
/// literal every other site delegates to. It is excluded from the literal scan and
/// pinned separately by `stubs_declares_exactly_one_pending_trigger_literal`, so the
/// exclusion cannot hide a second hand-rolled literal.
const LITERAL_SCAN_EXCLUDED: &str = "crates/card-types/src/state/stubs.rs";

/// The complete field set of `PendingTrigger` after SR-7.
///
/// Everything here is either identity (`source`, `controller`, `kind`,
/// `ability_index`) or *generic* trigger context read on `Normal` triggers
/// (`triggering_event`, `entering_object_id`, `targeting_stack_id`,
/// `triggering_player`, `exalted_attacker_id`, `defending_player_id`,
/// `damaged_player`, `combat_damage_amount`, `lki_counters`, `lki_power`,
/// `embedded_effect`) — plus `data`, which carries all per-kind payloads.
///
/// **Adding a name here is a design decision, not a formality.** If the new field is
/// only meaningful for one `PendingTriggerKind`, it belongs in a `TriggerData`
/// variant (`crates/card-types/src/state/stack.rs`), not on this struct.
const EXPECTED_FIELDS: &[&str] = &[
    "ability_index",
    "combat_damage_amount",
    "controller",
    "damaged_player",
    "data",
    "defending_player_id",
    "embedded_effect",
    "entering_object_id",
    "exalted_attacker_id",
    "kind",
    "lki_counters",
    "lki_power",
    "source",
    "targeting_stack_id",
    "triggering_event",
    "triggering_player",
];

/// The workspace root: `crates/engine/` is two levels down from it.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("engine manifest dir is <workspace>/crates/engine")
        .to_path_buf()
}

/// Blank out line comments, block comments, and string literals (raw strings
/// included), preserving byte positions and newlines.
///
/// Without this, the doc comments in *this very file* — which spell
/// `PendingTrigger { .. }` in prose — would be scanned as construction sites.
///
/// Raw strings must be handled before plain ones: an unescaped `"` inside
/// `r#"…"…"#` would otherwise desync quote-blanking for the rest of the file, and a
/// scanner that mis-slices is a scanner that reports whatever it likes.
fn strip_comments_and_strings(src: &str) -> String {
    let b: Vec<u8> = src.bytes().collect();
    let mut out = b.clone();
    let n = b.len();
    let blank = |out: &mut Vec<u8>, from: usize, to: usize| {
        for slot in out.iter_mut().take(to.min(n)).skip(from) {
            if *slot != b'\n' {
                *slot = b' ';
            }
        }
    };
    let mut i = 0;
    while i < n {
        if b[i] == b'/' && i + 1 < n && b[i + 1] == b'/' {
            let mut j = i;
            while j < n && b[j] != b'\n' {
                j += 1;
            }
            blank(&mut out, i, j);
            i = j;
        } else if b[i] == b'/' && i + 1 < n && b[i + 1] == b'*' {
            // Rust block comments nest.
            let (mut depth, mut j) = (1usize, i + 2);
            while j < n && depth > 0 {
                if b[j] == b'/' && j + 1 < n && b[j + 1] == b'*' {
                    depth += 1;
                    j += 2;
                } else if b[j] == b'*' && j + 1 < n && b[j + 1] == b'/' {
                    depth -= 1;
                    j += 2;
                } else {
                    j += 1;
                }
            }
            blank(&mut out, i, j);
            i = j;
        } else if b[i] == b'r'
            && i + 1 < n
            && (b[i + 1] == b'"' || b[i + 1] == b'#')
            // ...and the `r` starts a token, rather than ending one (`hasher`, `for`).
            && (i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_'))
        {
            // Raw string: r"…" / r#"…"# / r##"…"## …  Terminated by `"` followed by
            // exactly as many `#` as opened it; backslashes are not escapes.
            let mut hashes = 0usize;
            let mut j = i + 1;
            while j < n && b[j] == b'#' {
                hashes += 1;
                j += 1;
            }
            if j < n && b[j] == b'"' {
                j += 1;
                while j < n {
                    if b[j] == b'"' && b[j + 1..].iter().take(hashes).all(|&h| h == b'#') {
                        j += 1 + hashes;
                        break;
                    }
                    j += 1;
                }
                blank(&mut out, i, j);
                i = j;
            } else {
                // A bare identifier starting with `r`, e.g. `record`.
                i += 1;
            }
        } else if b[i] == b'"' {
            let mut j = i + 1;
            while j < n {
                if b[j] == b'\\' {
                    j += 2;
                    continue;
                }
                if b[j] == b'"' {
                    j += 1;
                    break;
                }
                j += 1;
            }
            blank(&mut out, i, j);
            i = j;
        } else {
            i += 1;
        }
    }
    String::from_utf8(out).expect("blanking preserves utf8 boundaries")
}

/// Index of the `}` matching the `{` at `open`.
fn matching_brace(src: &[u8], open: usize) -> usize {
    debug_assert_eq!(src[open], b'{');
    let mut depth = 0usize;
    for (i, &c) in src.iter().enumerate().skip(open) {
        if c == b'{' {
            depth += 1;
        } else if c == b'}' {
            depth -= 1;
            if depth == 0 {
                return i;
            }
        }
    }
    panic!("unbalanced braces from offset {open}");
}

/// Every `.rs` file under `SCAN_ROOTS`, as (workspace-relative path, source) pairs.
fn scanned_files() -> Vec<(String, String)> {
    fn walk(dir: &Path, acc: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("readable dir") {
            let path = entry.expect("readable entry").path();
            if path.is_dir() {
                walk(&path, acc);
            } else if path.extension().is_some_and(|e| e == "rs") {
                acc.push(path);
            }
        }
    }
    let root = workspace_root();
    let mut out = Vec::new();
    for scan_root in SCAN_ROOTS {
        let mut acc = Vec::new();
        walk(&root.join(scan_root), &mut acc);
        assert!(
            !acc.is_empty(),
            "scan root {scan_root} contributed zero files — the root moved or the walk is broken"
        );
        for path in acc {
            let rel = path
                .strip_prefix(&root)
                .expect("under workspace root")
                .to_string_lossy()
                .replace('\\', "/");
            out.push((rel, std::fs::read_to_string(&path).expect("readable file")));
        }
    }
    out
}

/// All `PendingTrigger {` construction sites in `src`, as `(offset, body)` pairs.
///
/// Four things put the bare name immediately before a `{` without constructing one:
/// a declaration (`pub struct PendingTrigger {`), an inherent impl
/// (`impl PendingTrigger {`), a trait impl (`impl HashInto for PendingTrigger {`), and
/// a return type (`fn blank(..) -> PendingTrigger {`). Detecting those syntactically —
/// rather than excluding `stubs.rs` and `hash.rs` wholesale — means a real literal
/// added to either file is still caught.
fn pending_trigger_literals(src: &str) -> Vec<(usize, String)> {
    let clean = strip_comments_and_strings(src);
    let bytes = clean.as_bytes();
    let mut out = Vec::new();
    for (idx, _) in clean.match_indices("PendingTrigger") {
        // The name must be followed by optional whitespace then `{`.
        let mut j = idx + "PendingTrigger".len();
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j >= bytes.len() || bytes[j] != b'{' {
            continue;
        }
        let before = clean[..idx].trim_end();
        if ["struct", "for", "impl", "->"]
            .iter()
            .any(|kw| before.ends_with(kw))
        {
            continue;
        }
        let end = matching_brace(bytes, j);
        out.push((idx, clean[j..=end].to_string()));
    }
    out
}

// ── Gate 1: the struct's field set is pinned ────────────────────────────────────

/// Field names declared on `pub struct PendingTrigger`, parsed out of the source.
fn declared_fields() -> BTreeSet<String> {
    let src = include_str!("../../../card-types/src/state/stubs.rs");
    let clean = strip_comments_and_strings(src);
    let decl = clean
        .find("pub struct PendingTrigger")
        .expect("PendingTrigger is declared in card-types/src/state/stubs.rs");
    let open = clean[decl..].find('{').expect("struct body") + decl;
    let close = matching_brace(clean.as_bytes(), open);

    let mut fields = BTreeSet::new();
    let body = &clean[open + 1..close];
    let mut depth = 0i32;
    for line in body.lines() {
        let trimmed = line.trim();
        // Only depth-0 lines are fields; `imbl::OrdMap<..>` has no braces, but a future
        // field type might, so track anyway.
        if depth == 0 {
            if let Some(rest) = trimmed.strip_prefix("pub ") {
                if let Some((name, _)) = rest.split_once(':') {
                    let name = name.trim();
                    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        fields.insert(name.to_string());
                    }
                }
            }
        }
        depth += trimmed.matches('{').count() as i32;
        depth -= trimmed.matches('}').count() as i32;
    }
    fields
}

/// SR-7: no `PendingTriggerKind`-specific payload may live as a field on the struct.
///
/// The 13 deleted names are asserted absent by name, so a revert reads as a test
/// failure rather than as a mysterious hash change.
#[test]
fn pending_trigger_has_no_per_keyword_payload_fields() {
    let declared = declared_fields();

    // Non-vacuity: a parser that returned {} would otherwise pass every check below
    // that is phrased as an absence.
    assert_eq!(
        declared.len(),
        EXPECTED_FIELDS.len(),
        "PendingTrigger field count drifted.\n  declared: {declared:?}\n  expected: {EXPECTED_FIELDS:?}\n\
         If you added a field: is it meaningful for more than one PendingTriggerKind?\n\
         If not, it belongs in a TriggerData variant (card-types/src/state/stack.rs)."
    );
    assert!(
        declared.contains("source") && declared.contains("data"),
        "field parser found {declared:?} — it is not reading the real struct body"
    );

    let expected: BTreeSet<String> = EXPECTED_FIELDS.iter().map(|s| s.to_string()).collect();
    assert_eq!(declared, expected, "PendingTrigger field set drifted");

    // The specific names SR-7 deleted. Named individually so a revert is unambiguous.
    for dead in [
        "ingest_target_player",
        "flanking_blocker_id",
        "rampage_n",
        "renown_n",
        "poisonous_n",
        "poisonous_target_player",
        "enlist_enlisted_creature",
        "recover_cost",
        "recover_card",
        "cipher_encoded_card_id",
        "cipher_encoded_object_id",
        "haunt_source_object_id",
        "haunt_source_card_id",
    ] {
        assert!(
            !declared.contains(dead),
            "`{dead}` is back on PendingTrigger. SR-7 removed it because it was always \
             None and read nowhere; its payload belongs in a TriggerData variant."
        );
    }
}

// ── Gate 2: every literal delegates to blank() ──────────────────────────────────

/// SR-7: `PendingTrigger::blank(..)` is the single place the default field set is
/// spelled, so adding a field never again requires editing 32 call sites.
#[test]
fn every_pending_trigger_literal_uses_blank() {
    let mut checked = 0usize;
    let mut offenders = Vec::new();

    for (path, src) in scanned_files() {
        if path == LITERAL_SCAN_EXCLUDED {
            continue;
        }
        for (offset, body) in pending_trigger_literals(&src) {
            checked += 1;
            if !body.contains("PendingTrigger::blank") {
                let line = src[..offset.min(src.len())].lines().count();
                offenders.push(format!("{path}:{line}"));
            }
        }
    }

    // Non-vacuity: a scanner that matched nothing reports "no offenders" forever.
    // 32 literals were collapsed by SR-7 and ~40 already used blank(); 30 is a floor
    // well under the real count and well over zero.
    assert!(
        checked >= 30,
        "literal scan found only {checked} PendingTrigger literals — the scanner is broken \
         (it must find every `PendingTrigger {{ .. }}` in {SCAN_ROOTS:?})"
    );

    assert!(
        offenders.is_empty(),
        "{} PendingTrigger literal(s) spell out fields instead of building on \
         `..PendingTrigger::blank(source, controller, kind)`:\n  {}",
        offenders.len(),
        offenders.join("\n  ")
    );
}

/// Gate 2 scans for the literal token `PendingTrigger`, so a hand-rolled literal
/// written as `Self { .. }` inside an `impl PendingTrigger` (or `impl .. for
/// PendingTrigger`) block would slip past it. Close that by forbidding `Self { .. }`
/// in every file that opens such a block.
///
/// Today those are exactly two: `stubs.rs` (`impl PendingTrigger`, whose `blank()`
/// names the type explicitly) and `hash.rs` (`impl HashInto for PendingTrigger`,
/// which constructs nothing). Neither uses `Self { .. }`, and neither may start.
#[test]
fn no_pending_trigger_impl_block_uses_a_self_literal() {
    let mut impl_files = Vec::new();

    for (path, src) in scanned_files() {
        let clean = strip_comments_and_strings(&src);
        let opens_impl = clean.lines().any(|l| {
            let l = l.trim_start();
            l.starts_with("impl")
                && l.contains("PendingTrigger")
                && !l.contains("PendingTriggerKind")
        });
        if !opens_impl {
            continue;
        }
        impl_files.push(path.clone());

        for (idx, _) in clean.match_indices("Self") {
            let mut j = idx + "Self".len();
            while j < clean.len() && clean.as_bytes()[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < clean.len() && clean.as_bytes()[j] == b'{' {
                let line = clean[..idx].lines().count();
                panic!(
                    "{path}:{line} builds a `Self {{ .. }}` literal inside an \
                     `impl .. PendingTrigger` block. Gate 2 scans for the token \
                     `PendingTrigger` and cannot see this. Use \
                     `PendingTrigger::blank(source, controller, kind)`."
                );
            }
        }
    }

    // Non-vacuity: if the scan found no impl blocks at all it proved nothing. There are
    // exactly two — `stubs.rs` (inherent) and `hash.rs` (HashInto).
    assert_eq!(
        impl_files.len(),
        2,
        "expected exactly 2 files with an `impl .. PendingTrigger` block, found {impl_files:?}. \
         A new impl block is fine — add it here — but it must not construct via `Self {{ .. }}`."
    );
}

/// The one file excluded from Gate 2 is pinned, so the exclusion cannot be used to
/// smuggle a hand-rolled literal back in next to the definition.
#[test]
fn stubs_declares_exactly_one_pending_trigger_literal() {
    let root = workspace_root();
    let src = std::fs::read_to_string(root.join(LITERAL_SCAN_EXCLUDED)).expect("stubs.rs readable");
    let literals = pending_trigger_literals(&src);
    assert_eq!(
        literals.len(),
        1,
        "{LITERAL_SCAN_EXCLUDED} should contain exactly one PendingTrigger literal — \
         the body of `blank()`. Found {}.",
        literals.len()
    );
    assert!(
        literals[0].1.contains("lki_counters"),
        "the sole literal in {LITERAL_SCAN_EXCLUDED} is not blank()'s body"
    );
}

// ── Gate 3: the payload path is the one the engine actually reads ───────────────

/// The `TriggerData` variants that replaced the 13 deleted fields must still be the
/// ones `flush_pending_triggers` and `resolve_stack_object` dispatch on.
///
/// This is the claim SR-7 rests on: the fields were safe to delete *because* the
/// payload already travelled in `data`. If a variant loses its consumer, the field
/// deletion silently became a behavior change.
///
/// **Known limit:** this is string presence, not reachability. A variant mentioned only
/// by a producer, or by dead code, would satisfy it. That is tolerable because the
/// second needle is `resolution.rs`, which is the *consumer* file — a variant named
/// there but never matched is a much narrower mistake than one deleted outright, which
/// is the regression actually observed (removing the Enlist arm leaves `cargo check`
/// green). Making this precise would mean parsing match arms; the cost is not yet worth it.
#[test]
fn replacement_trigger_data_variants_are_still_consumed() {
    let root = workspace_root();
    let flush = strip_comments_and_strings(
        &std::fs::read_to_string(root.join("crates/engine/src/rules/abilities.rs")).unwrap(),
    );
    let resolve = strip_comments_and_strings(
        &std::fs::read_to_string(root.join("crates/engine/src/rules/resolution.rs")).unwrap(),
    );

    // (deleted field family, the TriggerData variant that carries it now)
    let migrations = [
        ("ingest_target_player", "IngestExile"),
        ("flanking_blocker_id", "CombatFlanking"),
        ("rampage_n", "CombatRampage"),
        ("renown_n", "RenownDamage"),
        ("poisonous_n / poisonous_target_player", "CombatPoisonous"),
        ("enlist_enlisted_creature", "CombatEnlist"),
        ("recover_cost / recover_card", "DeathRecover"),
        ("cipher_encoded_card_id / _object_id", "CipherDamage"),
        ("haunt_source_* (HauntExile)", "DeathHauntExile"),
        (
            "haunt_source_* (HauntedCreatureDies)",
            "DeathHauntedCreatureDies",
        ),
    ];

    for (field, variant) in migrations {
        let needle = format!("TriggerData::{variant}");
        assert!(
            flush.contains(&needle),
            "`{field}` was deleted in favor of `{needle}`, but rules/abilities.rs \
             (flush_pending_triggers) no longer mentions it — the payload has no producer/reader"
        );
        assert!(
            resolve.contains(&needle),
            "`{field}` was deleted in favor of `{needle}`, but rules/resolution.rs \
             (resolve_stack_object) no longer mentions it — the trigger would resolve as a no-op"
        );
    }
}
