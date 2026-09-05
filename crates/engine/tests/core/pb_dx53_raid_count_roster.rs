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
//! **The mechanism that separates them is EXACT matching, not the
//! `PROSE_FIELDS` denylist**, and the first draft of this paragraph said
//! otherwise. `decision_site_walk`'s string arm fires only when a string is
//! EQUAL to the variant name; `Completeness::partial("...
//! Effect::AdditionalCombatPhase. ...")` is a sentence, never equal to
//! `"AdditionalCombatPhase"`, so `PROSE_FIELDS` is never consulted on this
//! input and contributes nothing to the result. `PROSE_FIELDS` defends the
//! narrower case of a note whose ENTIRE text is a variant name. The reason
//! matters because a later batch "hardening" one of these censuses by adding
//! a key to `PROSE_FIELDS` would be doing nothing at all — *a reason is the
//! half the next batch reuses* (`OOS-DX49`).
//!
//! *A census walk has two axes — how exhaustively it reaches, and whether what
//! it reaches is code or prose — and defending one of them says nothing about
//! the other* (`OOS-DX36-8`, one axis over).

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
        // The RENAME residual -- stated honestly rather than sold as a
        // non-vacuity check, which is what the first draft called it. The old
        // variant no longer exists in the enum and `def_contains_variant`
        // matches keys and whole string values EXACTLY, so the compiler
        // already guarantees this and the assertion cannot fail today. It is
        // kept as a tripwire for the one case the compiler does not cover: a
        // future def carrying the retired name as a whole-string value (a
        // note that is nothing but the identifier). That narrow reason is
        // written down instead of the broad claim.
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
        // phrase but are NOT this batch's subject at all, and neither gates on
        // a CREATURE COUNT. The "Pack tactics" ability word gates on the TOTAL
        // POWER of the attacking creatures; Melee (CR 702.121a) scales with
        // the number of OPPONENTS -- "it gets +1/+1 until end of turn for each
        // opponent you attacked with a creature this combat". Both are
        // genuinely different primitives, out of scope for
        // `Condition::YouAttackedWithNOrMore{ThisDeclaration,
        // CreaturesThisTurn}` and correctly undeclared by either.
        //
        // The filter keys on the literal "total power", so it selects the
        // Pack-tactics family. The Melee cite is spelled out because the first
        // draft of this comment attributed the total-power gate to Melee AND
        // numbered it CR 702.111a, which is Menace -- a plausible-sounding
        // justification for a filter that is correct for a different reason.
        let is_power_family = lower_joined.contains("total power");
        let has_this_turn = lower_joined.contains("this turn");
        // Prose-suppressed, like R1 and R3 -- NOT `format!("{def:#?}")`. The
        // first draft used the Debug render here and the `/review` defeated it
        // by execution: giving a def a printed "attacked with ... this turn"
        // line and a `Completeness::partial("blocked: needs
        // Condition::YouAttackedWithNOrMoreCreaturesThisTurn(3)")` note made
        // `is_declared` come back TRUE **from the note**, and the `undeclared`
        // assertion below -- the only thing in this file whose job is to find
        // an undeclared printed member -- came back EMPTY with all four roster
        // tests green. This module's own doc had named that exact risk for R1
        // and then left it standing in R2, which is the test that found
        // `minas_tirith` in the first place: a blocker note naming the missing
        // identifier is precisely what an undeclared member looks like, so
        // scoring the note as a declaration blinds this axis to its own
        // subject matter.
        let is_declared = def_contains_variant(def, "YouAttackedWithNOrMoreThisDeclaration")
            || def_contains_variant(def, "YouAttackedWithNOrMoreCreaturesThisTurn");
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
/// variant in prose, never declare it. And a FOURTH,
/// `scourge_of_the_throne.rs`, names it inside a COMPILED
/// `Completeness::partial` note rather than a `//` comment -- which is why
/// this census walks `def_contains_variant` and not `format!("{def:#?}")`,
/// and why that def is pinned in the must-be-absent list below.
///
/// The true declared population is **4**: `grep`'s 8 minus FOUR non-declaring
/// mentions, not one. (The first draft of this doc said **5** while the
/// assertion below said 4 and named four absentees -- a test doc
/// contradicting its own test, inside a batch whose subject is a false
/// comment. `memory/primitives/pb-DX53-execution-notes.md` had it right;
/// this was the stale copy.)
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
/// `creatures_declared_as_attackers_this_turn`, the mechanism-specific needle
/// that must appear in each, and the EXACT number of mutating references each
/// is allowed to hold.
///
/// The count is the load-bearing third column, and it is here because the
/// `/review` defeated the first draft by execution: that draft matched
/// `rel.ends_with(file) && joined.contains(needle)`, a PRESENCE check, so a
/// second `.insert(` planted beside the real one in `combat.rs` left the gate
/// green. That is `OOS-DX48`'s r1 defeat ("a duplicated call inside a marked
/// site collapses into one element") and `OOS-DX51`'s "a file-scoped allowlist
/// grants an exemption far wider than the reason that earned it" -- both cited
/// in this file's own justification while the code did the thing they warn
/// against. A duplicated insert is not hypothetical for this field: inserting
/// twice per declaration is exactly the double-count the dedup exists to stop.
const ALLOWED_WRITE_SITES: [(&str, &str, usize); 3] = [
    (
        "crates/engine/src/rules/combat.rs",
        "creatures_declared_as_attackers_this_turn\n                .insert(",
        1,
    ),
    (
        "crates/engine/src/rules/turn_actions.rs",
        "creatures_declared_as_attackers_this_turn = imbl::OrdSet::new()",
        1,
    ),
    (
        "crates/engine/src/state/builder.rs",
        "creatures_declared_as_attackers_this_turn: imbl::OrdSet::new()",
        1,
    ),
];

/// Methods that only READ an `OrdSet`. Everything else reachable through a `.`
/// on the field is treated as mutating.
///
/// **The list is deliberately the READ side, so the classifier fails CLOSED.**
/// The first draft enumerated the MUTATING forms (`.insert(`, `= `, `: `) and
/// was defeated by execution with `let set = &mut ps.field; set.insert(id);` --
/// `OOS-DX51-6` verbatim (`let map = &mut combat.attackers; map.insert(..)`),
/// whose published remedy is "re-key on the MECHANISM -- all four ways to
/// obtain a mutable path to the map, on ANY receiver". Enumerating what may
/// mutate an `OrdSet` is unbounded; enumerating what provably does not is
/// four names, and an unknown method is an offender rather than a pass.
const READ_ONLY_METHODS: [&str; 8] = [
    "len", "contains", "iter", "is_empty", "clone", "get", "keys", "values",
];

/// How one textual reference to the field uses it.
#[derive(Debug, PartialEq, Eq)]
enum FieldUse {
    /// `&mut <path>.field` -- a mutable borrow, whatever is done with it later.
    MutBorrow,
    /// `<path>.field.<method>(` where the method is not in [`READ_ONLY_METHODS`].
    MutMethod(String),
    /// `<path>.field = ...` -- whole-field assignment.
    Assign,
    /// `field: <expression>` -- a struct-literal construction.
    Construct,
    /// `field: <Type>` -- the struct's own field declaration.
    Declaration,
    /// A read: `.len()`, `.contains(`, `for x in &self.field`, a bare mention.
    Read,
}

impl FieldUse {
    fn is_mutating(&self) -> bool {
        matches!(
            self,
            FieldUse::MutBorrow | FieldUse::MutMethod(_) | FieldUse::Assign | FieldUse::Construct
        )
    }
}

/// Classify every occurrence of `field` in `src` (comments already stripped).
///
/// Operates on the whitespace-JOINED blob, deliberately: the one real
/// `.insert(` call site is rustfmt-wrapped across two lines, so a per-line scan
/// is blind to it (`OOS-DX51`'s multi-line lesson). The construction-vs-
/// declaration split needs the per-line text and is passed it separately.
fn classify_field_uses(src: &str, field: &str) -> Vec<FieldUse> {
    let joined: String = src.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(i) = joined[from..].find(field) {
        let at = from + i;
        from = at + field.len();

        // Skip a match that is only part of a LONGER identifier, so a future
        // `creatures_declared_as_attackers_this_turn_v2` is its own occurrence
        // rather than silently riding on this one.
        let next_ch = joined[at + field.len()..].chars().next();
        if next_ch.is_some_and(|c| c.is_alphanumeric() || c == '_') {
            continue;
        }

        // Axis 1 -- what PRECEDES the receiver path. Walk back over the path
        // (`ps.`, `self.`, `state.players[p].`) and ask whether the whole thing
        // is being mutably borrowed. This is the axis the first draft lacked.
        let mut start = at;
        let bytes = joined.as_bytes();
        while start > 0 {
            let c = bytes[start - 1] as char;
            if c.is_alphanumeric() || c == '_' || c == '.' || c == ':' {
                start -= 1;
            } else {
                break;
            }
        }
        if joined[..start].trim_end().ends_with("&mut") {
            out.push(FieldUse::MutBorrow);
            continue;
        }

        // Axis 2 -- what FOLLOWS.
        let rest = joined[at + field.len()..].trim_start();
        if let Some(after_dot) = rest.strip_prefix('.') {
            let method: String = after_dot
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if READ_ONLY_METHODS.contains(&method.as_str()) {
                out.push(FieldUse::Read);
            } else {
                out.push(FieldUse::MutMethod(method));
            }
        } else if rest.starts_with('=') && !rest.starts_with("==") {
            out.push(FieldUse::Assign);
        } else if let Some(value) = rest.strip_prefix(':') {
            // A construction's value is an EXPRESSION and therefore contains a
            // call: `imbl::OrdSet::new()`, `OrdSet::new()`. A DECLARATION's
            // value is a TYPE and contains none: `OrdSet<ObjectId>,`. Keying on
            // the presence of `(` rather than on the literal `imbl::` (the
            // first draft's discriminator) means a construction spelled with an
            // imported `OrdSet::new()` is still caught.
            let value = value.trim_start();
            let head: String = value.chars().take_while(|c| *c != ',').collect();
            if head.contains('(') {
                out.push(FieldUse::Construct);
            } else {
                out.push(FieldUse::Declaration);
            }
        } else {
            out.push(FieldUse::Read);
        }
    }
    out
}

#[test]
/// The mechanism gate: no production file other than the three
/// [`ALLOWED_WRITE_SITES`] obtains a MUTATING path to
/// `creatures_declared_as_attackers_this_turn`, and each allowlisted file holds
/// EXACTLY the number of mutating references its entry states. A fourth write
/// site would either duplicate the CR 400.7 dedup logic (drift risk) or bypass
/// it (a silent double-count); a second write inside an allowlisted file is the
/// double-count directly.
///
/// Both halves exist because the `/review` defeated the first draft on each of
/// them by execution -- see [`ALLOWED_WRITE_SITES`] and [`READ_ONLY_METHODS`].
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
    let mut confirmed_allowed: Vec<(&str, usize)> = Vec::new();

    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else {
            continue;
        };
        let stripped = strip_line_comments(&src);
        if !stripped.contains(field) {
            continue;
        }
        let rel = f.strip_prefix(&root).unwrap_or(f).display().to_string();
        let uses = classify_field_uses(&stripped, field);
        let mutating: Vec<&FieldUse> = uses.iter().filter(|u| u.is_mutating()).collect();
        if mutating.is_empty() {
            continue; // read-only: `.len()`, `.contains(`, `for x in &self.field`, the struct's own declaration
        }

        let joined: String = stripped.split_whitespace().collect::<Vec<_>>().join(" ");
        let matched_allowed = ALLOWED_WRITE_SITES.iter().find(|(file, needle, _)| {
            rel.ends_with(file)
                && joined.contains(&needle.split_whitespace().collect::<Vec<_>>().join(" "))
        });
        match matched_allowed {
            Some((file, _, expected)) => {
                assert_eq!(
                    mutating.len(),
                    *expected,
                    "PB-DX53 mechanism gate: allowlisted site {file} holds {} mutating \
                     references to `{field}`, not the {expected} its entry allows: \
                     {mutating:?}. An allowlist entry exempts ONE stated mechanism, not the \
                     whole file -- a second write here is the double-count the CR 400.7 dedup \
                     exists to prevent",
                    mutating.len()
                );
                confirmed_allowed.push((file, mutating.len()));
            }
            None => offenders.push((rel.clone(), format!("{mutating:?}"))),
        }
    }

    assert!(
        offenders.is_empty(),
        "PB-DX53 mechanism gate: found a mutating path to `{field}` outside the three \
         allowlisted sites: {offenders:#?}"
    );
    for (file, _, expected) in ALLOWED_WRITE_SITES {
        let found = confirmed_allowed.iter().find(|(f, _)| *f == file);
        assert_eq!(
            found.map(|(_, n)| *n),
            Some(expected),
            "PB-DX53 mechanism gate: allowlisted site {file} no longer matches its stated \
             needle with {expected} mutating reference(s) -- either the site moved (re-key the \
             entry) or it is gone (remove it, a dead entry is slack a real offender hides in)"
        );
    }
}

#[test]
/// Non-vacuity for [`classify_field_uses`] itself: the classifier is the whole
/// gate, so its discrimination is asserted on synthetic input rather than
/// inferred from the gate passing. Each row is a form the `/review` either
/// defeated the first draft with, or that must stay a READ.
fn mechanism_gate_classifier_discriminates() {
    let f = "the_field";
    let cases: [(&str, FieldUse); 9] = [
        // The two forms that defeated the first draft by execution.
        (
            "let s = &mut ps.the_field; s.insert(id);",
            FieldUse::MutBorrow,
        ),
        (
            "ps.the_field.clear();",
            FieldUse::MutMethod("clear".to_string()),
        ),
        // The forms the first draft did catch.
        (
            "ps.the_field\n    .insert(x);",
            FieldUse::MutMethod("insert".to_string()),
        ),
        ("p.the_field = imbl::OrdSet::new();", FieldUse::Assign),
        (
            "PlayerState { the_field: imbl::OrdSet::new(), }",
            FieldUse::Construct,
        ),
        // A construction spelled WITHOUT the `imbl::` prefix the first draft
        // keyed on -- the reason the discriminator is `(` and not that path.
        (
            "PlayerState { the_field: OrdSet::new(), }",
            FieldUse::Construct,
        ),
        // Must stay non-mutating.
        ("pub the_field: OrdSet<ObjectId>,", FieldUse::Declaration),
        ("for x in &self.the_field {", FieldUse::Read),
        ("p.the_field.len() as u32 >= n", FieldUse::Read),
    ];
    for (src, expected) in cases {
        let got = classify_field_uses(src, f);
        assert_eq!(
            got.len(),
            1,
            "PB-DX53 classifier: {src:?} should yield exactly one occurrence, got {got:?}"
        );
        assert_eq!(
            got[0], expected,
            "PB-DX53 classifier: {src:?} classified as {:?}, expected {expected:?}",
            got[0]
        );
    }
    // A longer identifier sharing the prefix is its own symbol, not this one.
    assert!(
        classify_field_uses("ps.the_field_v2.insert(x);", f).is_empty(),
        "PB-DX53 classifier: a longer identifier must not be counted as this field"
    );
}
