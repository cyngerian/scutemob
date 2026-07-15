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
