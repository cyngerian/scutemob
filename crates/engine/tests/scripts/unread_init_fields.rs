//! SR-22(c): approved scripts may not carry a value in an `InitialState` field
//! that `build_initial_state` never reads.
//!
//! `script_schema.rs` declares several init fields the harness silently ignores.
//! A script that sets one is describing a board the engine never builds — the
//! same class of bug SR-9b found with `turn_number` (declared, never read, so
//! every script secretly ran on turn 1). The unread fields are:
//!
//! | Field | Why it is unread |
//! |---|---|
//! | `InitialState.step` | `build_initial_state` derives the step from `phase` (`parse_step`); `step` is never consulted. Always `null` in the corpus. |
//! | `InitialState.priority` | The engine computes the priority holder itself; this is a human-readable mirror. **Always populated** — allowlisted. |
//! | `InitialState.continuous_effects` | No loop reads it; pre-existing continuous effects are not reconstructed. |
//! | `ZonesInitState.command_zone` | Commanders are populated from `players.<p>.commander`; this zone map is not iterated. |
//! | `PermanentInitState.summoning_sick` | `build_initial_state` sets tapped/counters/damage but never summoning sickness. |
//! | `PlayerInitState.commander_damage_received` | Life/mana/land-plays/poison are patched; this map is not. |
//!
//! The gate is **default-empty**: an unread field must be at its default in every
//! approved script, unless it is on [`ALWAYS_POPULATED_UNREAD_FIELDS`] (only
//! `priority`, which is structurally always set). `continuous_effects`,
//! `command_zone`, `commander_damage_received`, `summoning_sick`, and `step` are
//! empty/absent across today's corpus, so the gate passes now and fails the day a
//! script starts lying about one of them.
//!
//! If `build_initial_state` is ever wired to *read* one of these (as SR-9b did for
//! `turn_number`), remove it from [`UNREAD_INIT_FIELDS`] — it is no longer unread.
//! `unread_field_detectors_are_not_vacuous` proves each detector actually fires,
//! so the gate cannot rot into a rubber stamp.
//!
//! # `OOS-DX28-1` (PB-DX57): the six above are not all of them, and nothing had asked
//!
//! The table above was a hand-maintained list with nothing comparing it to
//! `script_schema.rs`'s declarations. `unread_field_detectors_are_not_vacuous` proves the six
//! detectors FIRE; **no test asked whether there was a SEVENTH unread field.** There are
//! nine, and they are listed with a reason each in [`UNREAD_WITHOUT_DETECTOR`] — among them
//! `CommanderInitState.times_cast_from_command_zone` (CR 903.8's commander tax, silently
//! dropped) and `PermanentInitState.attached` (which `scripts::harness_equivalence`'s own
//! module-doc table ALREADY listed as unread, in this same test binary, while this file's
//! list did not).
//!
//! [`every_declared_init_field_is_read_or_classified_unread`] is the pin: every field the
//! six-struct `InitialState` family declares is READ by `build_initial_state`, unread WITH a
//! detector, or unread WITHOUT one — and a new field that is none of those three is a red
//! test rather than a silent SR-9b.
//!
//! The nine are recorded rather than given detectors, and the cost is stated at the list. A
//! detector puts a field under `approved_scripts_do_not_populate_unread_init_fields`, which
//! requires it to be at its DEFAULT in every approved script; the row prints the measured
//! per-field corpus population so the next batch can see which of the nine could take one
//! without a corpus migration.

use mtg_engine::testing::script_schema::{InitialState, ReviewStatus};

/// One declared-but-unread init field: its name and a detector that returns true
/// when an `InitialState` carries a non-default value for it.
struct UnreadField {
    name: &'static str,
    is_populated: fn(&InitialState) -> bool,
}

const UNREAD_INIT_FIELDS: &[UnreadField] = &[
    UnreadField {
        name: "step",
        is_populated: |i| i.step.is_some(),
    },
    UnreadField {
        name: "priority",
        // Required `String`; "populated" == non-empty. Structurally always set.
        is_populated: |i| !i.priority.is_empty(),
    },
    UnreadField {
        name: "continuous_effects",
        is_populated: |i| !i.continuous_effects.is_empty(),
    },
    UnreadField {
        name: "command_zone",
        is_populated: |i| !i.zones.command_zone.is_empty(),
    },
    UnreadField {
        name: "summoning_sick",
        is_populated: |i| {
            i.zones
                .battlefield
                .values()
                .flatten()
                .any(|p| p.summoning_sick)
        },
    },
    UnreadField {
        name: "commander_damage_received",
        is_populated: |i| {
            i.players
                .values()
                .any(|p| !p.commander_damage_received.is_empty())
        },
    },
];

/// Unread fields that are *structurally* always populated and whose value is
/// documentation only, so requiring them empty is impossible/pointless. Guarded
/// by [`unread_field_allowlist_has_no_dead_entries`].
const ALWAYS_POPULATED_UNREAD_FIELDS: &[&str] = &["priority"];

fn approved_initial_states() -> Vec<(String, InitialState)> {
    crate::run_all_scripts::discover_scripts(std::path::Path::new(
        crate::run_all_scripts::SCRIPTS_DIR,
    ))
    .into_iter()
    .filter_map(|(label, parsed)| {
        let script = parsed.ok()?;
        (script.metadata.review_status == ReviewStatus::Approved)
            .then_some((label, script.initial_state))
    })
    .collect()
}

#[test]
/// No approved script populates an unread init field (except the allowlisted
/// `priority`). A populated unread field is a silent divergence: the harness
/// ignores it, so the board the engine builds is not the board the script claims.
fn approved_scripts_do_not_populate_unread_init_fields() {
    let allow: std::collections::HashSet<&str> =
        ALWAYS_POPULATED_UNREAD_FIELDS.iter().copied().collect();

    let approved = approved_initial_states();
    assert!(
        !approved.is_empty(),
        "found no approved scripts — the walk is broken and this test proves nothing"
    );

    let mut offenders: Vec<String> = Vec::new();
    for (label, init) in &approved {
        for field in UNREAD_INIT_FIELDS {
            if allow.contains(field.name) {
                continue;
            }
            if (field.is_populated)(init) {
                offenders.push(format!("  {label}: sets unread field `{}`", field.name));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "{} approved script(s) set an init field that `build_initial_state` ignores. \
         The harness builds a different board than the script describes (cf. SR-9b's \
         `turn_number`). Remove the value, or wire the field into `build_initial_state` \
         and drop it from UNREAD_INIT_FIELDS:\n{}",
        offenders.len(),
        offenders.join("\n")
    );
}

#[test]
/// Every allowlist entry is actually populated by some approved script. An
/// entry that no script hits is a rubber stamp — the field became empty, or was
/// renamed, and the exemption now masks nothing (SR-8/SR-9b denominator rule).
fn unread_field_allowlist_has_no_dead_entries() {
    let approved = approved_initial_states();
    let field_by_name = |name: &str| UNREAD_INIT_FIELDS.iter().find(|f| f.name == name);

    for entry in ALWAYS_POPULATED_UNREAD_FIELDS {
        let field = field_by_name(entry).unwrap_or_else(|| {
            panic!("allowlist entry `{entry}` is not an UNREAD_INIT_FIELDS name")
        });
        let hit = approved.iter().any(|(_, init)| (field.is_populated)(init));
        assert!(
            hit,
            "allowlist entry `{entry}` is populated by no approved script — remove it"
        );
    }
}

#[test]
/// Each detector actually fires on a value — otherwise the gate above is vacuous
/// (a detector that always returns false would let any script through). Build one
/// `InitialState` that populates every unread field and assert each detector
/// catches its own.
fn unread_field_detectors_are_not_vacuous() {
    use mtg_engine::testing::script_schema::{
        ContinuousEffectInitState, PermanentInitState, PlayerInitState, ZonesInitState,
    };
    use std::collections::HashMap;

    let mut players = HashMap::new();
    let mut cdr = HashMap::new();
    cdr.insert("p2".to_string(), 3);
    players.insert(
        "p1".to_string(),
        PlayerInitState {
            life: 40,
            mana_pool: HashMap::new(),
            land_plays_remaining: 0,
            poison_counters: 0,
            commander_damage_received: cdr,
            commander: None,
            partner_commander: None,
        },
    );

    let mut battlefield = HashMap::new();
    battlefield.insert(
        "p1".to_string(),
        vec![PermanentInitState {
            card: "Llanowar Elves".to_string(),
            tapped: false,
            summoning_sick: true,
            counters: HashMap::new(),
            attached: vec![],
            damage_marked: 0,
            is_commander: false,
            subtypes: None,
            is_basic: None,
        }],
    );

    let mut command_zone = HashMap::new();
    command_zone.insert("p1".to_string(), vec![]);

    let init = InitialState {
        format: "commander".to_string(),
        turn_number: 1,
        active_player: "p1".to_string(),
        phase: "precombat_main".to_string(),
        step: Some("precombat_main".to_string()),
        priority: "p1".to_string(),
        players,
        zones: ZonesInitState {
            battlefield,
            hand: HashMap::new(),
            graveyard: HashMap::new(),
            exile: vec![],
            command_zone,
            library: HashMap::new(),
            stack: vec![],
        },
        continuous_effects: vec![ContinuousEffectInitState {
            source: "x".to_string(),
            effect: "y".to_string(),
            layer: 6,
            timestamp: 0,
            duration: "end_of_turn".to_string(),
        }],
    };

    for field in UNREAD_INIT_FIELDS {
        assert!(
            (field.is_populated)(&init),
            "detector for `{}` did not fire on a populated InitialState — the gate is vacuous",
            field.name
        );
    }

    // And it must go quiet on a fully-default state, or it would flag everything.
    let empty = InitialState {
        step: None,
        priority: "p1".to_string(),
        players: HashMap::new(),
        zones: ZonesInitState {
            battlefield: HashMap::new(),
            hand: HashMap::new(),
            graveyard: HashMap::new(),
            exile: vec![],
            command_zone: HashMap::new(),
            library: HashMap::new(),
            stack: vec![],
        },
        continuous_effects: vec![],
        ..init.clone()
    };
    for field in UNREAD_INIT_FIELDS {
        let fired = (field.is_populated)(&empty);
        if field.name == "priority" {
            assert!(fired, "priority is always populated");
        } else {
            assert!(
                !fired,
                "detector for `{}` fired on a default InitialState — false positive",
                field.name
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// The total classification of the `InitialState` family (`OOS-DX28-1`)
// ─────────────────────────────────────────────────────────────────────────────

/// The schema structs this row classifies, in `script_schema.rs`.
const SCHEMA_STRUCTS: &[&str] = &[
    "InitialState",
    "PlayerInitState",
    "CommanderInitState",
    "ZonesInitState",
    "PermanentInitState",
    "CardInZone",
];

/// Schema structs deliberately left OUT of the classification, with the reason and the
/// condition under which the exclusion expires.
const EXCLUDED_SCHEMA_STRUCTS: &[(&str, &str, &str)] = &[(
    "ContinuousEffectInitState",
    "continuous_effects",
    "Every one of its five fields is unread because its PARENT -- `InitialState.\
     continuous_effects` -- is unread and detector-enforced empty: no loop in \
     `build_initial_state` reconstructs a pre-existing continuous effect. Listing the five \
     individually would add five entries all saying the same thing. The exclusion is CHECKED \
     rather than asserted: this row requires the named parent field to be classified unread, \
     so if `continuous_effects` is ever wired in, the exclusion expires loudly instead of \
     hiding five newly-relevant fields.",
)];

/// `(struct, field)` pairs `build_initial_state` reads.
///
/// Hand-classified, and CORROBORATED (not derived) by a token scan of the function body --
/// see [`every_declared_init_field_is_read_or_classified_unread`] for why the derivation runs
/// in that direction and not the other.
const READ_BY_BUILD_INITIAL_STATE: &[(&str, &str)] = &[
    ("InitialState", "turn_number"),
    ("InitialState", "active_player"),
    ("InitialState", "phase"),
    ("InitialState", "players"),
    ("InitialState", "zones"),
    ("PlayerInitState", "life"),
    ("PlayerInitState", "mana_pool"),
    ("PlayerInitState", "land_plays_remaining"),
    ("PlayerInitState", "poison_counters"),
    ("PlayerInitState", "commander"),
    ("PlayerInitState", "partner_commander"),
    ("CommanderInitState", "card"),
    ("ZonesInitState", "battlefield"),
    ("ZonesInitState", "hand"),
    ("ZonesInitState", "graveyard"),
    ("ZonesInitState", "exile"),
    ("ZonesInitState", "library"),
    ("PermanentInitState", "card"),
    ("PermanentInitState", "tapped"),
    ("PermanentInitState", "counters"),
    ("PermanentInitState", "damage_marked"),
    ("CardInZone", "card"),
    ("CardInZone", "owner"),
    ("CardInZone", "is_suspended"),
    ("CardInZone", "counters"),
];

/// Which struct each [`UNREAD_INIT_FIELDS`] entry belongs to. That list keys by BARE field
/// name because its detectors are closures over `&InitialState`; the classification below
/// needs the owner to compare against the declaration.
const DETECTOR_FIELD_OWNERS: &[(&str, &str)] = &[
    ("InitialState", "step"),
    ("InitialState", "priority"),
    ("InitialState", "continuous_effects"),
    ("ZonesInitState", "command_zone"),
    ("PermanentInitState", "summoning_sick"),
    ("PlayerInitState", "commander_damage_received"),
];

/// **Declared, unread by `build_initial_state`, and carrying NO detector** -- so a script may
/// populate any of these and the harness will silently build a different board.
///
/// This list is `OOS-DX28-1`'s yield on this member and it is the reason the row exists.
/// `UNREAD_INIT_FIELDS` was a hand-maintained list of SIX with no pin, and its own module doc
/// says the class it guards is *"SR-9b's `turn_number` defect exactly"*.
/// `unread_field_detectors_are_not_vacuous` proves the six existing detectors FIRE; **nothing
/// asked whether there was a seventh unread field.** There are nine.
///
/// They are recorded rather than given detectors, deliberately and with the cost stated: a
/// detector puts the field under `approved_scripts_do_not_populate_unread_init_fields`, which
/// requires it to be at its default in EVERY approved script. Several of these are populated
/// by the corpus today (the row prints the measured count for each), so adding detectors is a
/// corpus migration, not a gate fix -- the same call `script_schema.rs` itself makes about
/// `deny_unknown_fields` on `PermanentInitState`. What this list buys now is that a TENTH
/// unread field cannot arrive in silence.
const UNREAD_WITHOUT_DETECTOR: &[(&str, &str, &str)] = &[
    (
        "InitialState",
        "format",
        "Structurally always populated (a required `String`) and read by nothing: \
         `build_initial_state` never branches on the format. Same shape as the allowlisted \
         `priority`, so requiring it EMPTY is impossible; giving it a detector plus an \
         allowlist entry would assert nothing, which is why it is recorded here instead.",
    ),
    (
        "CommanderInitState",
        "zone",
        "The script says where the commander STARTS; `build_initial_state` pushes its \
         `CardId` into `commander_ids` and places the card only from the zone maps, so a \
         commander declared `zone: \"battlefield\"` and not also listed in \
         `zones.battlefield` is in no zone at all. This is also the one MEASURED read-token \
         collision behind `READ_TOKEN_COLLISIONS`: a naive token scan calls it read, because \
         the CR 702.62 suspend patch in the same function contains `obj.zone == \
         ZoneId::Exile` -- a `GameObject`'s zone, not the script's.",
    ),
    (
        "CommanderInitState",
        "times_cast_from_command_zone",
        "CR 903.8's commander tax, never applied: the commander loop reads `cmdr.card` and \
         nothing else. A script describing a commander already cast twice builds a state that \
         charges no tax -- SR-9b's `turn_number` defect one field over, and the closest thing \
         in this list to a live correctness gap rather than an unbuilt board.",
    ),
    (
        "ZonesInitState",
        "stack",
        "No object is ever created on the stack: the builder iterates battlefield, hand, \
         graveyard, exile and library, and not this. A script that pre-loads the stack \
         describes a board the harness does not build. (`zones.stack` also appears as an \
         ASSERTION path in `script_replay.rs`; that reads the BUILT state and is a different \
         thing from this field.)",
    ),
    (
        "PermanentInitState",
        "attached",
        "Auras and Equipment start UNATTACHED. Already named as a still-unread field by \
         `scripts::harness_equivalence`'s own module-doc table and absent from \
         `UNREAD_INIT_FIELDS` -- two enumerations of the same class, in the same test binary, \
         disagreeing with each other. Neither was pinned to the declaration, which is how \
         they were able to.",
    ),
    (
        "PermanentInitState",
        "is_commander",
        "The commander flag on a battlefield permanent is not applied; commander identity \
         comes only from `players.<p>.commander`, which is why `harness_equivalence`'s \
         `card_names` had to be taught to read the commander block separately.",
    ),
    (
        "PermanentInitState",
        "subtypes",
        "An override of the printed subtype line. `make_spec` takes subtypes from the \
         `CardDefinition` through `enrich_spec_from_def` and never consults this field, so a \
         script overriding a creature's types describes a permanent the harness does not \
         build.",
    ),
    (
        "PermanentInitState",
        "is_basic",
        "As `subtypes`: a supertype override the spec builder never consults.",
    ),
    (
        "CardInZone",
        "is_commander",
        "As `PermanentInitState.is_commander`, for cards in hand, graveyard, exile, library \
         and the command zone.",
    ),
];

/// Declared-but-unread field names whose bare `.name` token nevertheless appears in
/// `build_initial_state`'s body, because a DIFFERENT type has a field of the same name, with
/// the collision named.
///
/// This is the token scan's one fail-open direction, measured rather than assumed away, and
/// it is why the read side of this classification is hand-made with the scan as a
/// CORROBORATION rather than derived from the scan outright.
const READ_TOKEN_COLLISIONS: &[(&str, &str, &str)] = &[(
    "CommanderInitState",
    "zone",
    "`obj.zone == ZoneId::Exile` in the CR 702.62 suspend patch reads a `GameObject`'s zone, \
     not `CommanderInitState.zone`.",
)];

/// Every `pub` field name declared by `pub struct <struct_name>` in a workspace-relative
/// source file.
///
/// # This is a COPY, and the canonical version is named
///
/// The canonical implementation is
/// `crates/engine/tests/core/pb_dx57_declared_source.rs::declared_struct_fields`. It cannot
/// be shared: `core` and `scripts` are separate test BINARIES, and
/// `tests/no_stray_test_binaries.rs::group_main_rs_declares_modules_and_nothing_else` allows
/// a group's `main.rs` to hold bare `mod x;` lines and nothing else, so neither a `use` nor a
/// `#[path]` re-export is available. `primitives/pb_dp9_effect_choice.rs:2641` settled what to
/// do about exactly this: keep the copy, say it is a copy, name the canonical version, and
/// cross-check BY VALUE. The cross-check is in the test below.
///
/// # Bounds, stated
///
/// Strips `//` line comments only, then ASSERTS the file carries no `/* */` block comment
/// (PB-DX8's `OOS-DX32-6` defeat). Panics on an empty parse: a parser that returns `{}` makes
/// every `assert_eq!` against it trivially true, which is `OOS-DX28-1`'s own failure mode
/// re-entering through its fix.
fn declared_struct_fields(rel: &str, struct_name: &str) -> std::collections::BTreeSet<String> {
    let raw = read_engine_relative(rel);
    let clean = strip_line_comments(&raw);
    let header = format!("pub struct {struct_name} {{");
    let at = clean.find(&header).unwrap_or_else(|| {
        panic!(
            "`{header}` not found in {rel}. The declaration was renamed or moved -- re-point \
             this pin rather than deleting it and keeping the hand-written lists, which is \
             the defect `OOS-DX28-1` names."
        )
    });
    let body_start = clean[at..].find('{').expect("struct has a body") + at + 1;
    let mut depth = 1usize;
    let mut end = None;
    for (i, ch) in clean[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(body_start + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end.expect("the struct body is never closed \u{2014} the brace walk ran off the end");
    // Split the body on TOP-LEVEL commas rather than on lines. A line-based split takes only
    // the FIRST `pub X:` per line, so two fields written on one line
    // (`pub basic: bool, pub nonbasic: bool,` -- legal Rust that `cargo fmt` normally
    // splits, and therefore a shape a formatted tree hides) makes the parse SHORT and the
    // failure message say *"the declaration no longer has `nonbasic`"* when the declaration
    // still has it. Found by a live, unrelated plant during PB-DX57 rather than by reasoning:
    // a WRONG diagnosis from a red gate is only one step better than a green one.
    let mut fields: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    let mut prev = ' ';
    let push = |chunk: &str, out: &mut std::collections::BTreeSet<String>| {
        let mut s = chunk.trim();
        // Drop any leading `#[...]` attributes, possibly several.
        while s.starts_with("#[") {
            let mut d = 0usize;
            let mut e = None;
            for (i, ch) in s.char_indices() {
                match ch {
                    '[' => d += 1,
                    ']' => {
                        d -= 1;
                        if d == 0 {
                            e = Some(i + 1);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            match e {
                Some(i) => s = s[i..].trim_start(),
                None => break,
            }
        }
        let Some(rest) = s.strip_prefix("pub ") else {
            return;
        };
        // PB-DX57 adversarial pass: `pub r#type: bool` is a legal field declaration (a field
        // named after a keyword MUST be written that way), and an identifier scan that takes
        // only `[A-Za-z0-9_]` reads it as the EMPTY string and drops the field in silence. That
        // defeated this pin completely — the whole test target stayed green with the field
        // present. Handle the `r#` prefix, and FAIL CLOSED on a `pub ` chunk that still yields
        // nothing: a dropped field is invisible to every consumer at once, which is why the
        // by-value cross-check could not see it either.
        let rest = rest.trim_start();
        let (raw_prefix, rest) = match rest.strip_prefix("r#") {
            Some(r) => ("r#", r),
            None => ("", rest),
        };
        let body: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        assert!(
            !body.is_empty() && rest[body.len()..].trim_start().starts_with(':'),
            "could not parse a `pub` field declaration from {rest:?} -- refusing to return a \
             field set that silently omits it (PB-DX57 adversarial pass)"
        );
        out.insert(format!("{raw_prefix}{body}"));
    };
    for ch in clean[body_start..end].chars() {
        match ch {
            '{' | '(' | '[' => {
                depth += 1;
                cur.push(ch);
            }
            '}' | ')' | ']' => {
                depth -= 1;
                cur.push(ch);
            }
            '<' if prev.is_ascii_alphanumeric() || prev == '_' => {
                depth += 1;
                cur.push(ch);
            }
            '>' if depth > 0 && prev != '-' && prev != '=' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => {
                let chunk = std::mem::take(&mut cur);
                push(&chunk, &mut fields);
            }
            _ => cur.push(ch),
        }
        prev = ch;
    }
    push(&cur, &mut fields);
    let out = fields;
    assert!(
        !out.is_empty(),
        "parsed ZERO fields out of `{header}` in {rel}"
    );
    out
}

/// Read a path relative to `crates/engine`, asserting it carries no block comment.
fn read_engine_relative(rel: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} must be readable: {e}", path.display()));
    assert!(
        !raw.contains("/*"),
        "{} grew a `/* */` block comment; the parsers in this file strip `//` only, so a \
         block comment can hide or fake a declaration or a field read (`OOS-DX32-6`).",
        path.display()
    );
    raw
}

/// Length-preserving `//` strip, so offsets still map back to lines.
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|l| match l.find("//") {
            Some(i) => format!("{}{}", &l[..i], " ".repeat(l.len() - i)),
            None => l.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The brace-matched body of `pub fn build_initial_state(`, comments stripped.
fn build_initial_state_body() -> String {
    let raw = read_engine_relative("src/testing/replay_harness.rs");
    let clean = strip_line_comments(&raw);
    let at = clean
        .find("pub fn build_initial_state(")
        .expect("`pub fn build_initial_state(` must exist in src/testing/replay_harness.rs");
    let body_start = clean[at..].find('{').expect("the function has a body") + at + 1;
    let mut depth = 1usize;
    for (i, ch) in clean[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return clean[body_start..body_start + i].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("`build_initial_state`'s body is never closed -- the brace walk ran off the end");
}

/// Does `body` contain a field access `.<field>` (not part of a longer identifier)?
fn body_reads_token(body: &str, field: &str) -> bool {
    let needle = format!(".{field}");
    let bytes = body.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = body[from..].find(&needle) {
        let at = from + rel;
        let after = at + needle.len();
        let next_ok = bytes
            .get(after)
            .is_none_or(|b| !(b.is_ascii_alphanumeric() || *b == b'_'));
        let prev_ok = at == 0 || !bytes[at - 1].is_ascii_digit();
        if next_ok && prev_ok {
            return true;
        }
        from = at + 1;
    }
    false
}

/// Is this serialized JSON value at its serde default (absent / null / false / 0 / empty)?
fn is_default_json(v: &serde_json::Value) -> bool {
    match v {
        serde_json::Value::Null => true,
        serde_json::Value::Bool(b) => !*b,
        serde_json::Value::Number(n) => n.as_f64() == Some(0.0),
        serde_json::Value::String(s) => s.is_empty(),
        serde_json::Value::Array(a) => a.is_empty(),
        serde_json::Value::Object(o) => o.is_empty(),
    }
}

/// Does any node in `v` carry `key` with a non-default value?
fn json_populates(v: &serde_json::Value, key: &str) -> bool {
    match v {
        serde_json::Value::Object(map) => map
            .iter()
            .any(|(k, child)| (k == key && !is_default_json(child)) || json_populates(child, key)),
        serde_json::Value::Array(items) => items.iter().any(|i| json_populates(i, key)),
        _ => false,
    }
}

#[test]
/// `OOS-DX28-1` -- **every field the `InitialState` family declares is classified: read by
/// `build_initial_state`, unread with a detector, or unread without one.**
///
/// ## What was missing, and what it found
///
/// `UNREAD_INIT_FIELDS` is six hand-written entries, and this module's own doc says the class
/// it guards is *"the same class of bug SR-9b found with `turn_number`"* -- declared, never
/// read, so every script secretly ran on turn 1. `unread_field_detectors_are_not_vacuous`
/// proves those six detectors FIRE. **Nothing asked whether there was a SEVENTH unread
/// field.** There are nine, and they are in [`UNREAD_WITHOUT_DETECTOR`] with a reason each --
/// including `CommanderInitState.times_cast_from_command_zone`, which is CR 903.8's commander
/// tax silently dropped, and `PermanentInitState.attached`, which
/// `scripts::harness_equivalence`'s own module-doc table already listed as unread while
/// `UNREAD_INIT_FIELDS` did not. Two enumerations of one class in one test binary,
/// disagreeing, because neither was pinned to the declaration.
///
/// ## Why the READ side is hand-classified and the token scan only corroborates it
///
/// The semantic set is *declared fields MINUS the fields `build_initial_state` reads*, so it
/// wants a two-sided derivation. The read side can only be derived from source by a token
/// scan, and a token scan is **fail-OPEN**: it calls a field read whenever its bare name
/// appears, including when the name belongs to a different type. That is not hypothetical
/// here -- it is measured. `CommanderInitState.zone` is unread, and the body contains
/// `obj.zone == ZoneId::Exile` in the CR 702.62 suspend patch, so a scan-derived read set
/// would silently swallow it. A gate that fails open on the exact axis it is guarding is
/// worth less than no gate, because it reads like one.
///
/// So the classification is hand-made and the scan is used in the two directions where it
/// cannot fail open:
///
/// * for a **read** entry, the token MUST appear -- if someone deletes a read, the token
///   disappears and this row reddens, telling them to reclassify the field as unread;
/// * for an **unread** entry, the token must NOT appear, unless the field is on
///   [`READ_TOKEN_COLLISIONS`] with the colliding expression named -- so wiring an unread
///   field in reddens here rather than silently making a detector permanently false.
///
/// ## Residuals, stated rather than discovered later
///
/// 1. **Field-level, not zone-level.** `CardInZone.counters` is classified READ, and it is
///    read only in the EXILE loop -- a `counters` map on a hand, graveyard or library card is
///    dropped. A per-(struct, field) model cannot express that, and this row does not claim
///    to.
/// 2. **`READ_TOKEN_COLLISIONS` is a measured list, not a derived one.** It was built by
///    reading the body. A future collision that nobody notices would make an unread field's
///    token-absence assertion fail, which is the safe direction: it reddens and asks for a
///    decision.
/// 3. The corpus-population figures printed below are approximate by construction -- they
///    walk the serialized `InitialState` for a bare KEY, so the two `is_commander` fields
///    (one on `PermanentInitState`, one on `CardInZone`) are counted together. They are a
///    diagnostic for whoever writes the detectors, not an assertion.
fn every_declared_init_field_is_read_or_classified_unread() {
    // ── The declaration side.
    let mut declared: std::collections::BTreeSet<(String, String)> =
        std::collections::BTreeSet::new();
    for st in SCHEMA_STRUCTS {
        for f in declared_struct_fields("src/testing/script_schema.rs", st) {
            declared.insert(((*st).to_string(), f));
        }
    }
    assert!(
        declared.len() >= 35,
        "only {} field(s) parsed across the {} InitialState-family structs -- the parser is \
         broken and every assertion below is vacuous",
        declared.len(),
        SCHEMA_STRUCTS.len()
    );

    // The by-value cross-check with the canonical parser. `core::pb_dx57_declared_source`'s
    // `p1_the_parser_agrees_with_the_independent_parsers_already_in_the_tree` asserts the
    // canonical parser reads 33 fields from `pub struct TargetFilter` -- the struct whose
    // 32-to-33 growth IS `OOS-DX28-1`. This copy is pointed at the same declaration.
    let target_filter =
        declared_struct_fields("../card-types/src/cards/card_definition.rs", "TargetFilter");
    assert_eq!(
        target_filter.len(),
        33,
        "this file's COPY of the declaration parser reads {} fields from `pub struct \
         TargetFilter`; the canonical parser in \
         core::pb_dx57_declared_source::p1_the_parser_agrees_with_the_independent_parsers_already_in_the_tree \
         asserts 33. If the struct really grew, both move in the same commit; if only one \
         moved, one of the two parsers is wrong -- and this is the very struct whose 33rd \
         field is the seed.",
        target_filter.len()
    );

    // ── The classification side.
    let read: std::collections::BTreeSet<(String, String)> = READ_BY_BUILD_INITIAL_STATE
        .iter()
        .map(|(s, f)| ((*s).to_string(), (*f).to_string()))
        .collect();
    let detector_unread: std::collections::BTreeSet<(String, String)> = DETECTOR_FIELD_OWNERS
        .iter()
        .map(|(s, f)| ((*s).to_string(), (*f).to_string()))
        .collect();
    let bare_unread: std::collections::BTreeSet<(String, String)> = UNREAD_WITHOUT_DETECTOR
        .iter()
        .map(|(s, f, _)| ((*s).to_string(), (*f).to_string()))
        .collect();

    // `DETECTOR_FIELD_OWNERS` must be exactly `UNREAD_INIT_FIELDS`, by name -- otherwise the
    // owner table drifts from the list it annotates and the partition below is checking a
    // set nobody enforces.
    let detector_names: std::collections::BTreeSet<&str> =
        UNREAD_INIT_FIELDS.iter().map(|f| f.name).collect();
    let owner_names: std::collections::BTreeSet<&str> =
        DETECTOR_FIELD_OWNERS.iter().map(|(_, f)| *f).collect();
    assert_eq!(
        owner_names, detector_names,
        "DETECTOR_FIELD_OWNERS and UNREAD_INIT_FIELDS name different fields; the owner table \
         must be updated in the same edit as the detector list"
    );

    for (a, an, b, bn) in [
        (
            &read,
            "READ_BY_BUILD_INITIAL_STATE",
            &detector_unread,
            "UNREAD_INIT_FIELDS",
        ),
        (
            &read,
            "READ_BY_BUILD_INITIAL_STATE",
            &bare_unread,
            "UNREAD_WITHOUT_DETECTOR",
        ),
        (
            &detector_unread,
            "UNREAD_INIT_FIELDS",
            &bare_unread,
            "UNREAD_WITHOUT_DETECTOR",
        ),
    ] {
        let both: Vec<&(String, String)> = a.intersection(b).collect();
        assert!(
            both.is_empty(),
            "field(s) classified BOTH by {an} and by {bn}: {both:?}"
        );
    }

    let classified: std::collections::BTreeSet<(String, String)> = read
        .union(&detector_unread)
        .cloned()
        .collect::<std::collections::BTreeSet<(String, String)>>()
        .union(&bare_unread)
        .cloned()
        .collect();
    assert_eq!(
        classified,
        declared,
        "`OOS-DX28-1`: the `InitialState` family's declared fields and this classification \
         have diverged.\n  DECLARED but unclassified: {:?}\n    -> a NEW schema field. Decide \
         whether `build_initial_state` reads it. If it does, add it to \
         READ_BY_BUILD_INITIAL_STATE. If it does NOT, it is SR-9b's `turn_number` defect \
         again: a script can populate it and describe a board the harness never builds. Give \
         it a detector in UNREAD_INIT_FIELDS if the corpus leaves it empty, or record it in \
         UNREAD_WITHOUT_DETECTOR with the reason and the measured population.\n  CLASSIFIED \
         but undeclared: {:?}\n    -> a renamed or removed field; the entry now guards \
         nothing.",
        declared.difference(&classified).collect::<Vec<_>>(),
        classified.difference(&declared).collect::<Vec<_>>()
    );

    // ── The corroboration, in the two fail-CLOSED directions. See the doc.
    let body = build_initial_state_body();
    assert!(
        body.len() > 2_000,
        "`build_initial_state`'s extracted body is only {} bytes -- the brace walk is wrong \
         and every token check below is vacuous",
        body.len()
    );
    let collisions: std::collections::BTreeSet<(&str, &str)> = READ_TOKEN_COLLISIONS
        .iter()
        .map(|(s, f, _)| (*s, *f))
        .collect();

    for (st, field) in &read {
        assert!(
            body_reads_token(&body, field),
            "`{st}.{field}` is classified as READ by `build_initial_state`, and the token \
             `.{field}` does not appear anywhere in its body. Either the read was deleted -- \
             in which case the field is now UNREAD and a script populating it describes a \
             board the harness does not build (SR-9b's `turn_number`) -- or it was renamed. \
             Reclassify it; do not delete this check."
        );
    }
    for (st, field) in detector_unread.iter().chain(bare_unread.iter()) {
        if collisions.contains(&(st.as_str(), field.as_str())) {
            continue;
        }
        assert!(
            !body_reads_token(&body, field),
            "`{st}.{field}` is classified as UNREAD, and the token `.{field}` now appears in \
             `build_initial_state`'s body. Either the field was wired in -- in which case \
             move it to READ_BY_BUILD_INITIAL_STATE, and if it has a detector, delete that \
             too, because a detector on a field that IS read forbids scripts from using a \
             feature the harness now supports -- or this is a new NAME COLLISION with some \
             other type's field, in which case add it to READ_TOKEN_COLLISIONS naming the \
             colliding expression."
        );
    }
    for (st, field, why) in READ_TOKEN_COLLISIONS {
        assert!(
            body_reads_token(&body, field),
            "READ_TOKEN_COLLISIONS claims `{st}.{field}`'s token collides with something in \
             `build_initial_state`, and the token is not there at all. The collision is gone; \
             remove the entry, so the unread check above starts guarding this field again. \
             (The recorded collision was: {why})"
        );
    }

    // ── The excluded struct's exclusion is CHECKED, not asserted.
    for (st, parent, _) in EXCLUDED_SCHEMA_STRUCTS {
        assert!(
            !declared_struct_fields("src/testing/script_schema.rs", st).is_empty(),
            "excluded schema struct `{st}` no longer parses -- it was renamed or removed, and \
             the exclusion now names nothing"
        );
        let parent_is_unread = detector_unread
            .iter()
            .chain(bare_unread.iter())
            .any(|(_, f)| f == parent);
        assert!(
            parent_is_unread,
            "`{st}` is excluded from this classification because its parent field `{parent}` \
             is UNREAD, so none of its fields can matter. `{parent}` is no longer classified \
             unread, so the exclusion has expired: either classify `{st}`'s fields \
             individually, or re-establish why they cannot matter."
        );
    }

    for (st, field, why) in UNREAD_WITHOUT_DETECTOR {
        assert!(
            why.len() > 60,
            "the UNREAD_WITHOUT_DETECTOR entry for `{st}.{field}` carries no real reason: \
             {why:?}"
        );
    }

    // ── The measured corpus population, PRINTED rather than transcribed (PB-DX8's rule).
    //    This is what a later batch needs in order to decide which of the nine can be given a
    //    detector without a corpus migration.
    let approved = approved_initial_states();
    assert!(
        !approved.is_empty(),
        "found no approved scripts -- the walk is broken and the table below measures nothing"
    );
    println!(
        "OOS-DX28-1 / unread_init_fields: {} declared fields across {} structs -- {} read, \
         {} unread WITH a detector, {} unread WITHOUT one. Approved scripts populating each \
         undetected unread field (bare-key walk over the serialized InitialState, so the two \
         `is_commander` fields are counted together), out of {}:",
        declared.len(),
        SCHEMA_STRUCTS.len(),
        read.len(),
        detector_unread.len(),
        bare_unread.len(),
        approved.len()
    );
    for (st, field, _) in UNREAD_WITHOUT_DETECTOR {
        let n = approved
            .iter()
            .filter(|(_, init)| {
                let v = serde_json::to_value(init).expect("InitialState serializes");
                json_populates(&v, field)
            })
            .count();
        println!("  {st}.{field:<30} populated by {n} approved script(s)");
    }
}
