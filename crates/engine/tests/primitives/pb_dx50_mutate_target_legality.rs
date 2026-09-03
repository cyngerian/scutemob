//! PB-DX50 half 1 (`OOS-DX25-1`) — CR 702.140a makes a mutate cast **target** its host,
//! and the engine had never treated it as a target.
//!
//! `memory/primitives/pb-plan-DX50.md` is authoritative for scope. The defect, restated
//! from the shipped code rather than from the seed:
//!
//! * The host rode in `AdditionalCost::Mutate { target, on_top }` and was never put into
//!   the `StackObject`'s `targets`. `GameEvent::PermanentTargeted` — the ONLY event that
//!   drives Ward (CR 702.21a) and the `PermanentBecomesTarget` family — is derived from
//!   `targets` alone (`rules::events::permanent_targeted_events`), so **Ward never fired
//!   on a mutate cast**, and `GameEvent::TargetsAnnounced` never named the host either.
//! * Because the host was not a target, it never met the CR 115 legality machinery. The
//!   cast-time check was four hand-rolled conjuncts — battlefield, creature, non-Human,
//!   owner — with **no hexproof (CR 702.11b), no shroud (CR 702.18a) and no protection
//!   (CR 702.16b)**.
//!
//! **The CR constraint that shapes the whole fix.** CR 702.140b is an explicit exception
//! to CR 608.2b:
//!
//! > As a mutating creature spell begins resolving, if its target is illegal, it ceases
//! > to be a mutating creature spell and continues resolving as a creature spell and will
//! > be put onto the battlefield under the control of the spell's controller.
//!
//! So routing the host into `spell_targets` must NOT hand it to the generic CR 608.2b
//! fizzle gate. It does not, and the reason is structural rather than checked: that gate
//! lives inside `resolution::resolve_top_of_stack_inner`'s `StackObjectKind::Spell` arm,
//! and `StackObjectKind::MutatingCreatureSpell` is a **disjoint arm** with no fizzle gate
//! of its own. A load-bearing accident of the `match` shape is exactly what a later batch
//! deletes while "unifying the two arms", so `t7`/`t7b`/`t7c` pin it three ways.
//!
//! **A false comment corrected in passing, and it is this queue's own recurring shape.**
//! The `MutatingCreatureSpell` arm's doc comment already claimed the resolution re-check
//! covered a host that "gained protection from the mutating spell". It did not — the four
//! conjuncts there were the same four as at cast time. `t7c` is the probe that makes the
//! sentence true.
//!
//! CR citations used throughout: CR 702.140a/b/c (mutate), CR 702.21a (Ward), CR 608.2b
//! (target legality at resolution), CR 601.2c (announcement), CR 702.11b/702.16b/702.18a
//! (hexproof / protection / shroud), CR 108.3 (ownership), CR 109.4 (control).

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::stack::StackObjectKind;
use mtg_engine::state::types::{AltCostKind, ProtectionQuality};
use mtg_engine::AdditionalCost;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Color,
    Command, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, KeywordAbility,
    ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step, SubType, Target,
    TargetRequirement, TypeLine, ZoneId,
};

// ── Shared helpers ───────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn permanent_targeted_count(events: &[GameEvent], target_id: ObjectId) -> usize {
    events
        .iter()
        .filter(
            |e| matches!(e, GameEvent::PermanentTargeted { target_id: t, .. } if *t == target_id),
        )
        .count()
}

fn ward_ability_triggered_count(
    events: &[GameEvent],
    ward_id: ObjectId,
    ward_controller: PlayerId,
) -> usize {
    events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { controller, source_object_id, .. }
                if *source_object_id == ward_id && *controller == ward_controller
            )
        })
        .count()
}

const BEAST: &str = "DX50 Mutating Beast";
const HOST: &str = "DX50 Wolf Host";

fn beast_def(spell_targets: Vec<TargetRequirement>) -> CardDefinition {
    let mut abilities = vec![
        AbilityDefinition::Keyword(KeywordAbility::Mutate),
        AbilityDefinition::MutateCost {
            cost: ManaCost {
                generic: 1,
                green: 2,
                ..Default::default()
            },
        },
    ];
    if !spell_targets.is_empty() {
        // A creature spell that ALSO declares its own `Spell` targets. No shipped mutate
        // card does this; it exists here purely so `t8` can pin that the mutate host is
        // APPENDED after the declared targets rather than prepended before them.
        abilities.push(AbilityDefinition::Spell {
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            targets: spell_targets,
            cant_be_countered: false,
            modes: None,
        });
    }
    CardDefinition {
        card_id: CardId("dx50-mutating-beast".to_string()),
        name: BEAST.to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Beast".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Mutate {1}{G}{G}".to_string(),
        abilities,
        power: Some(4),
        toughness: Some(4),
        ..Default::default()
    }
}

fn host_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dx50-wolf-host".to_string()),
        name: HOST.to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Wolf".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(3),
        ..Default::default()
    }
}

fn beast_spec(owner: PlayerId) -> ObjectSpec {
    let mut s = ObjectSpec::card(owner, BEAST)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(CardId("dx50-mutating-beast".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Beast".to_string())])
        .with_keyword(KeywordAbility::Mutate)
        .with_colors(vec![Color::Green])
        .with_mana_cost(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        });
    s.power = Some(4);
    s.toughness = Some(4);
    s
}

/// A host, always OWNED by `owner` (CR 702.140a's own axis) and controlled by
/// `controller` — which for most probes is the owner, and for the Ward / hexproof probes
/// is an opponent, since CR 702.21a and CR 702.11b are keyed on CONTROL (CR 109.4) while
/// CR 702.140a is keyed on OWNERSHIP (CR 108.3). That divergence is the only reason those
/// two probes are constructible at all.
fn host_spec(owner: PlayerId, controller: PlayerId, subtypes: Vec<&str>) -> ObjectSpec {
    let mut s = ObjectSpec::card(owner, HOST)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("dx50-wolf-host".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(
            subtypes
                .into_iter()
                .map(|s| SubType(s.to_string()))
                .collect(),
        )
        .controlled_by(controller);
    s.power = Some(2);
    s.toughness = Some(3);
    s
}

/// Build a two-seat board with the beast in p1's hand and one host, and give p1 enough
/// mana to pay `{1}{G}{G}`.
fn board(
    host: ObjectSpec,
    spell_targets: Vec<TargetRequirement>,
    extra: Vec<ObjectSpec>,
) -> GameState {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![beast_def(spell_targets), host_def()]);
    let mut b = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(beast_spec(p1))
        .object(host);
    for o in extra {
        b = b.object(o);
    }
    let mut state = b
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let pool = &mut state.players_mut().get_mut(&p1).unwrap().mana_pool;
    pool.add(ManaColor::Green, 4);
    pool.add(ManaColor::Colorless, 4);
    state.turn_mut().priority_holder = Some(p1);
    state
}

fn mutate_cast(player: PlayerId, card: ObjectId, host: ObjectId, targets: Vec<Target>) -> Command {
    Command::CastSpell(Box::new(CastSpellData {
        player,
        card,
        targets,
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        alt_cost: Some(AltCostKind::Mutate),
        prototype: false,
        modes_chosen: vec![],
        x_value: 0,
        additional_costs: vec![AdditionalCost::Mutate { target: host }],
        face_down_kind: None,
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    }))
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

// ── t1 / t2 — the announcement halves ────────────────────────────────────────────

/// CR 702.140a / CR 702.21a — the mutate host is announced as a target, so
/// `GameEvent::PermanentTargeted` names it EXACTLY ONCE with this cast's own stack-entry
/// id.
///
/// The count is asserted as `== 1`, never `>= 1`: PB-DX48's own headline was that a
/// double-dispatch design fires Ward twice and satisfies every `>= 1` assertion in the
/// tree. The `targeting_stack_id` is asserted too, because that is the field the ward
/// `CounterSpell` effect locates the targeting object by — an event with the wrong id is
/// an event Ward cannot act on.
#[test]
fn test_dx50_t1_mutate_host_raises_exactly_one_permanent_targeted() {
    let p1 = p(1);
    let state = board(host_spec(p1, p1, vec!["Wolf"]), vec![], vec![]);
    let beast_id = find_object(&state, BEAST);
    let host_id = find_object(&state, HOST);

    let (state, events) = process_command(state, mutate_cast(p1, beast_id, host_id, vec![]))
        .expect("CR 702.140a: a legal mutate cast must succeed");

    assert_eq!(
        permanent_targeted_count(&events, host_id),
        1,
        "CR 702.21a: exactly one PermanentTargeted for the mutate host (events: {events:#?})"
    );
    let stack_entry_id = state.stack_objects()[0].id;
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::PermanentTargeted { target_id, targeting_stack_id, targeting_controller }
            if *target_id == host_id
                && *targeting_stack_id == stack_entry_id
                && *targeting_controller == p1
        )),
        "CR 702.21a: PermanentTargeted must carry this cast's own stack-entry id, \
         which is what the ward CounterSpell effect locates the spell by"
    );
}

/// CR 601.2c — the mutate host reaches the event log through
/// `GameEvent::TargetsAnnounced`, and is recorded on the `StackObject` with
/// `zone_at_cast: Some(Battlefield)` (the field CR 608.2b reads at resolution).
#[test]
fn test_dx50_t2_targets_announced_carries_the_mutate_host() {
    let p1 = p(1);
    let state = board(host_spec(p1, p1, vec!["Wolf"]), vec![], vec![]);
    let beast_id = find_object(&state, BEAST);
    let host_id = find_object(&state, HOST);

    let (state, events) = process_command(state, mutate_cast(p1, beast_id, host_id, vec![]))
        .expect("CR 702.140a: a legal mutate cast must succeed");

    let announced: Vec<&GameEvent> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::TargetsAnnounced { .. }))
        .collect();
    assert_eq!(
        announced.len(),
        1,
        "CR 601.2c: exactly one TargetsAnnounced for the cast"
    );
    match announced[0] {
        GameEvent::TargetsAnnounced { targets, .. } => {
            assert_eq!(
                targets.len(),
                1,
                "the mutate host is the spell's only target"
            );
            assert_eq!(targets[0].target, Target::Object(host_id));
            assert_eq!(
                targets[0].zone_at_cast,
                Some(ZoneId::Battlefield),
                "CR 608.2b reads zone_at_cast at resolution; it must be the battlefield"
            );
        }
        other => panic!("expected TargetsAnnounced, got {other:?}"),
    }

    let so = &state.stack_objects()[0];
    assert_eq!(
        so.targets.len(),
        1,
        "the StackObject records the host as a real target, not only in additional_costs"
    );
    assert_eq!(
        so.target_requirements.len(),
        1,
        "PB-DX25c: the recorded requirement list is the one the cast validated against"
    );
}

// ── t3 / t3b — Ward, the seed's own headline ─────────────────────────────────────

/// CR 702.21a — Ward fires on a mutate cast.
///
/// **The fixture is load-bearing and its construction is the finding.** CR 702.140a keys
/// the host on OWNERSHIP ("the same owner as this spell") while CR 702.21a keys Ward on
/// CONTROL ("a spell or ability an opponent controls"). Those are different axes
/// (CR 108.3 vs CR 109.4), so the ONLY board on which a caster's own mutate can trigger a
/// host's Ward is one where the caster OWNS the host and an OPPONENT CONTROLS it — an
/// ordinary Mind Control board. That is not a contrivance to make a test pass; it is the
/// complete set of positions in which this rule interaction exists, and it is stated here
/// rather than left for a reader to rediscover.
#[test]
fn test_dx50_t3_ward_fires_when_an_opponent_controls_the_owned_host() {
    let p1 = p(1);
    let p2 = p(2);
    let host = host_spec(p1, p2, vec!["Wolf"]).with_keyword(KeywordAbility::Ward(2));
    let state = board(host, vec![], vec![]);
    let beast_id = find_object(&state, BEAST);
    let host_id = find_object(&state, HOST);

    let (state, events) = process_command(state, mutate_cast(p1, beast_id, host_id, vec![]))
        .expect("CR 702.140a: the host is owned by the caster, so the cast is legal");

    assert_eq!(
        permanent_targeted_count(&events, host_id),
        1,
        "CR 702.21a: exactly one PermanentTargeted"
    );
    assert_eq!(
        ward_ability_triggered_count(&events, host_id, p2),
        1,
        "CR 702.21a: exactly one ward AbilityTriggered, controlled by the host's \
         CONTROLLER p2 -- this is what OOS-DX25-1 says never happened (events: {events:#?})"
    );
    assert_eq!(
        state.stack_objects().len(),
        2,
        "the mutating creature spell plus the ward trigger on top"
    );
    assert!(
        matches!(
            state.stack_objects().back().unwrap().kind,
            StackObjectKind::TriggeredAbility { .. }
        ),
        "the ward trigger is placed second and therefore resolves first"
    );
}

/// CR 702.21a's "an opponent controls" clause — the non-vacuity partner for `t3`.
///
/// Same board, except the caster CONTROLS the host as well as owning it (the ordinary
/// mutate position). Ward must NOT fire, and `PermanentTargeted` must STILL be emitted:
/// `permanent_targeted_events` is controller-agnostic and only `check_triggers`'s Ward
/// arm reads controller. Asserting the 1 alongside the 0 is what makes this
/// discriminate — a bare "ward fired zero times" is also satisfied by total emission
/// failure, i.e. by the pre-PB-DX50 engine.
#[test]
fn test_dx50_t3b_ward_does_not_fire_for_its_own_controllers_mutate() {
    let p1 = p(1);
    let host = host_spec(p1, p1, vec!["Wolf"]).with_keyword(KeywordAbility::Ward(2));
    let state = board(host, vec![], vec![]);
    let beast_id = find_object(&state, BEAST);
    let host_id = find_object(&state, HOST);

    let (state, events) = process_command(state, mutate_cast(p1, beast_id, host_id, vec![]))
        .expect("CR 702.140a: a legal mutate cast must succeed");

    assert_eq!(
        permanent_targeted_count(&events, host_id),
        1,
        "CR 702.21a: the targeting event fires regardless of controller -- without this \
         assertion the ward-count assertion below is vacuous"
    );
    assert_eq!(
        ward_ability_triggered_count(&events, host_id, p1),
        0,
        "CR 702.21a: ward does not fire for its own controller's spell"
    );
    assert_eq!(
        state.stack_objects().len(),
        1,
        "only the mutating creature spell is on the stack"
    );
}

// ── t4 / t5 / t6 — the three protection-family refusals ──────────────────────────

fn assert_targeting_refusal(err: mtg_engine::GameStateError, needle: &str) {
    let msg = format!("{err:?}");
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidTarget(_)),
        "the refusal must be the TARGETING one (GameStateError::InvalidTarget), not a \
         generic InvalidCommand -- otherwise the probe would also pass against the four \
         hand-rolled conjuncts this batch deleted. Got: {msg}"
    );
    assert!(
        msg.contains(needle),
        "expected the refusal to name {needle:?}; got {msg}"
    );
}

/// CR 702.11b — a hexproof host cannot be the target of an opponent's mutate.
///
/// Hexproof is opponent-only, so this needs the same owner/controller split as `t3`.
/// Before PB-DX50 this cast SUCCEEDED: the four hand-rolled conjuncts never looked at a
/// keyword.
#[test]
fn test_dx50_t4_hexproof_host_refuses_the_mutate_cast() {
    let p1 = p(1);
    let p2 = p(2);
    let host = host_spec(p1, p2, vec!["Wolf"]).with_keyword(KeywordAbility::Hexproof);
    let state = board(host, vec![], vec![]);
    let beast_id = find_object(&state, BEAST);
    let host_id = find_object(&state, HOST);

    let err = process_command(state, mutate_cast(p1, beast_id, host_id, vec![]))
        .expect_err("CR 702.11b: a hexproof host is not a legal target for an opponent");
    assert_targeting_refusal(err, "hexproof");
}

/// CR 702.18a — a shroud host cannot be targeted by ANYONE, including the caster who owns
/// and controls it. This is the probe that needs no control-change fixture, which is why
/// it is the cleanest demonstration that the cast path now runs the real CR 115 check.
#[test]
fn test_dx50_t5_shroud_host_refuses_the_mutate_cast() {
    let p1 = p(1);
    let host = host_spec(p1, p1, vec!["Wolf"]).with_keyword(KeywordAbility::Shroud);
    let state = board(host, vec![], vec![]);
    let beast_id = find_object(&state, BEAST);
    let host_id = find_object(&state, HOST);

    let err = process_command(state, mutate_cast(p1, beast_id, host_id, vec![]))
        .expect_err("CR 702.18a: a shroud host cannot be targeted at all");
    assert_targeting_refusal(err, "shroud");
}

/// CR 702.16b — protection from green refuses a green mutating creature spell. Also
/// controller-agnostic (protection's T is "can't be the target of", full stop), so this
/// too runs on the ordinary same-owner-same-controller board.
#[test]
fn test_dx50_t6_protection_from_the_spells_color_refuses_the_mutate_cast() {
    let p1 = p(1);
    let host = host_spec(p1, p1, vec!["Wolf"]).with_keyword(KeywordAbility::ProtectionFrom(
        ProtectionQuality::FromColor(Color::Green),
    ));
    let state = board(host, vec![], vec![]);
    let beast_id = find_object(&state, BEAST);
    let host_id = find_object(&state, HOST);

    let err = process_command(state, mutate_cast(p1, beast_id, host_id, vec![]))
        .expect_err("CR 702.16b: protection from green refuses a green mutate spell");
    assert_targeting_refusal(err, "protection");
}

// ── t7 / t7b / t7c — CR 702.140b, the exception to CR 608.2b ─────────────────────

/// Assert the CR 702.140b fallback: the spell resolved as an ORDINARY creature spell —
/// a permanent entered the battlefield, nothing fizzled, and no merge happened.
fn assert_cr_702_140b_fallback(state: &GameState, events: &[GameEvent], host_id: Option<ObjectId>) {
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellFizzled { .. })),
        "CR 702.140b is an EXCEPTION to CR 608.2b: a mutating creature spell with an \
         illegal target does NOT fizzle. If this fires, someone unified the \
         MutatingCreatureSpell resolution arm with the Spell arm's fizzle gate. \
         Events: {events:#?}"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. })),
        "CR 702.140b: the spell continues resolving as a creature spell and is put onto \
         the battlefield. Events: {events:#?}"
    );
    // Identity, not name: CR 729.2a would give a MERGED permanent the beast's name too,
    // so a name-only search cannot tell the fallback from a successful merge. The beast
    // must be a battlefield object that is NOT the host.
    let beast = state
        .objects()
        .values()
        .find(|o| {
            o.characteristics.name == BEAST
                && o.zone == ZoneId::Battlefield
                && Some(o.id) != host_id
        })
        .expect("CR 702.140b: the beast must be a SEPARATE battlefield permanent");
    assert!(
        beast.merged_components.is_empty(),
        "CR 702.140b: no merge happened, so the beast carries no merged components"
    );
    if let Some(host_id) = host_id {
        let host = state
            .objects()
            .get(&host_id)
            .expect("the host is still on the battlefield in this probe");
        assert!(
            host.merged_components.is_empty(),
            "CR 702.140b: the host was NOT mutated onto"
        );
    }
    assert!(
        state.stack_objects().is_empty(),
        "the stack is empty after resolution"
    );
}

/// CR 702.140b + CR 608.2b's zone clause — the host LEAVES the battlefield in response.
/// The spell must not fizzle; it enters as an ordinary creature.
///
/// This is the conjunct `is_target_legal` alone answers, and it is the only one of the
/// three `t7*` probes that a zone-only implementation of site 2 would get right.
#[test]
fn test_dx50_t7_host_leaves_the_battlefield_triggers_cr_702_140b_not_a_fizzle() {
    let p1 = p(1);
    let p2 = p(2);
    let state = board(host_spec(p1, p1, vec!["Wolf"]), vec![], vec![]);
    let beast_id = find_object(&state, BEAST);
    let host_id = find_object(&state, HOST);

    let (mut state, _) = process_command(state, mutate_cast(p1, beast_id, host_id, vec![]))
        .expect("CR 702.140a: a legal mutate cast must succeed");

    // In response, the host leaves the battlefield.
    state.objects_mut().get_mut(&host_id).unwrap().zone = ZoneId::Graveyard(p1);

    let (state, events) = pass_all(state, &[p1, p2]);
    assert_cr_702_140b_fallback(&state, &events, None);
}

/// CR 702.140b — the host **stays on the battlefield** and becomes a Human in response.
///
/// **This is the probe the coordinator's correction to the brief exists for.** The brief's
/// first draft said site 2 should be `is_target_legal` alone; `is_target_legal` compares
/// only `obj.zone` against `zone_at_cast`, so a host that never moved but stopped matching
/// CR 702.140a's own restriction passes it. Implementing site 2 with the zone check alone
/// makes this test RED and `t7` GREEN — which is precisely why both conjuncts are needed
/// and why one probe for the pair would have been worthless.
#[test]
fn test_dx50_t7b_host_becomes_a_human_in_response_triggers_cr_702_140b() {
    let p1 = p(1);
    let p2 = p(2);
    let state = board(host_spec(p1, p1, vec!["Wolf"]), vec![], vec![]);
    let beast_id = find_object(&state, BEAST);
    let host_id = find_object(&state, HOST);

    let (mut state, _) = process_command(state, mutate_cast(p1, beast_id, host_id, vec![]))
        .expect("CR 702.140a: a legal mutate cast must succeed");

    // In response, the host becomes a Human WITHOUT changing zones (CR 702.140a's
    // non-Human restriction is now violated; CR 608.2b's zone clause is not).
    state
        .objects_mut()
        .get_mut(&host_id)
        .unwrap()
        .characteristics
        .subtypes
        .insert(SubType("Human".to_string()));

    let (state, events) = pass_all(state, &[p1, p2]);
    assert_cr_702_140b_fallback(&state, &events, Some(host_id));
}

/// CR 702.140b + CR 702.11b — the host **stays on the battlefield** and gains hexproof
/// under an opponent's control in response.
///
/// This is the probe that makes site 2's protection half an exercised behaviour rather
/// than an unexercised claim, and it is the case the arm's own doc comment has claimed to
/// handle since the mutate subsystem shipped ("...or gained protection from the mutating
/// spell") while checking nothing of the kind.
#[test]
fn test_dx50_t7c_host_gains_hexproof_in_response_triggers_cr_702_140b() {
    let p1 = p(1);
    let p2 = p(2);
    // Owned by p1 (CR 702.140a), controlled by p2 so hexproof's opponent clause bites.
    let state = board(host_spec(p1, p2, vec!["Wolf"]), vec![], vec![]);
    let beast_id = find_object(&state, BEAST);
    let host_id = find_object(&state, HOST);

    let (mut state, _) = process_command(state, mutate_cast(p1, beast_id, host_id, vec![]))
        .expect("CR 702.140a: the host is owned by the caster, so the cast is legal");

    state
        .objects_mut()
        .get_mut(&host_id)
        .unwrap()
        .characteristics
        .keywords
        .insert(KeywordAbility::Hexproof);

    let (state, events) = pass_all(state, &[p1, p2]);
    assert_cr_702_140b_fallback(&state, &events, Some(host_id));
}

/// The non-vacuity partner for the whole `t7*` family: with nothing happening in
/// response, the SAME board merges. Without this, every `t7*` probe would also pass
/// against an engine that had simply broken mutate entirely.
#[test]
fn test_dx50_t7d_an_undisturbed_mutate_still_merges() {
    let p1 = p(1);
    let p2 = p(2);
    let state = board(host_spec(p1, p1, vec!["Wolf"]), vec![], vec![]);
    let beast_id = find_object(&state, BEAST);
    let host_id = find_object(&state, HOST);

    let (state, _) = process_command(state, mutate_cast(p1, beast_id, host_id, vec![]))
        .expect("CR 702.140a: a legal mutate cast must succeed");
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 702.140c (PB-DX50, half 2): the LEGAL-target branch suspends to ask the
    // controller over-or-under. The `t7*` fallback probes above reach the CR 702.140b
    // branch, which asks nothing -- so this non-vacuity partner is also the pin that the
    // ask fires on exactly the branch CR 702.140c names and not on the other one.
    let pending = state
        .pending_effect_choice()
        .cloned()
        .expect("CR 702.140c: a legal-target mutate resolution asks over-or-under");
    assert_eq!(
        pending.question,
        mtg_engine::EffectChoiceQuestion::MutateOnTop { host: host_id },
        "the question names the host"
    );
    let (state, events) = process_command(
        state,
        Command::AnswerEffectChoice {
            player: p1,
            choice_id: pending.choice_id,
            answer: mtg_engine::EffectChoiceAnswer::MutateOnTop { on_top: true },
        },
    )
    .expect("both answers are legal (CR 702.140c)");

    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellFizzled { .. })),
        "an undisturbed mutate never fizzles"
    );
    let host = state
        .objects()
        .get(&host_id)
        .expect("the host permanent survives the merge (CR 729.2: same object)");
    assert_eq!(
        host.merged_components.len(),
        2,
        "CR 729.2: the merged permanent carries both components (spell + host)"
    );
    // CR 729.2a: the merged permanent takes the TOPMOST component's name, so it is now
    // itself called `BEAST` -- a name-only assertion here would be satisfied by the merged
    // object and prove nothing. Assert on IDENTITY instead: the only battlefield object is
    // the host, i.e. the spell created no separate permanent.
    let battlefield: Vec<ObjectId> = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .collect();
    assert_eq!(
        battlefield,
        vec![host_id],
        "CR 729.2: the mutating spell does NOT enter the battlefield as a separate          permanent -- the only battlefield object is the merged host"
    );
}

// ── t8 — index stability ─────────────────────────────────────────────────────────

/// CR 601.2c — the mutate host is APPENDED to the spell's declared targets, never
/// prepended, so every pre-existing `CardEffectTarget::DeclaredTarget { index }` position
/// is unchanged.
///
/// **Stated limitation, rather than an implied one.** No shipped mutate card declares its
/// own `AbilityDefinition::Spell` targets, and `StackObjectKind::MutatingCreatureSpell`
/// resolution runs no spell effect at all, so there is no live `DeclaredTarget` consumer
/// this can drive end to end. The assertion is therefore on the RECORDED ORDER — the
/// value every positional consumer reads — and it goes red on a prepend, which is the
/// regression it exists to catch.
#[test]
fn test_dx50_t8_the_mutate_host_is_appended_after_declared_targets() {
    let p1 = p(1);
    let mut decoy = ObjectSpec::creature(p1, "DX50 Decoy", 1, 1);
    decoy.subtypes = vec![SubType("Wolf".to_string())];
    let state = board(
        host_spec(p1, p1, vec!["Wolf"]),
        vec![TargetRequirement::TargetCreature],
        vec![decoy],
    );
    let beast_id = find_object(&state, BEAST);
    let host_id = find_object(&state, HOST);
    let decoy_id = find_object(&state, "DX50 Decoy");

    let (state, events) = process_command(
        state,
        mutate_cast(p1, beast_id, host_id, vec![Target::Object(decoy_id)]),
    )
    .expect("a mutate cast that also declares its own Spell target must succeed");

    let so = &state.stack_objects()[0];
    assert_eq!(
        so.targets.len(),
        2,
        "the declared target plus the appended mutate host"
    );
    assert_eq!(
        so.targets[0].target,
        Target::Object(decoy_id),
        "index 0 must still be the DECLARED target -- appending, not prepending, is what \
         keeps every DeclaredTarget {{ index }} valid"
    );
    assert_eq!(
        so.targets[1].target,
        Target::Object(host_id),
        "the mutate host occupies the appended last slot"
    );
    assert_eq!(
        so.target_requirements.len(),
        2,
        "the recorded requirement list grew alongside the target list"
    );
    // CR 702.21a: both are battlefield permanents, so both raise their own event.
    assert_eq!(permanent_targeted_count(&events, decoy_id), 1);
    assert_eq!(permanent_targeted_count(&events, host_id), 1);
}

// ── t9 / t10 — the four deleted conjuncts, still enforced by the requirement ─────

/// CR 702.140a "non-Human" — a Human host is still refused, now by
/// `TargetFilter::exclude_subtypes` inside `mutate_target_requirement()` rather than by a
/// hand-rolled `subtypes.contains("Human")`. This is the regression guard for the
/// deletion.
#[test]
fn test_dx50_t9_human_host_is_still_refused() {
    let p1 = p(1);
    let state = board(host_spec(p1, p1, vec!["Human"]), vec![], vec![]);
    let beast_id = find_object(&state, BEAST);
    let host_id = find_object(&state, HOST);

    let err = process_command(state, mutate_cast(p1, beast_id, host_id, vec![]))
        .expect_err("CR 702.140a: a Human is not a legal mutate host");
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidTarget(_)),
        "the refusal is now the CR 115 targeting one, got {err:?}"
    );
}

/// CR 702.140a "the same owner as this spell" — a host the caster does not OWN is still
/// refused, now by `TargetOwner::You` (CR 108.3, PB-DX28's owner axis).
///
/// The host here is owned AND controlled by the opponent, so this probe cannot be
/// satisfied by a controller check standing in for an owner check; `t3` is the
/// complementary case (owned by the caster, controlled by the opponent) and it SUCCEEDS,
/// so the pair pins the axis as ownership rather than control.
#[test]
fn test_dx50_t10_host_owned_by_an_opponent_is_still_refused() {
    let p1 = p(1);
    let p2 = p(2);
    let state = board(host_spec(p2, p2, vec!["Wolf"]), vec![], vec![]);
    let beast_id = find_object(&state, BEAST);
    let host_id = find_object(&state, HOST);

    let err = process_command(state, mutate_cast(p1, beast_id, host_id, vec![]))
        .expect_err("CR 702.140a: the host must have the same owner as the spell");
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidTarget(_)),
        "the refusal is now the CR 115 targeting one, got {err:?}"
    );
}

/// CR 702.140a "creature" — a non-creature host is still refused, and it is refused from
/// LAYER-RESOLVED characteristics rather than printed ones. Kept as a distinct probe from
/// `t9` because `TargetCreatureWithFilter`'s creature conjunct and its `exclude_subtypes`
/// conjunct are different lines of `validate_object_satisfies_requirement`.
#[test]
fn test_dx50_t11_noncreature_host_is_still_refused() {
    let p1 = p(1);
    let rock = ObjectSpec::artifact(p1, HOST).with_card_id(CardId("dx50-wolf-host".to_string()));
    let state = board(rock, vec![], vec![]);
    let beast_id = find_object(&state, BEAST);
    let host_id = find_object(&state, HOST);

    let err = process_command(state, mutate_cast(p1, beast_id, host_id, vec![]))
        .expect_err("CR 702.140a: the host must be a creature");
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidTarget(_)),
        "the refusal is now the CR 115 targeting one, got {err:?}"
    );
}
