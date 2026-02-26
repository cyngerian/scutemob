//! Effect resolution for spells and abilities (CR 608.2, 608.3b).
//!
//! The `execute_effect` function is the core of card effect execution.
//! It takes an `Effect` (from a `CardDefinition`), the execution context
//! (controller, source, declared targets), and the current `GameState`,
//! and applies the effect, returning a list of `GameEvent`s.
//!
//! # Partial legality (CR 608.2)
//! Effects skip declared targets that are no longer legal at resolution time
//! (partial fizzle). Full fizzle (ALL targets illegal) is handled by
//! `resolution.rs` before this function is called.
//!
//! # Interactive effects (M9+)
//! Effects requiring player choice (`SearchLibrary`, modal `Choose`) use a
//! deterministic fallback in M7: the engine picks the first matching option
//! alphabetically. Interactive choice requires the Command loop added in M9.
//!
//! # Damage semantics (CR 120)
//! Non-combat damage from spells/abilities marks `damage_marked` on permanents
//! and reduces life for players. Death from lethal damage is handled by SBAs
//! AFTER the effect resolves — the caller (resolution.rs) runs SBA checks.

use std::collections::HashMap;

use rand::SeedableRng;

use crate::cards::card_definition::{
    Condition, Effect, EffectAmount, EffectTarget, ForEachTarget, PlayerTarget, TargetFilter,
    ZoneTarget,
};
use crate::rules::events::{CombatDamageTarget, GameEvent};
use crate::state::game_object::{Characteristics, GameObject, ObjectId, ObjectStatus};
use crate::state::player::PlayerId;
use crate::state::targeting::{SpellTarget, Target};
use crate::state::types::{CardType, KeywordAbility, ManaColor};
use crate::state::zone::{ZoneId, ZoneType};
use crate::state::GameState;

// ── Effect execution context ──────────────────────────────────────────────────

/// Context for executing an effect.
///
/// Carries the controller, source object, declared targets, and a remap table
/// for tracking ObjectId changes when targets move between zones during execution
/// (e.g. Swords to Plowshares: exile creature, then gain life equal to its power —
/// the "its" refers to the creature after it left the battlefield, so we track the
/// new ObjectId in exile for power lookup).
pub struct EffectContext {
    /// Controller of the spell or ability.
    pub controller: PlayerId,
    /// The source object (the card, ability source, or token). May no longer be on
    /// the battlefield at resolution time for instants/sorceries.
    pub source: ObjectId,
    /// Declared targets (captured at cast time, already zone-snapshot validated).
    pub targets: Vec<SpellTarget>,
    /// Maps declared target index → new ObjectId after a zone change.
    /// Updated by zone-change effects (ExileObject, MoveZone, etc.) so
    /// subsequent effects can still refer to the target's power/toughness/etc.
    pub target_remaps: HashMap<usize, ObjectId>,
}

impl EffectContext {
    /// Build a basic context from resolution data.
    pub fn new(controller: PlayerId, source: ObjectId, targets: Vec<SpellTarget>) -> Self {
        Self {
            controller,
            source,
            targets,
            target_remaps: HashMap::new(),
        }
    }

    /// Resolve a declared target to a player (if it's a player target).
    fn player_for_target(&self, index: usize) -> Option<PlayerId> {
        match self.targets.get(index)?.target {
            Target::Player(p) => Some(p),
            Target::Object(_) => None,
        }
    }
}

// ── Main entry point ──────────────────────────────────────────────────────────

/// Execute an `Effect`, modifying `state` and returning resulting `GameEvent`s.
///
/// Called from `resolution.rs` when a spell or ability resolves (CR 608.2/608.3b).
/// The caller is responsible for SBA checks AFTER this function returns.
///
/// # Arguments
/// * `state`  - mutable game state
/// * `effect` - the effect to execute (from `CardDefinition::abilities`)
/// * `ctx`    - execution context (controller, source, declared targets)
pub fn execute_effect(
    state: &mut GameState,
    effect: &Effect,
    ctx: &mut EffectContext,
) -> Vec<GameEvent> {
    let mut events = Vec::new();
    execute_effect_inner(state, effect, ctx, &mut events);
    events
}

fn execute_effect_inner(
    state: &mut GameState,
    effect: &Effect,
    ctx: &mut EffectContext,
    events: &mut Vec<GameEvent>,
) {
    match effect {
        // ── Damage & Life ──────────────────────────────────────────────────
        Effect::DealDamage { target, amount } => {
            // MR-M7-05: clamp negative amounts to 0 before cast to avoid wrapping.
            let dmg = resolve_amount(state, amount, ctx).max(0) as u32;
            if dmg == 0 {
                return;
            }
            let targets = resolve_effect_target_list(state, target, ctx);
            for resolved in targets {
                match resolved {
                    ResolvedTarget::Player(p) => {
                        // CR 615: check prevention before applying damage.
                        let damage_target = CombatDamageTarget::Player(p);
                        let (final_dmg, prev_events) =
                            crate::rules::replacement::apply_damage_prevention(
                                state,
                                ctx.source,
                                &damage_target,
                                dmg,
                            );
                        events.extend(prev_events);
                        if final_dmg > 0 {
                            if let Some(player) = state.players.get_mut(&p) {
                                player.life_total -= final_dmg as i32;
                            }
                            events.push(GameEvent::DamageDealt {
                                source: ctx.source,
                                target: damage_target,
                                amount: final_dmg,
                            });
                            events.push(GameEvent::LifeLost {
                                player: p,
                                amount: final_dmg,
                            });
                        }
                    }
                    ResolvedTarget::Object(id) => {
                        let card_types = state
                            .objects
                            .get(&id)
                            .map(|o| o.characteristics.card_types.clone())
                            .unwrap_or_default();

                        let damage_target = if card_types.contains(&CardType::Planeswalker) {
                            CombatDamageTarget::Planeswalker(id)
                        } else {
                            CombatDamageTarget::Creature(id)
                        };

                        // CR 702.16e + CR 615: apply_damage_prevention checks protection
                        // (static) then dynamic prevention shields in order.
                        let (final_dmg, prev_events) =
                            crate::rules::replacement::apply_damage_prevention(
                                state,
                                ctx.source,
                                &damage_target,
                                dmg,
                            );
                        events.extend(prev_events);

                        if final_dmg > 0 {
                            if card_types.contains(&CardType::Planeswalker) {
                                // CR 120.3c: damage to planeswalker removes loyalty counters.
                                if let Some(obj) = state.objects.get_mut(&id) {
                                    let cur = obj
                                        .counters
                                        .get(&crate::state::types::CounterType::Loyalty)
                                        .copied()
                                        .unwrap_or(0);
                                    let new_val = cur.saturating_sub(final_dmg);
                                    obj.counters
                                        .insert(crate::state::types::CounterType::Loyalty, new_val);
                                }
                            } else {
                                // CR 120.3b: damage to a creature marks damage_marked.
                                if let Some(obj) = state.objects.get_mut(&id) {
                                    obj.damage_marked += final_dmg;
                                    // CR 702.2: deathtouch — mark for SBA.
                                    // (deathtouch_damage field exists on the source; for spell
                                    // damage we do not set deathtouch_damage here — spell damage
                                    // sources don't have the deathtouch keyword in this context.)
                                }
                            }
                            events.push(GameEvent::DamageDealt {
                                source: ctx.source,
                                target: damage_target,
                                amount: final_dmg,
                            });
                        }
                    }
                }
            }
        }

        Effect::GainLife { player, amount } => {
            // MR-M7-05: clamp negative amounts to 0 before cast to avoid wrapping.
            let gain = resolve_amount(state, amount, ctx).max(0) as u32;
            if gain == 0 {
                return;
            }
            let players = resolve_player_target_list(state, player, ctx);
            for p in players {
                if let Some(ps) = state.players.get_mut(&p) {
                    ps.life_total += gain as i32;
                }
                events.push(GameEvent::LifeGained {
                    player: p,
                    amount: gain,
                });
            }
        }

        Effect::LoseLife { player, amount } => {
            // MR-M7-05: clamp negative amounts to 0 before cast to avoid wrapping.
            let loss = resolve_amount(state, amount, ctx).max(0) as u32;
            if loss == 0 {
                return;
            }
            let players = resolve_player_target_list(state, player, ctx);
            for p in players {
                if let Some(ps) = state.players.get_mut(&p) {
                    ps.life_total -= loss as i32;
                }
                events.push(GameEvent::LifeLost {
                    player: p,
                    amount: loss,
                });
            }
        }

        // ── Cards ──────────────────────────────────────────────────────────
        Effect::DrawCards { player, count } => {
            // MR-M7-05: clamp negative amounts to 0 before cast to avoid wrapping.
            let n = resolve_amount(state, count, ctx).max(0) as usize;
            let players = resolve_player_target_list(state, player, ctx);
            for p in players {
                for _ in 0..n {
                    let draw_evts = draw_one_card(state, p);
                    events.extend(draw_evts);
                }
            }
        }

        Effect::DiscardCards { player, count } => {
            let n = resolve_amount(state, count, ctx) as usize;
            let players = resolve_player_target_list(state, player, ctx);
            for p in players {
                discard_cards(state, p, n, events);
            }
        }

        Effect::MillCards { player, count } => {
            let n = resolve_amount(state, count, ctx) as usize;
            let players = resolve_player_target_list(state, player, ctx);
            for p in players {
                mill_cards(state, p, n, events);
            }
        }

        // ── Permanents ────────────────────────────────────────────────────
        Effect::CreateToken { spec } => {
            for _ in 0..spec.count {
                let obj = make_token(spec, ctx.controller);
                if let Ok(id) = state.add_object(obj, ZoneId::Battlefield) {
                    events.push(GameEvent::TokenCreated {
                        player: ctx.controller,
                        object_id: id,
                    });
                    events.push(GameEvent::PermanentEnteredBattlefield {
                        player: ctx.controller,
                        object_id: id,
                    });
                }
            }
        }

        Effect::DestroyPermanent { target } => {
            let targets = resolve_effect_target_list(state, target, ctx);
            for resolved in targets {
                if let ResolvedTarget::Object(id) = resolved {
                    // CR 702.12: indestructible permanents can't be destroyed.
                    let indestructible = state
                        .objects
                        .get(&id)
                        .map(|o| {
                            o.characteristics
                                .keywords
                                .contains(&KeywordAbility::Indestructible)
                        })
                        .unwrap_or(false);
                    if indestructible {
                        continue;
                    }
                    let card_types = state
                        .objects
                        .get(&id)
                        .map(|o| o.characteristics.card_types.clone())
                        .unwrap_or_default();
                    let owner = state
                        .objects
                        .get(&id)
                        .map(|o| o.owner)
                        .unwrap_or(ctx.controller);

                    // CR 614: Check replacement effects before moving to graveyard.
                    let action = crate::rules::replacement::check_zone_change_replacement(
                        state,
                        id,
                        ZoneType::Battlefield,
                        ZoneType::Graveyard,
                        owner,
                        &std::collections::HashSet::new(),
                    );

                    match action {
                        crate::rules::replacement::ZoneChangeAction::Redirect {
                            to: dest,
                            events: repl_events,
                            ..
                        } => {
                            events.extend(repl_events);
                            if let Ok((new_id, _old)) = state.move_object_to_zone(id, dest) {
                                match dest {
                                    ZoneId::Exile => {
                                        events.push(GameEvent::ObjectExiled {
                                            player: owner,
                                            object_id: id,
                                            new_exile_id: new_id,
                                        });
                                    }
                                    ZoneId::Command(_) => {
                                        // Commander redirected — no destruction event.
                                    }
                                    _ => {
                                        if card_types.contains(&CardType::Creature) {
                                            events.push(GameEvent::CreatureDied {
                                                object_id: id,
                                                new_grave_id: new_id,
                                            });
                                        } else {
                                            events.push(GameEvent::PermanentDestroyed {
                                                object_id: id,
                                                new_grave_id: new_id,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        // CR 616.1: Multiple replacements — defer until player chooses.
                        crate::rules::replacement::ZoneChangeAction::ChoiceRequired {
                            player,
                            choices,
                            event_description,
                        } => {
                            use crate::state::replacement_effect::PendingZoneChange;
                            state.pending_zone_changes.push_back(PendingZoneChange {
                                object_id: id,
                                original_from: ZoneType::Battlefield,
                                original_destination: ZoneType::Graveyard,
                                affected_player: player,
                                already_applied: Vec::new(),
                            });
                            events.push(GameEvent::ReplacementChoiceRequired {
                                player,
                                event_description,
                                choices,
                            });
                        }
                        crate::rules::replacement::ZoneChangeAction::Proceed => {
                            // No replacement — move to graveyard normally.
                            if let Ok((new_id, _old)) =
                                state.move_object_to_zone(id, ZoneId::Graveyard(owner))
                            {
                                if card_types.contains(&CardType::Creature) {
                                    events.push(GameEvent::CreatureDied {
                                        object_id: id,
                                        new_grave_id: new_id,
                                    });
                                } else {
                                    events.push(GameEvent::PermanentDestroyed {
                                        object_id: id,
                                        new_grave_id: new_id,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Effect::ExileObject { target } => {
            let targets = resolve_effect_target_list_indexed(state, target, ctx);
            for (idx_opt, resolved) in targets {
                match resolved {
                    ResolvedTarget::Object(id) => {
                        let owner = state
                            .objects
                            .get(&id)
                            .map(|o| o.owner)
                            .unwrap_or(ctx.controller);

                        // CR 614: Check replacement effects before exiling.
                        let from_zone_type = state
                            .objects
                            .get(&id)
                            .map(|o| match o.zone {
                                ZoneId::Battlefield => ZoneType::Battlefield,
                                ZoneId::Graveyard(_) => ZoneType::Graveyard,
                                ZoneId::Hand(_) => ZoneType::Hand,
                                ZoneId::Library(_) => ZoneType::Library,
                                ZoneId::Stack => ZoneType::Stack,
                                ZoneId::Exile => ZoneType::Exile,
                                ZoneId::Command(_) => ZoneType::Command,
                            })
                            .unwrap_or(ZoneType::Battlefield);

                        let action = crate::rules::replacement::check_zone_change_replacement(
                            state,
                            id,
                            from_zone_type,
                            ZoneType::Exile,
                            owner,
                            &std::collections::HashSet::new(),
                        );

                        match action {
                            crate::rules::replacement::ZoneChangeAction::Redirect {
                                to: dest,
                                events: repl_events,
                                ..
                            } => {
                                events.extend(repl_events);
                                if let Ok((new_id, _old)) = state.move_object_to_zone(id, dest) {
                                    if let Some(idx) = idx_opt {
                                        ctx.target_remaps.insert(idx, new_id);
                                    }
                                    match dest {
                                        ZoneId::Command(_) => {
                                            // Commander redirected — no exile event.
                                        }
                                        _ => {
                                            events.push(GameEvent::ObjectExiled {
                                                player: ctx.controller,
                                                object_id: id,
                                                new_exile_id: new_id,
                                            });
                                        }
                                    }
                                }
                            }
                            // CR 616.1: Multiple replacements — defer until player chooses.
                            crate::rules::replacement::ZoneChangeAction::ChoiceRequired {
                                player,
                                choices,
                                event_description,
                            } => {
                                use crate::state::replacement_effect::PendingZoneChange;
                                state.pending_zone_changes.push_back(PendingZoneChange {
                                    object_id: id,
                                    original_from: from_zone_type,
                                    original_destination: ZoneType::Exile,
                                    affected_player: player,
                                    already_applied: Vec::new(),
                                });
                                events.push(GameEvent::ReplacementChoiceRequired {
                                    player,
                                    event_description,
                                    choices,
                                });
                            }
                            crate::rules::replacement::ZoneChangeAction::Proceed => {
                                // No replacement — exile normally.
                                if let Ok((new_id, _old)) =
                                    state.move_object_to_zone(id, ZoneId::Exile)
                                {
                                    if let Some(idx) = idx_opt {
                                        ctx.target_remaps.insert(idx, new_id);
                                    }
                                    events.push(GameEvent::ObjectExiled {
                                        player: ctx.controller,
                                        object_id: id,
                                        new_exile_id: new_id,
                                    });
                                }
                            }
                        }
                    }
                    ResolvedTarget::Player(_) => {
                        // Exiling a player is not a legal effect.
                    }
                }
            }
        }

        Effect::CounterSpell { target } => {
            // CR 701.5: Counter target spell on the stack.
            let targets = resolve_effect_target_list(state, target, ctx);
            for resolved in targets {
                if let ResolvedTarget::Object(id) = resolved {
                    // CR 702.21a: Find the stack object by direct ID match (for Ward
                    // triggers that pass the stack object's own ID as their target) OR
                    // by source_object match (for traditional CounterSpell usage).
                    let pos = state
                        .stack_objects
                        .iter()
                        .position(|so| {
                            so.id == id
                                || matches!(&so.kind, crate::state::stack::StackObjectKind::Spell { source_object } if *source_object == id)
                        });
                    if let Some(pos) = pos {
                        // CR 101.6: If the spell can't be countered, the CounterSpell
                        // has no effect on it (does as much as possible — CR 101.2).
                        if state.stack_objects[pos].cant_be_countered {
                            continue;
                        }
                        let stack_obj = state.stack_objects.remove(pos);
                        let controller = stack_obj.controller;
                        match stack_obj.kind {
                            crate::state::stack::StackObjectKind::Spell { source_object } => {
                                let owner = state
                                    .objects
                                    .get(&source_object)
                                    .map(|o| o.owner)
                                    .unwrap_or(controller);
                                if let Ok((new_id, _)) = state
                                    .move_object_to_zone(source_object, ZoneId::Graveyard(owner))
                                {
                                    events.push(GameEvent::SpellCountered {
                                        player: controller,
                                        stack_object_id: stack_obj.id,
                                        source_object_id: new_id,
                                    });
                                }
                            }
                            crate::state::stack::StackObjectKind::ActivatedAbility {
                                source_object,
                                ..
                            }
                            | crate::state::stack::StackObjectKind::TriggeredAbility {
                                source_object,
                                ..
                            } => {
                                // CR 701.5: Countering an ability removes it from the stack.
                                // Unlike spells, the source stays in its current zone.
                                events.push(GameEvent::SpellCountered {
                                    player: controller,
                                    stack_object_id: stack_obj.id,
                                    source_object_id: source_object,
                                });
                            }
                            _ => {
                                // Other stack object kinds (StormTrigger, etc.) are not
                                // currently counterable via ward.
                            }
                        }
                    }
                }
            }
        }

        Effect::TapPermanent { target } => {
            let targets = resolve_effect_target_list(state, target, ctx);
            for resolved in targets {
                if let ResolvedTarget::Object(id) = resolved {
                    if let Some(obj) = state.objects.get_mut(&id) {
                        if !obj.status.tapped {
                            obj.status.tapped = true;
                            let player = obj.controller;
                            events.push(GameEvent::PermanentTapped {
                                player,
                                object_id: id,
                            });
                        }
                    }
                }
            }
        }

        Effect::UntapPermanent { target } => {
            let targets = resolve_effect_target_list(state, target, ctx);
            for resolved in targets {
                if let ResolvedTarget::Object(id) = resolved {
                    if let Some(obj) = state.objects.get_mut(&id) {
                        if obj.status.tapped {
                            obj.status.tapped = false;
                            let player = obj.controller;
                            events.push(GameEvent::PermanentUntapped {
                                player,
                                object_id: id,
                            });
                        }
                    }
                }
            }
        }

        // ── Mana ──────────────────────────────────────────────────────────
        Effect::AddMana { player, mana } => {
            let players = resolve_player_target_list(state, player, ctx);
            // Collect mana colors/amounts before mutating state to avoid borrow conflicts.
            let mana_entries: Vec<(ManaColor, u32)> = [
                (ManaColor::White, mana.white),
                (ManaColor::Blue, mana.blue),
                (ManaColor::Black, mana.black),
                (ManaColor::Red, mana.red),
                (ManaColor::Green, mana.green),
                (ManaColor::Colorless, mana.colorless),
            ]
            .into_iter()
            .filter(|(_, amt)| *amt > 0)
            .collect();
            for p in players {
                if let Some(ps) = state.players.get_mut(&p) {
                    for &(color, amount) in &mana_entries {
                        ps.mana_pool.add(color, amount);
                        events.push(GameEvent::ManaAdded {
                            player: p,
                            color,
                            amount,
                        });
                    }
                }
            }
        }

        Effect::AddManaAnyColor { player } | Effect::AddManaChoice { player, .. } => {
            // M9+: interactive mana color choice. For now, add colorless.
            let players = resolve_player_target_list(state, player, ctx);
            for p in players {
                if let Some(ps) = state.players.get_mut(&p) {
                    ps.mana_pool.add(ManaColor::Colorless, 1);
                    events.push(GameEvent::ManaAdded {
                        player: p,
                        color: ManaColor::Colorless,
                        amount: 1,
                    });
                }
            }
        }

        // ── Counters ──────────────────────────────────────────────────────
        Effect::AddCounter {
            target,
            counter,
            count,
        } => {
            let counter = counter.clone();
            let count = *count;
            let targets = resolve_effect_target_list(state, target, ctx);
            for resolved in targets {
                if let ResolvedTarget::Object(id) = resolved {
                    if let Some(obj) = state.objects.get_mut(&id) {
                        let cur = obj.counters.get(&counter).copied().unwrap_or(0);
                        obj.counters.insert(counter.clone(), cur + count);
                        events.push(GameEvent::CounterAdded {
                            object_id: id,
                            counter: counter.clone(),
                            count,
                        });
                    }
                }
            }
        }

        Effect::RemoveCounter {
            target,
            counter,
            count,
        } => {
            let counter = counter.clone();
            let count = *count;
            let targets = resolve_effect_target_list(state, target, ctx);
            for resolved in targets {
                if let ResolvedTarget::Object(id) = resolved {
                    if let Some(obj) = state.objects.get_mut(&id) {
                        let cur = obj.counters.get(&counter).copied().unwrap_or(0);
                        let new_val = cur.saturating_sub(count);
                        if new_val == 0 {
                            obj.counters.remove(&counter);
                        } else {
                            obj.counters.insert(counter.clone(), new_val);
                        }
                        events.push(GameEvent::CounterRemoved {
                            object_id: id,
                            counter: counter.clone(),
                            count: count.min(cur),
                        });
                    }
                }
            }
        }

        // ── Zone ──────────────────────────────────────────────────────────
        Effect::MoveZone { target, to } => {
            // MR-M7-04: resolve zone using owner PlayerTarget (not always controller).
            // MR-M7-01: emit destination-correct event instead of always ObjectExiled.
            let targets = resolve_effect_target_list_indexed(state, target, ctx);
            for (idx_opt, resolved) in targets {
                if let ResolvedTarget::Object(id) = resolved {
                    let dest = resolve_zone_target(to, state, ctx);
                    if let Ok((new_id, _)) = state.move_object_to_zone(id, dest) {
                        if let Some(idx) = idx_opt {
                            ctx.target_remaps.insert(idx, new_id);
                        }
                        let event = match dest {
                            ZoneId::Exile => GameEvent::ObjectExiled {
                                player: ctx.controller,
                                object_id: id,
                                new_exile_id: new_id,
                            },
                            ZoneId::Battlefield => GameEvent::PermanentEnteredBattlefield {
                                player: ctx.controller,
                                object_id: new_id,
                            },
                            ZoneId::Graveyard(_) => GameEvent::ObjectPutInGraveyard {
                                player: ctx.controller,
                                object_id: id,
                                new_grave_id: new_id,
                            },
                            ZoneId::Hand(_) => GameEvent::ObjectReturnedToHand {
                                player: ctx.controller,
                                object_id: id,
                                new_hand_id: new_id,
                            },
                            ZoneId::Library(_) => GameEvent::ObjectPutOnLibrary {
                                player: ctx.controller,
                                object_id: id,
                                new_lib_id: new_id,
                            },
                            // Command zone and stack: rare edge case, emit generic exile-like event.
                            ZoneId::Command(_) | ZoneId::Stack => GameEvent::ObjectExiled {
                                player: ctx.controller,
                                object_id: id,
                                new_exile_id: new_id,
                            },
                        };
                        events.push(event);
                    }
                }
            }
        }

        // ── Library ───────────────────────────────────────────────────────
        Effect::PutOnLibrary {
            player,
            count,
            from,
        } => {
            // CR 701.20: Put N cards from the source zone onto the top of a library.
            // M7 deterministic: takes the first N objects (by ObjectId ascending).
            let n = resolve_amount(state, count, ctx).max(0) as usize;
            let players = resolve_player_target_list(state, player, ctx);
            for p in players {
                let from_zone = resolve_zone_target(from, state, ctx);
                let lib_zone = ZoneId::Library(p);
                // Collect candidate objects from the source zone (deterministic order).
                let mut ids: Vec<ObjectId> = state
                    .objects
                    .iter()
                    .filter(|(_, obj)| obj.zone == from_zone)
                    .map(|(id, _)| *id)
                    .collect();
                ids.sort_by_key(|id| id.0);
                ids.truncate(n);
                for id in ids {
                    if let Ok((new_id, _)) = state.move_object_to_zone(id, lib_zone) {
                        events.push(GameEvent::ObjectPutOnLibrary {
                            player: p,
                            object_id: id,
                            new_lib_id: new_id,
                        });
                    }
                }
            }
        }

        Effect::SearchLibrary {
            player,
            filter,
            reveal: _,
            destination,
        } => {
            // M9+: interactive card search. For M7, deterministic fallback:
            // find the first matching card (by ObjectId, ascending) in the library.
            let players = resolve_player_target_list(state, player, ctx);
            for p in players {
                let lib_id = ZoneId::Library(p);
                let candidates: Vec<ObjectId> = state
                    .objects
                    .iter()
                    .filter(|(_, obj)| {
                        obj.zone == lib_id && matches_filter(&obj.characteristics, filter)
                    })
                    .map(|(id, _)| *id)
                    .collect();

                if let Some(&card_id) = candidates.iter().min_by_key(|&&id| id.0) {
                    // MR-M7-10: check tapped flag before resolving to ZoneId.
                    // MR-M7-04: resolve owner from PlayerTarget, not blindly controller.
                    let tapped_opt = dest_tapped(destination);
                    let dest = resolve_zone_target(destination, state, ctx);
                    if let Some(tap) = tapped_opt {
                        if let Ok((new_id, _)) = state.move_object_to_zone(card_id, dest) {
                            // If the destination is battlefield tapped, apply tapped status.
                            if tap {
                                if let Some(obj) = state.objects.get_mut(&new_id) {
                                    obj.status.tapped = true;
                                }
                            }
                            events.push(GameEvent::PermanentEnteredBattlefield {
                                player: ctx.controller,
                                object_id: new_id,
                            });
                        }
                    } else {
                        // Hand or graveyard or other zone.
                        let _ = state.move_object_to_zone(card_id, dest);
                    }
                }
            }
        }

        // CR 701.18: Scry N — deterministic fallback: put top N cards on bottom
        // in ObjectId ascending order (interactive ordering deferred to M10+).
        Effect::Scry { player, count } => {
            let n = resolve_amount(state, count, ctx).max(0) as usize;
            let players = resolve_player_target_list(state, player, ctx);
            for p in players {
                let lib_zone = ZoneId::Library(p);
                // Collect the top N cards of the library (ordered from top).
                let top_ids: Vec<ObjectId> = state
                    .zones
                    .get(&lib_zone)
                    .map(|z| z.object_ids())
                    .unwrap_or_default()
                    .into_iter()
                    .take(n)
                    .collect();
                // Deterministic fallback: sort by ObjectId and move to bottom.
                let mut to_bottom = top_ids.clone();
                to_bottom.sort_by_key(|id| id.0);
                for id in to_bottom {
                    // Move to bottom by removing and re-inserting at the bottom
                    // (library zones are Ordered, so we use move_to_zone back).
                    let _ = state.move_object_to_zone(id, lib_zone);
                }
                events.push(GameEvent::Scried {
                    player: p,
                    count: n as u32,
                });
            }
        }

        Effect::Shuffle { player } => {
            // MR-M7-17: use timestamp_counter as seed instead of from_entropy() so
            // shuffles are deterministic given the same game state sequence.
            let players = resolve_player_target_list(state, player, ctx);
            for p in players {
                let seed = state.timestamp_counter;
                state.timestamp_counter += 1;
                if let Some(zone) = state.zones.get_mut(&ZoneId::Library(p)) {
                    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
                    zone.shuffle(&mut rng);
                }
                events.push(GameEvent::LibraryShuffled { player: p });
            }
        }

        // ── Continuous Effects ─────────────────────────────────────────────
        Effect::ApplyContinuousEffect { effect_def } => {
            use crate::state::continuous_effect::EffectFilter as CEFilter;
            // Resolve DeclaredTarget filter to a specific object at runtime (CR 611.2a).
            let resolved_filter = match &effect_def.filter {
                CEFilter::DeclaredTarget { index } => {
                    // Resolve declared target index to a SingleObject filter.
                    let obj_id = ctx.targets.get(*index).and_then(|t| match t.target {
                        Target::Object(id) => Some(id),
                        Target::Player(_) => None,
                    });
                    match obj_id {
                        Some(id) => CEFilter::SingleObject(id),
                        None => return, // No valid target; skip effect.
                    }
                }
                // CR 702.108a: Prowess and similar "this permanent" effects use Source
                // as a placeholder. Resolve it to the source object at execution time.
                CEFilter::Source => CEFilter::SingleObject(ctx.source),
                other => other.clone(),
            };
            // Build a ContinuousEffect from the definition and add it to state.
            let id_inner = state.next_object_id().0;
            let ts = state.timestamp_counter;
            let source = ctx.source;
            let eff = crate::state::continuous_effect::ContinuousEffect {
                id: crate::state::continuous_effect::EffectId(id_inner),
                source: Some(source),
                layer: effect_def.layer,
                modification: effect_def.modification.clone(),
                filter: resolved_filter,
                duration: effect_def.duration,
                is_cda: false,
                timestamp: ts,
            };
            state.continuous_effects.push_back(eff);
        }

        // ── Combinators ────────────────────────────────────────────────────
        Effect::Sequence(effects) => {
            for e in effects {
                execute_effect_inner(state, e, ctx, events);
            }
        }

        Effect::Conditional {
            condition,
            if_true,
            if_false,
        } => {
            if check_condition(state, condition, ctx) {
                execute_effect_inner(state, if_true, ctx, events);
            } else {
                execute_effect_inner(state, if_false, ctx, events);
            }
        }

        Effect::ForEach { over, effect } => {
            // MR-M7-06: handle player-based ForEach targets separately — collect_for_each
            // returns Vec<ObjectId> and cannot represent players.
            match over {
                ForEachTarget::EachPlayer | ForEachTarget::EachOpponent => {
                    let player_target = match over {
                        ForEachTarget::EachPlayer => PlayerTarget::EachPlayer,
                        ForEachTarget::EachOpponent => PlayerTarget::EachOpponent,
                        _ => unreachable!(),
                    };
                    let players = resolve_player_target_list(state, &player_target, ctx);
                    for p in players {
                        let mut inner_ctx = EffectContext {
                            controller: ctx.controller,
                            source: ctx.source,
                            targets: vec![SpellTarget {
                                target: Target::Player(p),
                                zone_at_cast: None,
                            }],
                            target_remaps: HashMap::new(),
                        };
                        execute_effect_inner(state, effect, &mut inner_ctx, events);
                    }
                }
                _ => {
                    let collection = collect_for_each(state, over, ctx);
                    for id in collection {
                        // Build a synthetic single-object context for the inner effect.
                        let mut inner_ctx = EffectContext {
                            controller: ctx.controller,
                            source: ctx.source,
                            targets: vec![SpellTarget {
                                target: Target::Object(id),
                                zone_at_cast: Some(ZoneId::Battlefield),
                            }],
                            target_remaps: HashMap::new(),
                        };
                        execute_effect_inner(state, effect, &mut inner_ctx, events);
                    }
                }
            }
        }

        Effect::Choose { choices, .. } => {
            // M9+: interactive modal choice. For M7, execute the first option.
            if let Some(first) = choices.first() {
                execute_effect_inner(state, first, ctx, events);
            }
        }

        Effect::MayPayOrElse { or_else, .. } => {
            // M9+: interactive choice to pay or not. For M7, don't pay → apply or_else.
            execute_effect_inner(state, or_else, ctx, events);
        }

        // CR 701.15a: Goad — mark the target creature as goaded until the start of
        // the goaded creature controller's next turn. The goaded creature must attack
        // each combat if able (CR 701.15b) and must attack a player other than the
        // goading player if able (CR 701.15b). The goading player is stored in
        // `goaded_by` on the GameObject for combat enforcement.
        Effect::Goad { target } => {
            let targets = resolve_effect_target_list(state, target, ctx);
            for resolved in targets {
                if let ResolvedTarget::Object(id) = resolved {
                    if let Some(obj) = state.objects.get_mut(&id) {
                        if !obj.goaded_by.contains(&ctx.controller) {
                            obj.goaded_by.push_back(ctx.controller);
                        }
                        events.push(GameEvent::Goaded {
                            object_id: id,
                            goading_player: ctx.controller,
                        });
                    }
                }
            }
        }

        Effect::Nothing => {}
    }
}

// ── Target resolution helpers ─────────────────────────────────────────────────

/// A resolved target: either a player or an object.
#[derive(Clone, Debug)]
enum ResolvedTarget {
    Player(PlayerId),
    Object(ObjectId),
}

/// Resolve an `EffectTarget` into a list of `ResolvedTarget`s.
fn resolve_effect_target_list(
    state: &GameState,
    target: &EffectTarget,
    ctx: &EffectContext,
) -> Vec<ResolvedTarget> {
    resolve_effect_target_list_indexed(state, target, ctx)
        .into_iter()
        .map(|(_, t)| t)
        .collect()
}

/// Like `resolve_effect_target_list` but also returns the declared-target index
/// for each target (used to update `ctx.target_remaps` on zone changes).
fn resolve_effect_target_list_indexed(
    state: &GameState,
    target: &EffectTarget,
    ctx: &EffectContext,
) -> Vec<(Option<usize>, ResolvedTarget)> {
    match target {
        EffectTarget::DeclaredTarget { index } => {
            let idx = *index;
            // Use remap first, then fall back to original declared target.
            if let Some(&remapped_id) = ctx.target_remaps.get(&idx) {
                return vec![(Some(idx), ResolvedTarget::Object(remapped_id))];
            }
            match ctx.targets.get(idx) {
                Some(SpellTarget {
                    target: Target::Object(id),
                    ..
                }) => {
                    // Only return if the target object still exists (partial fizzle skip).
                    // CR 702.21a: Also accept IDs that refer to stack objects (e.g., the
                    // stack object representing the spell or ability Ward is countering).
                    // Stack entries are NOT in state.objects, but CounterSpell handles them.
                    let exists_in_objects = state.objects.contains_key(id);
                    let exists_on_stack = state.stack_objects.iter().any(|so| so.id == *id);
                    if exists_in_objects || exists_on_stack {
                        vec![(Some(idx), ResolvedTarget::Object(*id))]
                    } else {
                        vec![]
                    }
                }
                Some(SpellTarget {
                    target: Target::Player(p),
                    ..
                }) => {
                    if state.players.get(p).map(|ps| !ps.has_lost).unwrap_or(false) {
                        vec![(None, ResolvedTarget::Player(*p))]
                    } else {
                        vec![]
                    }
                }
                None => vec![],
            }
        }
        EffectTarget::Controller => {
            vec![(None, ResolvedTarget::Player(ctx.controller))]
        }
        EffectTarget::Source => {
            if state.objects.contains_key(&ctx.source) {
                vec![(None, ResolvedTarget::Object(ctx.source))]
            } else {
                vec![]
            }
        }
        EffectTarget::EachPlayer => state
            .players
            .keys()
            .filter(|&&p| {
                state
                    .players
                    .get(&p)
                    .map(|ps| !ps.has_lost)
                    .unwrap_or(false)
            })
            .map(|&p| (None, ResolvedTarget::Player(p)))
            .collect(),
        EffectTarget::EachOpponent => state
            .players
            .keys()
            .filter(|&&p| {
                p != ctx.controller
                    && state
                        .players
                        .get(&p)
                        .map(|ps| !ps.has_lost)
                        .unwrap_or(false)
            })
            .map(|&p| (None, ResolvedTarget::Player(p)))
            .collect(),
        EffectTarget::AllCreatures => state
            .objects
            .iter()
            .filter(|(_, obj)| {
                obj.zone == ZoneId::Battlefield
                    && obj.characteristics.card_types.contains(&CardType::Creature)
            })
            .map(|(&id, _)| (None, ResolvedTarget::Object(id)))
            .collect(),
        EffectTarget::AllPermanents => state
            .objects
            .iter()
            .filter(|(_, obj)| obj.zone == ZoneId::Battlefield)
            .map(|(&id, _)| (None, ResolvedTarget::Object(id)))
            .collect(),
        EffectTarget::AllPermanentsMatching(filter) => state
            .objects
            .iter()
            .filter(|(_, obj)| {
                obj.zone == ZoneId::Battlefield && matches_filter(&obj.characteristics, filter)
            })
            .map(|(&id, _)| (None, ResolvedTarget::Object(id)))
            .collect(),
    }
}

/// Resolve a `PlayerTarget` into a list of `PlayerId`s.
fn resolve_player_target_list(
    state: &GameState,
    player: &PlayerTarget,
    ctx: &EffectContext,
) -> Vec<PlayerId> {
    match player {
        PlayerTarget::Controller => vec![ctx.controller],
        PlayerTarget::EachPlayer => state
            .players
            .keys()
            .filter(|&&p| {
                state
                    .players
                    .get(&p)
                    .map(|ps| !ps.has_lost)
                    .unwrap_or(false)
            })
            .copied()
            .collect(),
        PlayerTarget::EachOpponent => state
            .players
            .keys()
            .filter(|&&p| {
                p != ctx.controller
                    && state
                        .players
                        .get(&p)
                        .map(|ps| !ps.has_lost)
                        .unwrap_or(false)
            })
            .copied()
            .collect(),
        PlayerTarget::DeclaredTarget { index } => {
            // Must be a player target.
            if let Some(p) = ctx.player_for_target(*index) {
                if state
                    .players
                    .get(&p)
                    .map(|ps| !ps.has_lost)
                    .unwrap_or(false)
                {
                    return vec![p];
                }
            }
            vec![]
        }
        PlayerTarget::ControllerOf(effect_target) => {
            // Find the controller of the specified object.
            // CR 702.21a: The target may be a stack object (e.g., the spell or ability that
            // triggered ward). Check state.objects first (battlefield/graveyard/etc.), then
            // fall back to state.stack_objects for spells/abilities still on the stack.
            let targets = resolve_effect_target_list(state, effect_target, ctx);
            targets
                .into_iter()
                .filter_map(|t| {
                    if let ResolvedTarget::Object(id) = t {
                        // Check battlefield objects first.
                        if let Some(obj) = state.objects.get(&id) {
                            return Some(obj.controller);
                        }
                        // Fall back to stack objects (e.g., targeting spell for ward).
                        state
                            .stack_objects
                            .iter()
                            .find(|so| so.id == id)
                            .map(|so| so.controller)
                    } else {
                        None
                    }
                })
                .collect()
        }
    }
}

// ── Amount resolution ─────────────────────────────────────────────────────────

/// Resolve an `EffectAmount` to a concrete integer value.
fn resolve_amount(state: &GameState, amount: &EffectAmount, ctx: &EffectContext) -> i32 {
    match amount {
        EffectAmount::Fixed(n) => *n,
        EffectAmount::XValue => 0, // M9+: X-cost support
        EffectAmount::PowerOf(target) => {
            let targets = resolve_effect_target_list(state, target, ctx);
            targets
                .into_iter()
                .filter_map(|t| {
                    if let ResolvedTarget::Object(id) = t {
                        // Use remap to find the object even if it has moved zones.
                        state
                            .objects
                            .get(&id)
                            .and_then(|obj| obj.characteristics.power)
                    } else {
                        None
                    }
                })
                .next()
                .unwrap_or(0)
        }
        EffectAmount::ToughnessOf(target) => {
            let targets = resolve_effect_target_list(state, target, ctx);
            targets
                .into_iter()
                .filter_map(|t| {
                    if let ResolvedTarget::Object(id) = t {
                        state
                            .objects
                            .get(&id)
                            .and_then(|obj| obj.characteristics.toughness)
                    } else {
                        None
                    }
                })
                .next()
                .unwrap_or(0)
        }
        EffectAmount::ManaValueOf(target) => {
            let targets = resolve_effect_target_list(state, target, ctx);
            targets
                .into_iter()
                .filter_map(|t| {
                    if let ResolvedTarget::Object(id) = t {
                        state.objects.get(&id).map(|obj| {
                            obj.characteristics
                                .mana_cost
                                .as_ref()
                                .map(|mc| {
                                    (mc.white
                                        + mc.blue
                                        + mc.black
                                        + mc.red
                                        + mc.green
                                        + mc.colorless
                                        + mc.generic) as i32
                                })
                                .unwrap_or(0)
                        })
                    } else {
                        None
                    }
                })
                .next()
                .unwrap_or(0)
        }
        EffectAmount::CardCount {
            zone,
            player: _,
            filter,
        } => {
            // MR-M7-07: count objects in the specified zone matching the filter.
            // Resolve the zone target using the current controller context.
            let zone_id = resolve_zone_target(zone, state, ctx);
            state
                .objects
                .values()
                .filter(|obj| {
                    obj.zone == zone_id
                        && filter
                            .as_ref()
                            .map(|f| matches_filter(&obj.characteristics, f))
                            .unwrap_or(true)
                })
                .count() as i32
        }
    }
}

// ── Zone resolution helpers ───────────────────────────────────────────────────

/// Convert a `ZoneTarget` to a concrete `ZoneId`, resolving the owner `PlayerTarget`
/// via the execution context (MR-M7-04: previously always used controller).
fn resolve_zone_target(zone: &ZoneTarget, state: &GameState, ctx: &EffectContext) -> ZoneId {
    // Resolve the owner PlayerTarget to a concrete PlayerId, falling back to controller.
    let resolve_owner = |owner: &PlayerTarget| -> PlayerId {
        resolve_player_target_list(state, owner, ctx)
            .into_iter()
            .next()
            .unwrap_or(ctx.controller)
    };
    match zone {
        ZoneTarget::Battlefield { .. } => ZoneId::Battlefield,
        ZoneTarget::Graveyard { owner } => ZoneId::Graveyard(resolve_owner(owner)),
        ZoneTarget::Hand { owner } => ZoneId::Hand(resolve_owner(owner)),
        ZoneTarget::Library { owner, .. } => ZoneId::Library(resolve_owner(owner)),
        ZoneTarget::Exile => ZoneId::Exile,
        ZoneTarget::CommandZone => ZoneId::Command(ctx.controller),
    }
}

/// Returns `Some(tapped)` if the zone target is Battlefield (with the tapped flag),
/// else `None` for all other destinations (MR-M7-10: previously ignored the tapped field).
fn dest_tapped(zone: &ZoneTarget) -> Option<bool> {
    if let ZoneTarget::Battlefield { tapped } = zone {
        Some(*tapped)
    } else {
        None
    }
}

// ── Token creation ────────────────────────────────────────────────────────────

fn make_token(spec: &crate::cards::card_definition::TokenSpec, controller: PlayerId) -> GameObject {
    use crate::state::game_object::Characteristics;
    use im::OrdSet;

    let mut card_types = OrdSet::new();
    for ct in &spec.card_types {
        card_types.insert(*ct);
    }

    let mut keywords = OrdSet::new();
    for kw in &spec.keywords {
        keywords.insert(kw.clone());
    }

    let mut subtypes = im::OrdSet::new();
    for st in &spec.subtypes {
        subtypes.insert(st.clone());
    }

    let mut colors = OrdSet::new();
    for c in &spec.colors {
        colors.insert(*c);
    }

    let characteristics = Characteristics {
        name: spec.name.clone(),
        power: Some(spec.power),
        toughness: Some(spec.toughness),
        card_types,
        keywords,
        subtypes,
        colors,
        ..Characteristics::default()
    };

    let status = ObjectStatus {
        tapped: spec.tapped,
        ..ObjectStatus::default()
    };

    GameObject {
        id: ObjectId(0), // will be replaced by add_object
        card_id: None,
        characteristics,
        controller,
        owner: controller,
        zone: ZoneId::Battlefield,
        status,
        counters: im::OrdMap::new(),
        attachments: im::Vector::new(),
        attached_to: None,
        damage_marked: 0,
        deathtouch_damage: false,
        is_token: true,
        timestamp: 0,
        has_summoning_sickness: true, // tokens have summoning sickness (CR 302.6)
        enchants_creatures: false,
        goaded_by: im::Vector::new(),
    }
}

// ── Card draw helper ──────────────────────────────────────────────────────────

/// Draw one card for a player (CR 121.1). Returns events.
fn draw_one_card(state: &mut GameState, player: PlayerId) -> Vec<GameEvent> {
    // CR 614.11: Check WouldDraw replacement effects before performing the draw.
    // Shared logic lives in `replacement::check_would_draw_replacement` (MR-M8-07).
    {
        use crate::rules::replacement::{self, DrawAction};
        match replacement::check_would_draw_replacement(state, player) {
            DrawAction::Proceed => {}
            DrawAction::Skip(event) => return vec![event],
            DrawAction::NeedsChoice(event) => {
                // CR 616.1: Multiple WouldDraw replacements apply — defer the draw.
                return vec![event];
            }
        }
    }

    let lib_id = ZoneId::Library(player);
    let top = state.zones.get(&lib_id).and_then(|z| z.top());

    match top {
        None => {
            // CR 104.3b: drawing from empty library causes loss.
            if let Some(ps) = state.players.get_mut(&player) {
                ps.has_lost = true;
            }
            vec![GameEvent::PlayerLost {
                player,
                reason: crate::rules::events::LossReason::LibraryEmpty,
            }]
        }
        Some(card_id) => {
            if let Ok((new_id, _)) = state.move_object_to_zone(card_id, ZoneId::Hand(player)) {
                // CR 121.1: increment per-turn draw counter for Sylvan Library and similar effects.
                if let Some(ps) = state.players.get_mut(&player) {
                    ps.cards_drawn_this_turn += 1;
                }
                vec![GameEvent::CardDrawn {
                    player,
                    new_object_id: new_id,
                }]
            } else {
                vec![]
            }
        }
    }
}

/// Discard `n` cards from a player's hand (first by ObjectId, deterministic).
fn discard_cards(state: &mut GameState, player: PlayerId, n: usize, events: &mut Vec<GameEvent>) {
    let hand_id = ZoneId::Hand(player);
    for _ in 0..n {
        let card_id = state
            .objects
            .iter()
            .filter(|(_, obj)| obj.zone == hand_id)
            .map(|(&id, _)| id)
            .min_by_key(|id| id.0);

        if let Some(card_id) = card_id {
            if let Ok((new_id, _)) = state.move_object_to_zone(card_id, ZoneId::Graveyard(player)) {
                events.push(GameEvent::CardDiscarded {
                    player,
                    object_id: card_id,
                    new_id,
                });
            }
        }
    }
}

/// Mill `n` cards from the top of a player's library.
fn mill_cards(state: &mut GameState, player: PlayerId, n: usize, events: &mut Vec<GameEvent>) {
    let lib_id = ZoneId::Library(player);
    for _ in 0..n {
        let top = state.zones.get(&lib_id).and_then(|z| z.top());
        if let Some(card_id) = top {
            if let Ok((new_id, _)) = state.move_object_to_zone(card_id, ZoneId::Graveyard(player)) {
                events.push(GameEvent::CardMilled { player, new_id });
            }
        }
    }
}

// ── Filter matching ───────────────────────────────────────────────────────────

/// Check if an object's characteristics match a `TargetFilter`.
pub fn matches_filter(chars: &Characteristics, filter: &TargetFilter) -> bool {
    if let Some(max_p) = filter.max_power {
        if chars.power.map(|p| p > max_p).unwrap_or(true) {
            return false;
        }
    }
    if let Some(min_p) = filter.min_power {
        if chars.power.map(|p| p < min_p).unwrap_or(true) {
            return false;
        }
    }
    if let Some(ct) = &filter.has_card_type {
        if !chars.card_types.contains(ct) {
            return false;
        }
    }
    for kw in &filter.has_keywords {
        if !chars.keywords.contains(kw) {
            return false;
        }
    }
    if let Some(colors) = &filter.colors {
        if !chars.colors.iter().any(|c| colors.contains(c)) {
            return false;
        }
    }
    if let Some(excluded) = &filter.exclude_colors {
        if chars.colors.iter().any(|c| excluded.contains(c)) {
            return false;
        }
    }
    if filter.non_creature && chars.card_types.contains(&CardType::Creature) {
        return false;
    }
    if filter.non_land && chars.card_types.contains(&CardType::Land) {
        return false;
    }
    if filter.basic
        && !chars
            .supertypes
            .contains(&crate::state::types::SuperType::Basic)
    {
        return false;
    }
    if let Some(st) = &filter.has_subtype {
        if !chars.subtypes.contains(st) {
            return false;
        }
    }
    true
}

// ── Condition checking ────────────────────────────────────────────────────────

fn check_condition(state: &GameState, condition: &Condition, ctx: &EffectContext) -> bool {
    match condition {
        Condition::Always => true,
        Condition::ControllerLifeAtLeast(n) => state
            .players
            .get(&ctx.controller)
            .map(|ps| ps.life_total >= *n as i32)
            .unwrap_or(false),
        Condition::SourceOnBattlefield => state
            .objects
            .get(&ctx.source)
            .map(|obj| obj.zone == ZoneId::Battlefield)
            .unwrap_or(false),
        Condition::YouControlPermanent(filter) => state.objects.values().any(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.controller == ctx.controller
                && matches_filter(&obj.characteristics, filter)
        }),
        Condition::OpponentControlsPermanent(filter) => state.objects.values().any(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.controller != ctx.controller
                && matches_filter(&obj.characteristics, filter)
        }),
        Condition::TargetIsLegal { index } => {
            // The target is legal if its object still exists.
            match ctx.targets.get(*index) {
                Some(SpellTarget {
                    target: Target::Object(id),
                    zone_at_cast,
                }) => state
                    .objects
                    .get(id)
                    .map(|obj| Some(obj.zone) == *zone_at_cast)
                    .unwrap_or(false),
                Some(SpellTarget {
                    target: Target::Player(p),
                    ..
                }) => state.players.get(p).map(|ps| !ps.has_lost).unwrap_or(false),
                None => false,
            }
        }
        Condition::SourceHasCounters { counter, min } => state
            .objects
            .get(&ctx.source)
            .and_then(|obj| obj.counters.get(counter).copied())
            .map(|count| count >= *min)
            .unwrap_or(false),
    }
}

// ── ForEach collection ────────────────────────────────────────────────────────

fn collect_for_each(state: &GameState, over: &ForEachTarget, ctx: &EffectContext) -> Vec<ObjectId> {
    match over {
        ForEachTarget::EachCreature => state
            .objects
            .iter()
            .filter(|(_, obj)| {
                obj.zone == ZoneId::Battlefield
                    && obj.characteristics.card_types.contains(&CardType::Creature)
            })
            .map(|(&id, _)| id)
            .collect(),
        ForEachTarget::EachCreatureYouControl => state
            .objects
            .iter()
            .filter(|(_, obj)| {
                obj.zone == ZoneId::Battlefield
                    && obj.controller == ctx.controller
                    && obj.characteristics.card_types.contains(&CardType::Creature)
            })
            .map(|(&id, _)| id)
            .collect(),
        ForEachTarget::EachOpponentsCreature => state
            .objects
            .iter()
            .filter(|(_, obj)| {
                obj.zone == ZoneId::Battlefield
                    && obj.controller != ctx.controller
                    && obj.characteristics.card_types.contains(&CardType::Creature)
            })
            .map(|(&id, _)| id)
            .collect(),
        ForEachTarget::EachPermanentMatching(filter) => state
            .objects
            .iter()
            .filter(|(_, obj)| {
                obj.zone == ZoneId::Battlefield && matches_filter(&obj.characteristics, filter)
            })
            .map(|(&id, _)| id)
            .collect(),
        // Player-based ForEach targets return no objects (players aren't ObjectIds).
        ForEachTarget::EachOpponent | ForEachTarget::EachPlayer => vec![],

        // CR 614.1: All cards currently in any graveyard.
        ForEachTarget::EachCardInAllGraveyards => state
            .objects
            .iter()
            .filter(|(_, obj)| matches!(obj.zone, ZoneId::Graveyard(_)))
            .map(|(&id, _)| id)
            .collect(),
    }
}
