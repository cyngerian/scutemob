//! PB-OS5 (OOS-EF4-1): `EffectAmount::OtherAttackersSharingCreatureType { relative_to }`.
//!
//! CR 205.3m / 508.1 / 613.1d / 109.1 / 603.2: counts OTHER attacking creatures (any
//! controller) whose layer-resolved creature-type set shares >= 1 type with
//! `relative_to`'s layer-resolved creature-type set, excluding `relative_to` itself.
//! Single new `EffectAmount` variant (discriminant 24, PROTOCOL_VERSION 20 /
//! HASH_SCHEMA_VERSION 57). Ships three cards:
//! - Shared Animosity (inert -> Complete): `relative_to: EffectTarget::TriggeringCreature`,
//!   all-controller scope, shares-a-type predicate.
//! - Goblin Piledriver (new -> Complete): reuses `AttackingCreatureCount` with a fixed
//!   Goblin subtype filter, `Sum(count, count)` for the x2 multiplier -- NOT this new
//!   variant, included here as the piledriver-family exclude-self/scope decoy.
//! - Goblin Rabblemaster (partial, pump clause implemented): same `AttackingCreatureCount`
//!   shape as Piledriver without the `Sum` doubling.
//! - Muxus, Goblin Grandee (new -> partial, attack half only): reuses `PermanentCount`
//!   (you-control scope, NOT attacking-only) -- included as the 4-player you-control
//!   scope decoy.
//!
//! Test pattern: attack triggers driven through the real `Command::DeclareAttackers` path
//! (mirrors `pb_ef4_triggering_creature_subject_source.rs` / `pb_ac3_dynamic_pt_counts.rs`),
//! reading layer-resolved P/T via `calculate_characteristics` AFTER the trigger resolves.
//! One test (`test_os5_scope_animosity_piledriver_any_controller`) constructs `CombatState`
//! directly (mirrors `pb_ac3_dynamic_pt_counts.rs::test_attacking_creature_count_basic`)
//! because normal `DeclareAttackers` command validation only allows the active player's
//! own creatures to attack in a single combat -- there is no real-command path to an
//! attacking set spanning two controllers. `execute_effect` is still the real resolution
//! function (`resolve_amount`'s only caller for the effect-path substitution), so this
//! remains execution-probing, not source-tracing (SR-34/36).

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    all_cards, calculate_characteristics, enrich_spec_from_def, process_command, AttackTarget,
    CardContinuousEffectDef, CardDefinition, CardEffectTarget, CardRegistry, CombatState, Command,
    EffectAmount, EffectDuration, EffectFilter, EffectLayer, GameEvent, GameState,
    GameStateBuilder, KeywordAbility, LayerModification, ObjectId, ObjectSpec, PlayerId, Step,
    SubType, HASH_SCHEMA_VERSION, PROTOCOL_VERSION,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn power(state: &GameState, id: ObjectId) -> Option<i32> {
    calculate_characteristics(state, id).and_then(|c| c.power)
}

fn toughness(state: &GameState, id: ObjectId) -> Option<i32> {
    calculate_characteristics(state, id).and_then(|c| c.toughness)
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn place_card(p1: PlayerId, name: &str, defs: &HashMap<String, CardDefinition>) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(p1, name).in_zone(mtg_engine::ZoneId::Battlefield),
        defs,
    )
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Drain the stack fully (repeated `pass_all` rounds) -- needed when a single
/// `DeclareAttackers` synthesizes multiple stack objects (one per attacking creature
/// you control, CR 508.1m).
fn drain_stack(mut state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(
            guard < 50,
            "drain_stack: stack did not empty after 50 rounds"
        );
        let (s, ev) = pass_all(state, players);
        state = s;
        all_events.extend(ev);
    }
    (state, all_events)
}

fn declare_attackers(
    state: GameState,
    player: PlayerId,
    attackers: Vec<(ObjectId, AttackTarget)>,
) -> GameState {
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player,
            attackers,
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");
    state
}

// ── Test 1: shared_animosity counts shared-type attackers, excludes non-attackers ──

/// CR 508.1m / CR 205.3m -- Shared Animosity: the triggering creature (subject) gets
/// +1/+0 for each OTHER attacking creature sharing a creature type with it. A same-type
/// creature that is NOT attacking does not count; a differently-typed attacker does not
/// count.
#[test]
fn test_os5_shared_animosity_counts_shared_type_attackers() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let animosity = place_card(p1, "Shared Animosity", &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(animosity)
        .object(
            ObjectSpec::creature(p1, "Subject Elf", 2, 2)
                .with_subtypes(vec![SubType("Elf".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1, "Ally Elf A", 2, 2)
                .with_subtypes(vec![SubType("Elf".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1, "Ally Elf B", 2, 2)
                .with_subtypes(vec![SubType("Elf".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1, "Ally Elf C", 2, 2)
                .with_subtypes(vec![SubType("Elf".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1, "Foreign Attacker", 2, 2)
                .with_subtypes(vec![SubType("Human".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1, "Bench Elf", 2, 2)
                .with_subtypes(vec![SubType("Elf".to_string())]),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let subject_id = find_object(&state, "Subject Elf");
    let ally_a_id = find_object(&state, "Ally Elf A");
    let ally_b_id = find_object(&state, "Ally Elf B");
    let ally_c_id = find_object(&state, "Ally Elf C");
    let foreign_id = find_object(&state, "Foreign Attacker");
    let bench_id = find_object(&state, "Bench Elf");

    let state = declare_attackers(
        state,
        p1,
        vec![
            (subject_id, AttackTarget::Player(p2)),
            (ally_a_id, AttackTarget::Player(p2)),
            (ally_b_id, AttackTarget::Player(p2)),
            (ally_c_id, AttackTarget::Player(p2)),
            (foreign_id, AttackTarget::Player(p2)),
        ],
    );
    let (state, _) = drain_stack(state, &[p1, p2]);

    assert_eq!(
        power(&state, subject_id),
        Some(5),
        "CR 508.1m/205.3m: Subject Elf should get +1/+0 for each of the 3 other \
         attacking Elves (2 base + 3 = 5)"
    );
    assert_eq!(
        power(&state, foreign_id),
        Some(2),
        "Foreign Attacker shares no type with anyone attacking (all-Elf allies, \
         itself Human) -- must stay at base power"
    );
    assert_eq!(
        power(&state, bench_id),
        Some(2),
        "Bench Elf is not attacking -- must be completely unaffected by the trigger"
    );
}

// ── Test 2 (MANDATORY layer-resolution decoy) ──────────────────────────────────

/// CR 613.1d / 702.73a -- a Changeling attacker (all creature types via the Layer-4
/// CDA) shares a type with the subject and IS counted; a base-typed Elf creature that
/// is NOT attacking is NOT counted (attacking-membership gate). Proves the count reads
/// layer-resolved subtypes (not base -- a Changeling's BASE subtype set is empty, so
/// reading base characteristics would silently drop it from the count) and reads combat
/// state (attacking-set membership), not just any battlefield object with a matching type.
#[test]
fn test_os5_shared_animosity_layer_resolved_subtype_decoy() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let animosity = place_card(p1, "Shared Animosity", &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(animosity)
        .object(
            ObjectSpec::creature(p1, "Subject Elf", 2, 2)
                .with_subtypes(vec![SubType("Elf".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1, "Changeling Attacker", 2, 2)
                .with_keyword(KeywordAbility::Changeling),
        )
        .object(
            ObjectSpec::creature(p1, "Base Elf NonAttacker", 2, 2)
                .with_subtypes(vec![SubType("Elf".to_string())]),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let subject_id = find_object(&state, "Subject Elf");
    let changeling_id = find_object(&state, "Changeling Attacker");

    // precondition: the Changeling's BASE subtype set does not already contain Elf --
    // proves any match comes from layer resolution, not the builder pre-populating it.
    let changeling_obj = state.objects().get(&changeling_id).unwrap();
    assert!(
        !changeling_obj
            .characteristics
            .subtypes
            .contains(&SubType("Elf".to_string())),
        "precondition: Changeling Attacker's BASE subtypes must not include Elf"
    );

    let state = declare_attackers(
        state,
        p1,
        vec![
            (subject_id, AttackTarget::Player(p2)),
            (changeling_id, AttackTarget::Player(p2)),
        ],
    );
    let (state, _) = drain_stack(state, &[p1, p2]);

    assert_eq!(
        power(&state, subject_id),
        Some(3),
        "CR 702.73a/613.1d: the attacking Changeling shares every creature type \
         (layer-resolved), including Elf -- Subject should get +1/+0 (2 base + 1 = 3), \
         proving the Non-Attacking base-Elf is correctly excluded and the Changeling's \
         layer-resolved subtypes are correctly included"
    );
}

// ── Test 3 (MANDATORY exclude-self, source != subject decoy) ───────────────────

/// CR 109.1 / 603.2 -- with only the Subject attacking (no other attackers at all),
/// the trigger's "other" must exclude the TRIGGERING CREATURE (the subject itself),
/// not the enchantment SOURCE. Since Shared Animosity's `ctx.source` (the enchantment)
/// is never itself an attacking creature, a correct exclusion via `relative_to`
/// (resolved to the subject) and an incorrect exclusion via `ctx.source` (the
/// enchantment) diverge exactly here: incorrectly excluding only `ctx.source` would
/// leave the subject counting ITSELF as an "other" attacker (obj.id != ctx.source is
/// trivially true for the subject), producing +1/+0 instead of +0/+0.
#[test]
fn test_os5_shared_animosity_excludes_triggering_creature() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let animosity = place_card(p1, "Shared Animosity", &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(animosity)
        .object(
            ObjectSpec::creature(p1, "Subject Elf", 2, 2)
                .with_subtypes(vec![SubType("Elf".to_string())]),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let subject_id = find_object(&state, "Subject Elf");

    let state = declare_attackers(state, p1, vec![(subject_id, AttackTarget::Player(p2))]);
    let (state, _) = drain_stack(state, &[p1, p2]);

    assert_eq!(
        power(&state, subject_id),
        Some(2),
        "CR 109.1/603.2: with no OTHER attackers, the subject must not count itself -- \
         exclusion must key on the resolved relative_to (the triggering creature), not \
         ctx.source (the enchantment, which is never an attacker and would trivially \
         pass an obj.id != ctx.source check)"
    );
}

// ── Test 4 (MANDATORY piledriver x2 multiplier + exclude-self) ─────────────────

/// CR 508.1m -- Goblin Piledriver: alone attacking -> +0/+0; +1 other attacking
/// Goblin -> +2/+0; +2 other attacking Goblins -> +4/+0. Proves `Sum(count, count)`
/// doubles correctly and that Piledriver excludes itself from its own count via
/// `ctx.source` (`WhenAttacks` -> source is Piledriver itself).
#[test]
fn test_os5_piledriver_double_multiplier_and_exclude_self() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    // Alone: +0/+0.
    {
        let piledriver = place_card(p1, "Goblin Piledriver", &defs);
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(CardRegistry::new(all_cards()))
            .object(piledriver)
            .active_player(p1)
            .at_step(Step::DeclareAttackers)
            .build()
            .unwrap();
        let pd_id = find_object(&state, "Goblin Piledriver");
        let state = declare_attackers(state, p1, vec![(pd_id, AttackTarget::Player(p2))]);
        let (state, _) = drain_stack(state, &[p1, p2]);
        assert_eq!(
            power(&state, pd_id),
            Some(1),
            "Piledriver alone attacking: no other Goblins -- stays at base power 1"
        );
    }

    // +1 other attacking Goblin: +2/+0.
    {
        let piledriver = place_card(p1, "Goblin Piledriver", &defs);
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(CardRegistry::new(all_cards()))
            .object(piledriver)
            .object(
                ObjectSpec::creature(p1, "Goblin Ally", 2, 2)
                    .with_subtypes(vec![SubType("Goblin".to_string())]),
            )
            .active_player(p1)
            .at_step(Step::DeclareAttackers)
            .build()
            .unwrap();
        let pd_id = find_object(&state, "Goblin Piledriver");
        let ally_id = find_object(&state, "Goblin Ally");
        let state = declare_attackers(
            state,
            p1,
            vec![
                (pd_id, AttackTarget::Player(p2)),
                (ally_id, AttackTarget::Player(p2)),
            ],
        );
        let (state, _) = drain_stack(state, &[p1, p2]);
        assert_eq!(
            power(&state, pd_id),
            Some(3),
            "Piledriver + 1 other attacking Goblin: +2/+0 (1 base + 2 = 3)"
        );
    }

    // +2 other attacking Goblins: +4/+0.
    {
        let piledriver = place_card(p1, "Goblin Piledriver", &defs);
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(CardRegistry::new(all_cards()))
            .object(piledriver)
            .object(
                ObjectSpec::creature(p1, "Goblin Ally A", 2, 2)
                    .with_subtypes(vec![SubType("Goblin".to_string())]),
            )
            .object(
                ObjectSpec::creature(p1, "Goblin Ally B", 2, 2)
                    .with_subtypes(vec![SubType("Goblin".to_string())]),
            )
            .active_player(p1)
            .at_step(Step::DeclareAttackers)
            .build()
            .unwrap();
        let pd_id = find_object(&state, "Goblin Piledriver");
        let ally_a_id = find_object(&state, "Goblin Ally A");
        let ally_b_id = find_object(&state, "Goblin Ally B");
        let state = declare_attackers(
            state,
            p1,
            vec![
                (pd_id, AttackTarget::Player(p2)),
                (ally_a_id, AttackTarget::Player(p2)),
                (ally_b_id, AttackTarget::Player(p2)),
            ],
        );
        let (state, _) = drain_stack(state, &[p1, p2]);
        assert_eq!(
            power(&state, pd_id),
            Some(5),
            "Piledriver + 2 other attacking Goblins: +4/+0 (1 base + 4 = 5)"
        );
    }
}

/// Negative case: a non-Goblin attacker does not contribute to Piledriver's count
/// (fixed Goblin subtype filter, not a shared-type predicate).
#[test]
fn test_os5_piledriver_ignores_nongoblin_attackers() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let piledriver = place_card(p1, "Goblin Piledriver", &defs);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(piledriver)
        .object(
            ObjectSpec::creature(p1, "Elf Attacker", 2, 2)
                .with_subtypes(vec![SubType("Elf".to_string())]),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    let pd_id = find_object(&state, "Goblin Piledriver");
    let elf_id = find_object(&state, "Elf Attacker");
    let state = declare_attackers(
        state,
        p1,
        vec![
            (pd_id, AttackTarget::Player(p2)),
            (elf_id, AttackTarget::Player(p2)),
        ],
    );
    let (state, _) = drain_stack(state, &[p1, p2]);
    assert_eq!(
        power(&state, pd_id),
        Some(1),
        "a non-Goblin attacker must not contribute to Piledriver's count"
    );
}

// ── Test 5 (MANDATORY 4-player you-control scope decoy: Muxus) ─────────────────

/// CR 205.3m -- Muxus, Goblin Grandee: counts other Goblins YOU CONTROL (not
/// attacking-only), excludes Muxus itself, and does NOT count an opponent's Goblin
/// regardless of whether it's attacking. 4-player game.
#[test]
fn test_os5_muxus_you_control_scope_4player() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let defs = load_defs();

    let muxus = place_card(p1, "Muxus, Goblin Grandee", &defs);

    let state = GameStateBuilder::four_player()
        .with_registry(CardRegistry::new(all_cards()))
        .object(muxus)
        .object(
            ObjectSpec::creature(p1, "P1 Goblin A", 2, 2)
                .with_subtypes(vec![SubType("Goblin".to_string())]),
        )
        .object(
            ObjectSpec::creature(p1, "P1 Goblin B", 2, 2)
                .with_subtypes(vec![SubType("Goblin".to_string())]),
        )
        .object(
            ObjectSpec::creature(p2, "P2 Goblin", 2, 2)
                .with_subtypes(vec![SubType("Goblin".to_string())]),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let muxus_id = find_object(&state, "Muxus, Goblin Grandee");

    let state = declare_attackers(state, p1, vec![(muxus_id, AttackTarget::Player(p2))]);
    let (state, _) = drain_stack(state, &[p1, p2, p3, p4]);

    assert_eq!(
        power(&state, muxus_id),
        Some(6),
        "CR 205.3m: Muxus should get +1/+1 for each of the 2 other Goblins YOU \
         CONTROL (neither attacking) -- 4 base + 2 = 6. P2's Goblin must NOT count \
         (wrong controller)."
    );
    assert_eq!(
        toughness(&state, muxus_id),
        Some(6),
        "the toughness half of the +1/+1 pump must move in lockstep with power"
    );
}

// ── Test 6 (MANDATORY any-controller scope decoy) ───────────────────────────────

/// CR 508.1: the shared-animosity-family count does NOT filter by controller -- a
/// same-type attacker controlled by a DIFFERENT player still counts, and a
/// non-attacking same-type permanent controlled by that other player does NOT count
/// (attacking-membership gate, independent of controller). `DeclareAttackers`
/// validation only allows the active player's own creatures to attack in a single
/// combat, so this constructs `CombatState` directly (mirrors
/// `pb_ac3_dynamic_pt_counts.rs::test_attacking_creature_count_basic`) and drives the
/// real `execute_effect` / `resolve_amount` path -- execution-probing, not
/// source-tracing.
#[test]
fn test_os5_scope_animosity_piledriver_any_controller() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Subject Goblin", 2, 2)
                .with_subtypes(vec![SubType("Goblin".to_string())]),
        )
        .object(
            ObjectSpec::creature(p2, "Foreign Goblin Attacker", 2, 2)
                .with_subtypes(vec![SubType("Goblin".to_string())]),
        )
        .object(
            ObjectSpec::creature(p2, "Foreign Goblin Bench", 2, 2)
                .with_subtypes(vec![SubType("Goblin".to_string())]),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let subject_id = find_object(&state, "Subject Goblin");
    let foreign_attacker_id = find_object(&state, "Foreign Goblin Attacker");
    let foreign_bench_id = find_object(&state, "Foreign Goblin Bench");

    // Manually construct a two-controller attacking set (CR 508.1 doesn't forbid it
    // in the abstract -- the DeclareAttackers command just enforces single-active-
    // player combat, which is the common case, not a rules requirement on this
    // primitive). Both Subject and the Foreign attacker are declared attacking p1's
    // hypothetical opponent context; Foreign Goblin Bench is NOT declared.
    *state.combat_mut() = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(subject_id, AttackTarget::Player(p2));
        cs.attackers
            .insert(foreign_attacker_id, AttackTarget::Player(p1));
        cs
    });

    let effect = mtg_engine::Effect::ApplyContinuousEffect {
        effect_def: Box::new(CardContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyPowerDynamic {
                amount: Box::new(EffectAmount::OtherAttackersSharingCreatureType {
                    relative_to: CardEffectTarget::Source,
                }),
                negate: false,
            },
            filter: EffectFilter::Source,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p1, subject_id, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        power(&state, subject_id),
        Some(3),
        "CR 508.1: the count must NOT filter by controller -- Foreign Goblin \
         Attacker (controlled by p2) still counts toward Subject Goblin's (p1) \
         pump (2 base + 1 = 3). Foreign Goblin Bench (not attacking) must not \
         count, regardless of controller."
    );
    let _ = foreign_bench_id;
}

// ── Test 7: registration / execution smoke tests ────────────────────────────────

/// `all_cards()` contains the four affected defs, and each loads without panicking.
#[test]
fn test_os5_cards_registered() {
    let names = [
        "Shared Animosity",
        "Goblin Piledriver",
        "Goblin Rabblemaster",
        "Muxus, Goblin Grandee",
    ];
    let all = all_cards();
    for name in names {
        assert!(
            all.iter().any(|d| d.name == name),
            "'{}' should be present in all_cards()",
            name
        );
    }
}

/// CR 508.1m -- Goblin Rabblemaster's pump clause (implemented this batch) produces
/// correct game state through a real DeclareAttackers path: alone attacking -> +0/+0;
/// + 1 other attacking Goblin -> +1/+0.
#[test]
fn test_os5_rabblemaster_pump_clause_executes() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let rabblemaster = place_card(p1, "Goblin Rabblemaster", &defs);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(rabblemaster)
        .object(
            ObjectSpec::creature(p1, "Goblin Ally", 2, 2)
                .with_subtypes(vec![SubType("Goblin".to_string())]),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    let rm_id = find_object(&state, "Goblin Rabblemaster");
    let ally_id = find_object(&state, "Goblin Ally");
    let state = declare_attackers(
        state,
        p1,
        vec![
            (rm_id, AttackTarget::Player(p2)),
            (ally_id, AttackTarget::Player(p2)),
        ],
    );
    let (state, _) = drain_stack(state, &[p1, p2]);
    assert_eq!(
        power(&state, rm_id),
        Some(3),
        "Rabblemaster (base 2) + 1 other attacking Goblin: +1/+0 -> power 3"
    );
}

/// CR 205.3m -- Muxus' attack-half executes and registers via a real Triggered
/// ability dispatch even with zero other Goblins (the +0/+0 floor case).
#[test]
fn test_os5_muxus_registers_and_pumps_zero_floor() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let muxus = place_card(p1, "Muxus, Goblin Grandee", &defs);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(muxus)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    let muxus_id = find_object(&state, "Muxus, Goblin Grandee");
    let state = declare_attackers(state, p1, vec![(muxus_id, AttackTarget::Player(p2))]);
    let (state, _) = drain_stack(state, &[p1, p2]);
    assert_eq!(
        power(&state, muxus_id),
        Some(4),
        "Muxus alone attacking, no other Goblins you control: stays at base 4/4"
    );
    assert_eq!(toughness(&state, muxus_id), Some(4));
}

// ── Test 8: wire sentinels ───────────────────────────────────────────────────────

/// PB-OS5 bumped PROTOCOL_VERSION 19 -> 20 and HASH_SCHEMA_VERSION 56 -> 57 (a single
/// new `EffectAmount` variant, discriminant 24). See
/// crates/engine/src/rules/protocol.rs and crates/engine/src/state/hash.rs for the
/// authoritative bump.
#[test]
fn test_os5_version_sentinels() {
    assert_eq!(
        PROTOCOL_VERSION, 23,
        "PROTOCOL_VERSION should be 20 after PB-OS5 (EffectAmount gained \
         OtherAttackersSharingCreatureType)"
    );
    assert_eq!(
        HASH_SCHEMA_VERSION, 60u8,
        "HASH_SCHEMA_VERSION should be 57 after PB-OS5"
    );
}
