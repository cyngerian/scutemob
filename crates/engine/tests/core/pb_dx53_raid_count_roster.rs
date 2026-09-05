//! PB-DX53 (`scutemob-231`) — CR 508.3d/508.4, ruling 2007-10-01 (`OOS-DX21-1`):
//! the three censuses `memory/primitives/pb-DX53-plan.md` §7 asks for, plus the
//! single-write-site mechanism gate for the new `PlayerState` field.
//!
//! Every census below is walked over `all_cards()` and PRINTED by its own test
//! (PB-DX8's rule: publish the figure, do not transcribe it), never asserted as
//! an exact membership list except where the plan names a specific pin.
//!
//! **The declared axes use `decision_site_walk::def_contains_variant`, not
//! `format!("{:#?}", def)`.** Both walk the whole def by construction, so both
//! are immune to PB-DX26's `RollDice` lesson (a hand-written recursive walker
//! that enumerates nesting sites by hand can miss one). They differ on the
//! OTHER axis, and the first draft of this file was wrong on it: a `Debug`
//! render includes PROSE — a `Completeness` note, an `oracle_text`, a card
//! `name` — so a def that merely *names* a variant in a blocker note is
//! indistinguishable from a def that DECLARES it.
//!
//! That is not hypothetical here. `scourge_of_the_throne`'s
//! `Completeness::partial("... Effect::UntapAll{is_attacking},
//! Effect::AdditionalCombatPhase. ...")` is a compiled string literal, so the
//! Debug walk counted it and R3's population read **5**. The real declared
//! population is **4**. And R1 is one blocker note away from the same false
//! positive, which is not a remote risk for this batch in particular: the card
//! it repaired, `minas_tirith`, carried a note naming a `Condition` variant by
//! identifier — that is exactly what blocker notes do.
//!
//! `def_contains_variant` suppresses bare strings sitting under a `PROSE_FIELDS`
//! key, and that list already carries `"Inert"` / `"Partial"` / `"KnownWrong"`
//! (the `Completeness` variant keys) precisely for this. *A census walk has two
//! axes — how exhaustively it reaches, and whether what it reaches is code or
//! prose — and defending one of them says nothing about the other*
//! (`OOS-DX36-8`, one axis over).

use std::path::{Path, PathBuf};

use mtg_engine::all_cards;

use crate::decision_site_walk::def_contains_variant;

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p
}

// ─────────────────────────────────────────────────────────────────────────────
// R1 — declared axis: which defs reference either Condition variant
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// R1: every `all_cards()` def whose Debug-formatted ability tree mentions
/// `Condition::YouAttackedWithNOrMoreThisDeclaration` (CR 508.3d,
/// per-declaration) or `Condition::YouAttackedWithNOrMoreCreaturesThisTurn`
/// (ruling 2007-10-01, per-turn). Printed, and pinned to the two members this
/// batch itself created plus the one pre-existing per-declaration member
/// (`legions_landing`) -- so a THIRD future member joining either bucket is a
/// visible test change, not a silent drift.
fn r1_declared_axis_census() {
    let cards = all_cards();
    assert!(
        cards.len() >= 1000,
        "PB-DX53 r1: all_cards() returned suspiciously few defs ({}) -- non-vacuity check \
         failed",
        cards.len()
    );

    let mut per_declaration: Vec<String> = Vec::new();
    let mut per_turn: Vec<String> = Vec::new();

    for def in &cards {
        if def_contains_variant(def, "YouAttackedWithNOrMoreThisDeclaration") {
            per_declaration.push(def.name.clone());
        }
        if def_contains_variant(def, "YouAttackedWithNOrMoreCreaturesThisTurn") {
            per_turn.push(def.name.clone());
        }
        // Non-vacuity for the RENAME itself: nothing in the live corpus should
        // still spell the OLD, retired variant name after this batch. Checked
        // through the same prose-suppressed walk, so a blocker note quoting the
        // historical name is correctly NOT a failure.
        assert!(
            !def_contains_variant(def, "YouAttackedWithNOrMore"),
            "PB-DX53 r1: {} still references the RETIRED `Condition::YouAttackedWithNOrMore` \
             (renamed to `YouAttackedWithNOrMoreThisDeclaration`) -- update the def",
            def.name
        );
    }
    per_declaration.sort();
    per_turn.sort();

    println!(
        "PB-DX53 r1 declared-axis census: per-declaration = {per_declaration:?}, per-turn = \
         {per_turn:?}"
    );

    assert_eq!(
        per_declaration,
        vec!["Legion's Landing".to_string()],
        "PB-DX53 r1: the per-declaration (CR 508.3d) bucket must be exactly Legion's Landing \
         -- if a new member joined, that is real news, not a stale pin to bump silently"
    );
    assert_eq!(
        per_turn,
        vec!["Minas Tirith".to_string(), "Windbrisk Heights".to_string()],
        "PB-DX53 r1: the per-turn (ruling 2007-10-01) bucket must be exactly the two members \
         this batch authored"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// R2 — inverse oracle axis: printed attack-scope vocabulary vs declared axis
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// R2: every def whose PRINTED oracle text (front face and every `CardFace`,
/// PB-DX8's lesson) contains "attacked with" -- the shared vocabulary of BOTH
/// CR 508.3d and ruling 2007-10-01's printed wording -- classified by whether
/// it ALSO says "this turn" nearby (a proxy for the per-turn scope) or not (a
/// proxy for the per-declaration scope). Printed rather than asserted exactly,
/// because natural-language classification is not exhaustive by construction
/// the way R1's Debug scan is -- this is a FLOOR-finding census, not a gate.
fn r2_inverse_oracle_axis_census() {
    let cards = all_cards();
    // (name, has_this_turn, is_declared, scope)
    let mut hits: Vec<(String, bool, bool, &'static str)> = Vec::new();

    for def in &cards {
        let mut faces: Vec<&str> = vec![def.oracle_text.as_str()];
        if let Some(back) = &def.back_face {
            faces.push(back.oracle_text.as_str());
        }
        let lower_joined = faces.join("\n").to_lowercase();
        if !lower_joined.contains("attacked with") {
            continue;
        }
        // Two of the plan's four printed scopes share the "attacked with"
        // phrase but are NOT this batch's subject at all: Pack tactics / Melee
        // gate on TOTAL POWER of attacking creatures, not a CREATURE COUNT
        // (CR 702.111a Melee, the "Pack tactics" ability word) -- a genuinely
        // different primitive, out of scope for `Condition::
        // YouAttackedWithNOrMore{ThisDeclaration,CreaturesThisTurn}` and
        // correctly undeclared by either.
        let is_power_family = lower_joined.contains("total power");
        let has_this_turn = lower_joined.contains("this turn");
        let text = format!("{def:#?}");
        let is_declared = text.contains("YouAttackedWithNOrMoreThisDeclaration")
            || text.contains("YouAttackedWithNOrMoreCreaturesThisTurn");
        let scope = if is_power_family {
            "per-combat-power (Melee/Pack-tactics family, OUT OF SCOPE)"
        } else if has_this_turn {
            "per-turn-count"
        } else {
            "per-declaration-count"
        };
        hits.push((def.name.clone(), has_this_turn, is_declared, scope));
    }
    hits.sort();

    println!(
        "PB-DX53 r2 inverse-oracle-axis census ({} members): {hits:#?}",
        hits.len()
    );

    // Legion's Landing prints "attack WITH three or more creatures" (present
    // tense, CR 508.3d trigger phrasing), not "attacKED with" -- so it is
    // correctly ABSENT from this bucket; R1's Debug scan is what finds it. The
    // three found here are the two per-turn-count members (ruling 2007-10-01)
    // plus one Melee/Pack-tactics power-family member (genuinely out of scope,
    // proving the exclusion isn't vacuous).
    assert!(
        hits.len() >= 3,
        "PB-DX53 r2: expected at least the 3 known members (Windbrisk Heights, Minas Tirith, \
         and at least one Melee/Pack-tactics power-family member), found {}: {hits:?}",
        hits.len()
    );

    // Every CREATURE-COUNT member (not the power family) must be on the
    // declared axis -- an oracle-side member with no corresponding declared-axis
    // entry names an exact missing identifier rather than a vague "some cards
    // are unhandled".
    let undeclared: Vec<&(String, bool, bool, &str)> = hits
        .iter()
        .filter(|(_, _, declared, scope)| !declared && !scope.contains("OUT OF SCOPE"))
        .collect();
    assert!(
        undeclared.is_empty(),
        "PB-DX53 r2: these defs print 'attacked with' (creature-count family, not \
         Melee/Pack-tactics) but declare NEITHER Condition::YouAttackedWithNOrMoreThisDeclaration \
         NOR ...CreaturesThisTurn -- name the missing identifier per def rather than leaving \
         this floor to rot: {undeclared:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// R3 — extra-combat axis: Effect::AdditionalCombatPhase declarers
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// R3 (§9.2 of the plan): every `all_cards()` def whose Debug-formatted ability
/// tree contains `AdditionalCombatPhase`, with its `Completeness`.
///
/// **The plan's own §9.2 arithmetic does NOT reproduce, and the correction is
/// reported rather than silently absorbed** (the brief is a claim like any
/// other). §9.2 says: "`grep` finds 8 files; `windbrisk_heights.rs` only
/// MENTIONS the variant in a comment; so the declared population is 7." That
/// arithmetic is itself SR-36's failure mode one level deeper: `grep -rl
/// AdditionalCombatPhase` finds **8** files at the merge base, but TWO more of
/// them -- `breath_of_fury.rs` (a TODO placeholder whose comment explicitly
/// says the primitive is "exercised by OTHER cards") and
/// `moraug_fury_of_akoum.rs` (whose comment says verbatim "no
/// `Effect::AdditionalCombatPhase` ... exists" for it) -- ALSO only mention the
/// variant in prose, never declare it. The true declared population is **5**,
/// not 7: `grep`'s 8 minus THREE comment-only mentions (windbrisk_heights,
/// breath_of_fury, moraug_fury_of_akoum), not one.
fn r3_extra_combat_axis_census() {
    let cards = all_cards();
    let mut declarers: Vec<(String, &'static str)> = Vec::new();
    for def in &cards {
        if def_contains_variant(def, "AdditionalCombatPhase") {
            declarers.push((def.name.clone(), def.completeness.kind()));
        }
    }
    declarers.sort();
    println!(
        "PB-DX53 r3 extra-combat-axis census ({} members): {declarers:#?}",
        declarers.len()
    );

    assert_eq!(
        declarers.len(),
        4,
        "PB-DX53 r3: expected exactly 4 AdditionalCombatPhase declarers (re-derived directly \
         against all_cards() through the PROSE-SUPPRESSED walk -- see this test's doc for why \
         a Debug render reads 5 and why that fifth is scourge_of_the_throne's completeness \
         NOTE rather than a declaration), found {}: {declarers:?}",
        declarers.len()
    );

    let names: Vec<&str> = declarers.iter().map(|(n, _)| n.as_str()).collect();
    assert!(
        names.contains(&"Aggravated Assault"),
        "PB-DX53 r3: Aggravated Assault (the card driving the c1/c2 channel probes) must be \
         a member: {names:?}"
    );
    // Every def that MENTIONS the variant without DECLARING it. The first three
    // mention it in a `//` comment (invisible to any walk over the compiled def);
    // `scourge_of_the_throne` mentions it inside its `Completeness::partial`
    // note, which IS compiled in -- so it is the one that discriminates the
    // prose-suppressed walk from a Debug render, and the reason it is listed
    // here rather than left to the count.
    for absent in [
        "Windbrisk Heights",
        "Breath of Fury",
        "Moraug, Fury of Akoum",
        "Scourge of the Throne",
    ] {
        assert!(
            !names.contains(&absent),
            "PB-DX53 r3: {absent} mentions AdditionalCombatPhase only in a comment and must \
             NOT be a declared-axis member: {names:?}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Mechanism gate — the new PlayerState field has exactly three legitimate
// write sites: the declaration, the turn reset, and the initial construction.
// ─────────────────────────────────────────────────────────────────────────────

fn is_test_only_file(path: &Path) -> bool {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|stem| {
            let lower = stem.to_ascii_lowercase();
            lower == "tests"
                || lower == "test"
                || lower.ends_with("_tests")
                || lower.ends_with("_test")
        })
        .unwrap_or(false)
}

fn walk_rs(dir: &Path, acc: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            walk_rs(&path, acc);
        } else if path.extension().is_some_and(|x| x == "rs") && !is_test_only_file(&path) {
            acc.push(path);
        }
    }
}

fn workspace_src_files() -> Vec<PathBuf> {
    let root = workspace_root();
    let mut out = Vec::new();
    for base in ["crates", "tools"] {
        let Ok(entries) = std::fs::read_dir(root.join(base)) else {
            continue;
        };
        let mut dirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        dirs.sort();
        for dir in dirs {
            let src = dir.join("src");
            if src.is_dir() {
                walk_rs(&src, &mut out);
            }
        }
    }
    out.sort();
    out
}

fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|line| line.find("//").map(|i| &line[..i]).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The three files legitimately allowed to mutate
/// `creatures_declared_as_attackers_this_turn`, and the mechanism-specific
/// needle that must appear in each -- keyed on the MECHANISM (declare / reset /
/// construct), not merely on the file, per `OOS-DX47`'s "an allowlist whose
/// reason is never checked is a comment" and `OOS-DX51`'s "a file-scoped
/// allowlist grants an exemption far wider than the reason that earned it".
const ALLOWED_WRITE_SITES: [(&str, &str); 3] = [
    (
        "crates/engine/src/rules/combat.rs",
        "creatures_declared_as_attackers_this_turn\n                .insert(",
    ),
    (
        "crates/engine/src/rules/turn_actions.rs",
        "creatures_declared_as_attackers_this_turn = imbl::OrdSet::new()",
    ),
    (
        "crates/engine/src/state/builder.rs",
        "creatures_declared_as_attackers_this_turn: imbl::OrdSet::new()",
    ),
];

#[test]
/// The mechanism gate: no production file other than the three
/// [`ALLOWED_WRITE_SITES`] mutates `creatures_declared_as_attackers_this_turn`
/// at all -- an `.insert(`, a whole-field assignment, or a struct-literal
/// construction. A fourth write site would either duplicate the CR 400.7 dedup
/// logic (drift risk) or bypass it (a silent double-count).
fn mechanism_gate_single_write_site() {
    let field = "creatures_declared_as_attackers_this_turn";
    let files = workspace_src_files();
    assert!(
        files.len() >= 500,
        "PB-DX53 mechanism gate: workspace source walk found only {} files -- non-vacuity \
         check failed",
        files.len()
    );

    let root = workspace_root();
    let mut offenders: Vec<(String, String)> = Vec::new();
    let mut confirmed_allowed: Vec<&str> = Vec::new();

    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else {
            continue;
        };
        let stripped = strip_line_comments(&src);
        if !stripped.contains(field) {
            continue;
        }
        let rel = f.strip_prefix(&root).unwrap_or(f).display().to_string();
        let joined: String = stripped.split_whitespace().collect::<Vec<_>>().join(" ");

        // The insert/assign checks use the WHITESPACE-JOINED blob (`joined`),
        // deliberately, because the one real `.insert(` call site in this
        // codebase (`combat.rs`) is rustfmt-wrapped across two lines -- a
        // per-line-only scan would be blind to it (`OOS-DX51`'s multi-line
        // lesson, one primitive over). The colon (construction-vs-declaration)
        // check is per-line: every occurrence of that FORM in this codebase
        // today is single-line, and per-line is what lets a TYPE (the struct
        // field's own declaration) be told apart from a VALUE (a struct
        // literal constructing one) at all -- the joined blob cannot
        // distinguish them once whitespace is collapsed.
        // Both the single-line spelling and the rustfmt-wrapped one (a space
        // where the line break was, after whitespace-collapsing `joined`).
        let is_insert = joined.contains(&format!("{field}.insert("))
            || joined.contains(&format!("{field} .insert("));
        let is_assign = {
            let mut found = false;
            let mut from = 0usize;
            while let Some(i) = joined[from..].find(field) {
                let at = from + i;
                let rest = joined[at + field.len()..].trim_start();
                if rest.starts_with('=') && !rest.starts_with("==") {
                    found = true;
                    break;
                }
                from = at + field.len();
            }
            found
        };
        let mut literal_construction_lines: Vec<&str> = Vec::new();
        for raw_line in stripped.lines() {
            let line = raw_line.trim();
            let Some(i) = line.find(&format!("{field}:")) else {
                continue;
            };
            let rest = line[i + field.len() + 1..].trim_start();
            // A construction's value is an EXPRESSION (starts with a
            // call/path like `imbl::OrdSet::new()`); a DECLARATION's value is
            // a TYPE (starts with the bare type name, `OrdSet<...>,`, as
            // `player.rs`'s own struct field spells it). `imbl::` vs `OrdSet<`
            // is the discriminator actually present in this codebase today.
            // `OOS-DX51-6`'s lesson: this gate's first draft conflated the two
            // and was defeated by execution on `player.rs`'s own field
            // declaration, caught by re-running the gate rather than assumed
            // correct on the first pass.
            if rest.starts_with("imbl::") {
                literal_construction_lines.push(line);
            }
        }

        if !is_insert && !is_assign && literal_construction_lines.is_empty() {
            continue; // read-only reference: `.len()`, `.contains(`, a doc comment, hash.rs's iterate, the struct's own declaration
        }

        let matched_allowed = ALLOWED_WRITE_SITES.iter().find(|(file, needle)| {
            rel.ends_with(file)
                && joined.contains(&needle.split_whitespace().collect::<Vec<_>>().join(" "))
        });
        match matched_allowed {
            Some((file, _)) => confirmed_allowed.push(file),
            None => offenders.push((
                rel.clone(),
                format!(
                    "insert={is_insert} assign={is_assign} literal_construction_lines={literal_construction_lines:?}"
                ),
            )),
        }
    }

    assert!(
        offenders.is_empty(),
        "PB-DX53 mechanism gate: found a mutating reference to `{field}` outside the three \
         allowlisted sites: {offenders:#?}"
    );
    for (file, _) in ALLOWED_WRITE_SITES {
        assert!(
            confirmed_allowed.contains(&file),
            "PB-DX53 mechanism gate: allowlisted site {file} no longer matches its stated \
             needle -- either the site moved (re-key the entry) or it is gone (remove it, a \
             dead entry is slack a real offender hides in)"
        );
    }
}
