/// Replay harness helpers — extracted from `crates/engine/tests/script_replay.rs`
/// so that external tools (e.g. `tools/replay-viewer`) can reuse the same
/// `build_initial_state` logic without code duplication.
///
/// The test file retains `replay_script()`, `check_assertions()`,
/// `AssertionMismatch`, and `ReplayResult` since those types are test-specific.
///
/// # Public surface
/// - [`build_initial_state`] — converts a [`GameScript`] initial-state block into a live `GameState`
/// - [`parse_step`] — maps phase string → [`Step`]
/// - [`card_name_to_id`] — converts display name → kebab-case `CardId`
/// - [`enrich_spec_from_def`] — fills in types/costs/keywords from a `CardDefinition`
/// - [`parse_counter_type`] — maps counter string → [`CounterType`]
/// - [`translate_player_action`] — maps a script `PlayerAction` string → `Command`
use std::collections::HashMap;

use im::OrdMap;

use crate::state::{ActivatedAbility, ActivationCost, CounterType};
use crate::testing::script_schema::{ActionTarget, InitialState};
use crate::{
    all_cards, register_commander_zone_replacements, AbilityDefinition, CardDefinition, CardId,
    CardRegistry, Command, Cost, Effect, GameState, GameStateBuilder, ManaAbility, ManaColor,
    ObjectSpec, PlayerId, Step, TimingRestriction, TriggerCondition, TriggerEvent,
    TriggeredAbilityDef, ZoneId,
};

// ── Public API ────────────────────────────────────────────────────────────────

/// Build a [`GameState`] from a [`GameScript`]'s initial state description.
///
/// Returns the state and a mapping from script player names → [`PlayerId`].
///
/// Player names are sorted alphabetically and assigned `PlayerId(1)`, `PlayerId(2)`, …
/// This is deterministic for a given set of player names.
pub fn build_initial_state(init: &InitialState) -> (GameState, HashMap<String, PlayerId>) {
    // Sort player names deterministically.
    let mut names: Vec<String> = init.players.keys().cloned().collect();
    names.sort();

    let player_map: HashMap<String, PlayerId> = names
        .iter()
        .enumerate()
        .map(|(i, name)| (name.clone(), PlayerId(i as u64 + 1)))
        .collect();

    let active = player_map
        .get(&init.active_player)
        .copied()
        .unwrap_or(PlayerId(1));

    let step = parse_step(&init.phase);

    // Load card definitions once for registry and for spec enrichment.
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();

    // Build registry (for spell effect execution during resolution).
    let registry = CardRegistry::new(cards); // returns Arc<CardRegistry>

    let mut builder = GameStateBuilder::new()
        .at_step(step)
        .active_player(active)
        .with_registry(registry);

    // Add players with their initial life / mana.
    for name in &names {
        let pid = player_map[name];
        builder = builder.add_player(pid);
    }

    // Helper closure: build a card spec enriched with definition characteristics.
    let make_spec = |owner: PlayerId, name: &str, zone: ZoneId| -> ObjectSpec {
        let base = ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name));
        enrich_spec_from_def(base, &defs)
    };

    // Add battlefield permanents (under each player's control).
    for (ctrl_name, permanents) in &init.zones.battlefield {
        if let Some(&ctrl) = player_map.get(ctrl_name) {
            for perm in permanents {
                let mut spec = make_spec(ctrl, &perm.card, ZoneId::Battlefield);
                if perm.tapped {
                    spec = spec.tapped();
                }
                for (ctype, count) in &perm.counters {
                    if let Some(ct) = parse_counter_type(ctype) {
                        spec = spec.with_counter(ct, *count);
                    }
                }
                if perm.damage_marked > 0 {
                    spec = spec.with_damage(perm.damage_marked);
                }
                builder = builder.object(spec);
            }
        }
    }

    // Add hand cards.
    for (owner_name, hand_cards) in &init.zones.hand {
        if let Some(&owner) = player_map.get(owner_name) {
            for card in hand_cards {
                let owner_pid = if let Some(o) = &card.owner {
                    player_map.get(o).copied().unwrap_or(owner)
                } else {
                    owner
                };
                builder = builder.object(make_spec(owner_pid, &card.card, ZoneId::Hand(owner)));
            }
        }
    }

    // Add graveyard cards.
    for (owner_name, gy_cards) in &init.zones.graveyard {
        if let Some(&owner) = player_map.get(owner_name) {
            for card in gy_cards {
                builder = builder.object(make_spec(owner, &card.card, ZoneId::Graveyard(owner)));
            }
        }
    }

    // Add exile cards.
    for card in &init.zones.exile {
        let owner = card
            .owner
            .as_deref()
            .and_then(|n| player_map.get(n))
            .copied()
            .unwrap_or(PlayerId(1));
        builder = builder.object(make_spec(owner, &card.card, ZoneId::Exile));
    }

    // Add library cards (top-to-bottom order).
    for (owner_name, lib_cards) in &init.zones.library {
        if let Some(&owner) = player_map.get(owner_name) {
            for card in lib_cards {
                builder = builder.object(make_spec(owner, &card.card, ZoneId::Library(owner)));
            }
        }
    }

    let mut state = builder.build().unwrap();

    // Patch life totals, mana pools, and land plays (can't do these via builder).
    for (name, pstate) in &init.players {
        if let Some(&pid) = player_map.get(name) {
            if let Some(ps) = state.players.get_mut(&pid) {
                ps.life_total = pstate.life;
                for (color_str, amount) in &pstate.mana_pool {
                    if let Some(color) = parse_mana_color(color_str) {
                        ps.mana_pool.add(color, *amount);
                    }
                }
                ps.land_plays_remaining = pstate.land_plays_remaining;
            }
        }
    }

    // Register commanders: populate PlayerState::commander_ids from the script's
    // commander fields, then register zone-change replacement effects (CR 903.9).
    let mut any_commanders = false;
    for (name, pstate) in &init.players {
        if let Some(&pid) = player_map.get(name) {
            for cmdr in [&pstate.commander, &pstate.partner_commander]
                .into_iter()
                .flatten()
            {
                let cid = card_name_to_id(&cmdr.card);
                if let Some(ps) = state.players.get_mut(&pid) {
                    ps.commander_ids.push_back(cid);
                }
                any_commanders = true;
            }
        }
    }
    if any_commanders {
        register_commander_zone_replacements(&mut state);
    }

    (state, player_map)
}

/// Map a script `PlayerAction` string and its parameters to a [`Command`].
///
/// Returns `None` for unrecognized action strings (future-proof: new actions
/// are silently skipped rather than panicking).
pub fn translate_player_action(
    action: &str,
    player: PlayerId,
    card_name: Option<&str>,
    ability_index: usize,
    targets: &[ActionTarget],
    state: &GameState,
    players: &HashMap<String, PlayerId>,
) -> Option<Command> {
    match action {
        "pass_priority" => Some(Command::PassPriority { player }),

        "play_land" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            Some(Command::PlayLand {
                player,
                card: card_id,
            })
        }

        "cast_spell" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
            })
        }

        // CR 702.34a: Cast a spell with flashback from the player's graveyard.
        // The engine determines it's a flashback cast by checking the card's zone
        // (graveyard) and whether it has the Flashback keyword. No new Command variant
        // is needed — CastSpell handles flashback automatically when the source is
        // in the graveyard.
        "cast_spell_flashback" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
            })
        }

        "tap_for_mana" => {
            let source_id = find_on_battlefield(state, player, card_name?)?;
            // Assume ability index 0 for basic mana abilities.
            Some(Command::TapForMana {
                player,
                source: source_id,
                ability_index: 0,
            })
        }

        "activate_ability" => {
            let source_id = find_on_battlefield(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::ActivateAbility {
                player,
                source: source_id,
                ability_index,
                targets: target_list,
            })
        }

        "cycle_card" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            Some(Command::CycleCard {
                player,
                card: card_id,
            })
        }

        // CR 702.52a: Choose to use dredge instead of drawing. card_name is the
        // dredge card to return from graveyard; if absent, declines dredge (draws normally).
        "choose_dredge" => {
            let card = card_name.and_then(|name| find_in_graveyard(state, player, name));
            Some(Command::ChooseDredge { player, card })
        }

        "concede" => Some(Command::Concede { player }),

        "return_commander_to_command_zone" => {
            // Find the commander by card name in graveyard or exile.
            let card_name = card_name?;
            let obj_id = state.objects.iter().find_map(|(&id, obj)| {
                if obj.characteristics.name == card_name
                    && obj.owner == player
                    && (matches!(obj.zone, ZoneId::Graveyard(_)) || obj.zone == ZoneId::Exile)
                {
                    Some(id)
                } else {
                    None
                }
            })?;
            Some(Command::ReturnCommanderToCommandZone {
                player,
                object_id: obj_id,
            })
        }

        "leave_commander_in_zone" => {
            // Find the commander by card name in graveyard or exile.
            let card_name = card_name?;
            let obj_id = state.objects.iter().find_map(|(&id, obj)| {
                if obj.characteristics.name == card_name
                    && obj.owner == player
                    && (matches!(obj.zone, ZoneId::Graveyard(_)) || obj.zone == ZoneId::Exile)
                {
                    Some(id)
                } else {
                    None
                }
            })?;
            Some(Command::LeaveCommanderInZone {
                player,
                object_id: obj_id,
            })
        }

        _ => {
            // Unrecognized action — skip without error.
            None
        }
    }
}

/// Convert a card's display name to its canonical [`CardId`] (kebab-case, lowercase).
///
/// Examples:
///   "Lightning Bolt" → `CardId("lightning-bolt")`
///   "Sol Ring"       → `CardId("sol-ring")`
///   "Swords to Plowshares" → `CardId("swords-to-plowshares")`
pub fn card_name_to_id(name: &str) -> CardId {
    let id = name
        .to_lowercase()
        .replace(' ', "-")
        .replace(['\'', ','], "")
        .replace("--", "-"); // avoid double-dashes from punctuation
    CardId(id)
}

/// Map a phase/step string from a script to the engine's [`Step`] enum.
pub fn parse_step(phase: &str) -> Step {
    match phase {
        "untap" => Step::Untap,
        "upkeep" | "beginning_of_upkeep" => Step::Upkeep,
        "draw" | "draw_step" => Step::Draw,
        "precombat_main" | "main1" | "pre_combat_main" => Step::PreCombatMain,
        "beginning_of_combat" | "begin_combat" => Step::BeginningOfCombat,
        "declare_attackers" | "declare_attackers_step" => Step::DeclareAttackers,
        "declare_blockers" | "declare_blockers_step" => Step::DeclareBlockers,
        "combat_damage" | "combat_damage_step" => Step::CombatDamage,
        "first_strike_damage" | "first_strike_damage_step" => Step::FirstStrikeDamage,
        "end_of_combat" | "combat_end" => Step::EndOfCombat,
        "postcombat_main" | "main2" | "post_combat_main" => Step::PostCombatMain,
        "end" | "end_step" | "ending_step" => Step::End,
        "cleanup" => Step::Cleanup,
        _ => Step::PreCombatMain, // Default to main phase.
    }
}

/// Map a counter type string to the engine's [`CounterType`] enum.
pub fn parse_counter_type(s: &str) -> Option<CounterType> {
    match s.to_lowercase().as_str() {
        "+1/+1" | "plus_one_plus_one" | "plus1plus1" => Some(CounterType::PlusOnePlusOne),
        "-1/-1" | "minus_one_minus_one" | "minus1minus1" => Some(CounterType::MinusOneMinusOne),
        "loyalty" => Some(CounterType::Loyalty),
        "poison" => Some(CounterType::Poison),
        "charge" => Some(CounterType::Charge),
        _ => None,
    }
}

/// Enrich an [`ObjectSpec`] with card type, mana cost, keyword, and mana-ability
/// information from the card's definition, if available.
///
/// This is necessary because `ObjectSpec::card(owner, name)` creates a minimal
/// object with no characteristics — the harness uses actual card definitions to
/// ensure `PlayLand`, instant-speed checks, and `TapForMana` work correctly.
pub fn enrich_spec_from_def(
    mut spec: ObjectSpec,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    let Some(def) = defs.get(&spec.name) else {
        return spec;
    };

    // Apply card types (Land, Instant, Sorcery, Artifact, etc.)
    spec.card_types = def.types.card_types.iter().cloned().collect();

    // Apply mana cost (for cost-payment validation at cast time).
    spec.mana_cost = def.mana_cost.clone();

    // Apply printed power/toughness for creatures.
    // This allows EffectAmount::PowerOf / ToughnessOf to read correct values.
    if def.power.is_some() {
        spec.power = def.power;
    }
    if def.toughness.is_some() {
        spec.toughness = def.toughness;
    }

    // Apply keyword abilities (Haste, Vigilance, Hexproof, etc.)
    for ability in &def.abilities {
        if let AbilityDefinition::Keyword(kw) = ability {
            spec = spec.with_keyword(kw.clone());
        }
    }

    // Convert simple tap-for-mana activated abilities into mana abilities.
    // This covers basic lands and any rock with `{T}: Add {N mana}`.
    // Multi-step costs (e.g. Evolving Wilds's tap+sacrifice) are intentionally
    // excluded — those are activated abilities, not mana abilities.
    for ability in &def.abilities {
        if let AbilityDefinition::Activated { cost, effect, .. } = ability {
            if matches!(cost, Cost::Tap) {
                if let Some(ma) = try_as_tap_mana_ability(effect) {
                    spec = spec.with_mana_ability(ma);
                }
            }
        }
    }

    // Populate non-mana activated abilities into characteristics.activated_abilities.
    // This is required so that Command::ActivateAbility can look up the ability by index.
    for ability in &def.abilities {
        if let AbilityDefinition::Activated {
            cost,
            effect,
            timing_restriction,
        } = ability
        {
            // Skip ALL tap-for-mana abilities (both fixed-mana and any-color variants).
            // These are either registered as ManaAbility above or handled via TapForMana.
            // Including them here would shift the ability_index of non-mana abilities.
            let is_tap_mana_ability = matches!(cost, Cost::Tap)
                && matches!(
                    effect,
                    Effect::AddMana { .. } | Effect::AddManaAnyColor { .. }
                );
            if !is_tap_mana_ability {
                let activation_cost = cost_to_activation_cost(cost);
                let ab = ActivatedAbility {
                    cost: activation_cost,
                    description: String::new(),
                    effect: Some(effect.clone()),
                    // CR 602.5d: Propagate timing restriction so sorcery-speed abilities
                    // (e.g., Equip) are enforced in handle_activate_ability.
                    sorcery_speed: matches!(
                        timing_restriction,
                        Some(TimingRestriction::SorcerySpeed)
                    ),
                };
                spec = spec.with_activated_ability(ab);
            }
        }
    }

    // CR 603.6c / CR 700.4: Convert "When ~ dies" card-definition triggers into
    // runtime TriggeredAbilityDef entries so check_triggers can dispatch them.
    // This covers self-referential dies triggers (e.g. Solemn Simulacrum).
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenDies,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::SelfDies,
                intervening_if: None,
                description: "When ~ dies (CR 700.4)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }

    // CR 508.1m / CR 508.3a: Convert "Whenever ~ attacks" card-definition triggers into
    // runtime TriggeredAbilityDef entries so check_triggers can dispatch them.
    // This covers self-referential attack triggers (e.g. Audacious Thief).
    // Note: CR 508.4 — creatures put onto the battlefield attacking do NOT trigger
    // "whenever ~ attacks"; they were never declared as attackers. The engine correctly
    // dispatches SelfAttacks only from AttackersDeclared events (not from any other
    // mechanism), so this enrichment path is safe.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenAttacks,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::SelfAttacks,
                intervening_if: None,
                description: "Whenever ~ attacks (CR 508.3a)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }

    // CR 509.1: Convert "Whenever ~ blocks" card-definition triggers into runtime
    // TriggeredAbilityDef entries so check_triggers can dispatch them.
    // This covers self-referential block triggers (e.g. a creature with "Whenever ~ blocks").
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenBlocks,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::SelfBlocks,
                intervening_if: None,
                description: "Whenever ~ blocks (CR 509.1)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }

    // CR 510.3a / CR 603.2: Convert "Whenever ~ deals combat damage to a player"
    // card-definition triggers into runtime TriggeredAbilityDef entries so
    // check_triggers can dispatch them via CombatDamageDealt events.
    // intervening_if is None here: Condition and InterveningIf are separate types;
    // conditional combat-damage triggers are rare and deferred.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::SelfDealsCombatDamageToPlayer,
                intervening_if: None,
                description: "Whenever ~ deals combat damage to a player (CR 510.3a)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }

    // CR 603.2 / CR 102.2: Convert "Whenever an opponent casts a spell"
    // card-definition triggers into runtime TriggeredAbilityDef entries so
    // check_triggers can dispatch them via SpellCast events.
    // The opponent check is done at trigger-collection time in abilities.rs,
    // not here -- this only wires the TriggerEvent.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverOpponentCastsSpell,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::OpponentCastsSpell,
                intervening_if: None,
                description: "Whenever an opponent casts a spell (CR 603.2)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }

    spec
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn resolve_targets(
    targets: &[ActionTarget],
    state: &GameState,
    players: &HashMap<String, PlayerId>,
) -> Vec<crate::state::Target> {
    targets
        .iter()
        .filter_map(|t| {
            match t.target_type.as_str() {
                "player" => {
                    let pname = t.player.as_deref()?;
                    players
                        .get(pname)
                        .map(|&pid| crate::state::Target::Player(pid))
                }
                "spell" => {
                    let cname = t.card.as_deref()?;
                    // Look for the named object on the stack.
                    let obj_id = state.objects.iter().find_map(|(&id, obj)| {
                        if obj.characteristics.name == cname && obj.zone == ZoneId::Stack {
                            Some(id)
                        } else {
                            None
                        }
                    })?;
                    Some(crate::state::Target::Object(obj_id))
                }
                "permanent" | "creature" | "artifact" | "enchantment" | "card" => {
                    let cname = t.card.as_deref()?;
                    // Look for the named permanent on the battlefield.
                    let obj_id = state.objects.iter().find_map(|(&id, obj)| {
                        if obj.characteristics.name == cname && obj.zone == ZoneId::Battlefield {
                            Some(id)
                        } else {
                            None
                        }
                    })?;
                    Some(crate::state::Target::Object(obj_id))
                }
                _ => None,
            }
        })
        .collect()
}

fn find_in_hand(state: &GameState, player: PlayerId, name: &str) -> Option<crate::state::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == ZoneId::Hand(player) {
            Some(id)
        } else {
            None
        }
    })
}

/// CR 702.34a: Find a named card in a player's graveyard (for flashback casting).
fn find_in_graveyard(
    state: &GameState,
    player: PlayerId,
    name: &str,
) -> Option<crate::state::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == ZoneId::Graveyard(player) {
            Some(id)
        } else {
            None
        }
    })
}

fn find_on_battlefield(
    state: &GameState,
    controller: PlayerId,
    name: &str,
) -> Option<crate::state::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name
            && obj.zone == ZoneId::Battlefield
            && obj.controller == controller
        {
            Some(id)
        } else {
            None
        }
    })
}

/// If `effect` is `AddMana` with exactly one non-zero single-color entry,
/// return a corresponding `ManaAbility::tap_for`. Returns `None` otherwise.
///
/// This covers all 5 basic land colors (produces exactly 1 mana of one color).
/// Sol Ring ({T}: Add {CC}) produces 2 colorless — handled via ActivateAbility
/// in scripts instead of TapForMana.
fn try_as_tap_mana_ability(effect: &Effect) -> Option<ManaAbility> {
    if let Effect::AddMana { mana, .. } = effect {
        // Collect all non-zero color entries (supports multi-mana like Sol Ring's {CC}).
        let color_amounts = [
            (ManaColor::White, mana.white),
            (ManaColor::Blue, mana.blue),
            (ManaColor::Black, mana.black),
            (ManaColor::Red, mana.red),
            (ManaColor::Green, mana.green),
            (ManaColor::Colorless, mana.colorless),
        ];
        let non_zero: Vec<_> = color_amounts
            .iter()
            .filter(|(_, amount)| *amount > 0)
            .collect();

        // Must produce at least one mana in at least one color.
        if non_zero.is_empty() {
            return None;
        }

        let mut produces = OrdMap::new();
        for (color, amount) in &non_zero {
            produces.insert(*color, *amount);
        }
        return Some(ManaAbility {
            produces,
            requires_tap: true,
        });
    }
    None
}

/// Convert a card-definition [`Cost`] into an [`ActivationCost`] for object characteristics.
///
/// Handles `Tap`, `Mana`, `Sacrifice`, and `Sequence` (recursively). Unrecognised
/// cost components (`PayLife`, `DiscardCard`) are silently ignored — they have no
/// representation in `ActivationCost` yet.
fn cost_to_activation_cost(cost: &Cost) -> ActivationCost {
    let mut ac = ActivationCost::default();
    flatten_cost_into(cost, &mut ac);
    ac
}

fn flatten_cost_into(cost: &Cost, ac: &mut ActivationCost) {
    match cost {
        Cost::Tap => ac.requires_tap = true,
        Cost::Mana(m) => ac.mana_cost = Some(m.clone()),
        Cost::Sacrifice(_) => ac.sacrifice_self = true,
        Cost::Sequence(costs) => costs.iter().for_each(|c| flatten_cost_into(c, ac)),
        Cost::PayLife(_) | Cost::DiscardCard => {} // no ActivationCost representation yet
    }
}

fn parse_mana_color(s: &str) -> Option<ManaColor> {
    match s.to_lowercase().as_str() {
        "white" | "w" => Some(ManaColor::White),
        "blue" | "u" => Some(ManaColor::Blue),
        "black" | "b" => Some(ManaColor::Black),
        "red" | "r" => Some(ManaColor::Red),
        "green" | "g" => Some(ManaColor::Green),
        "colorless" | "c" => Some(ManaColor::Colorless),
        "generic" | "any" => Some(ManaColor::Colorless),
        _ => None,
    }
}
