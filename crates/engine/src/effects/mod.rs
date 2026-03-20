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
    Condition, Effect, EffectAmount, EffectTarget, ForEachTarget, ManaRestriction, PlayerTarget,
    TargetController, TargetFilter, ZoneTarget,
};
use crate::rules::events::{CombatDamageTarget, GameEvent};
use crate::state::game_object::{
    Characteristics, Designations, GameObject, HybridMana, ObjectId, ObjectStatus, PhyrexianMana,
};
use crate::state::player::PlayerId;
use crate::state::stubs::{PendingTrigger, PendingTriggerKind};
use crate::state::targeting::{SpellTarget, Target};
use crate::state::turn::Phase;
use crate::state::types::{CardType, Color, KeywordAbility, ManaColor, SubType, SuperType};
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
    /// CR 702.33d: Number of times kicker was paid for this spell.
    ///
    /// 0 = not kicked. Used by `Condition::WasKicked`. Set from `StackObject.kicker_times_paid`
    /// at spell resolution, or from `GameObject.kicker_times_paid` for ETB triggers.
    pub kicker_times_paid: u32,
    /// CR 702.96a: If true, this spell was cast with its overload cost paid.
    /// Used by `Condition::WasOverloaded`. Set from `StackObject.was_overloaded`
    /// at spell resolution.
    pub was_overloaded: bool,
    /// CR 702.166b: If true, this spell was cast with its bargain cost paid.
    /// Used by `Condition::WasBargained`. Set from `StackObject.was_bargained`
    /// at spell resolution.
    pub was_bargained: bool,
    /// CR 702.148a: If true, this spell was cast by paying its cleave cost.
    /// Used by `Condition::WasCleaved`. Set from `StackObject.was_cleaved`
    /// at spell resolution.
    pub was_cleaved: bool,
    /// CR 701.59c: If true, this spell was cast with its collect evidence cost paid.
    /// Used by `Condition::EvidenceWasCollected`. Set from `StackObject.evidence_collected`
    /// at spell resolution.
    pub evidence_collected: bool,
    /// CR 107.3m: The value of X for this spell or ability.
    /// Set from StackObject.x_value at resolution so EffectAmount::XValue resolves correctly.
    /// 0 for non-X spells and for abilities that don't carry an X value.
    pub x_value: u32,
    /// CR 702.174b: If true, this spell was cast with its gift cost paid.
    /// Used by `Condition::GiftWasGiven`. Set from `StackObject.gift_was_given`
    /// at spell resolution.
    pub gift_was_given: bool,
    /// CR 702.174a: The opponent chosen to receive the gift.
    /// Set from `StackObject.gift_opponent` at spell resolution.
    pub gift_opponent: Option<crate::state::PlayerId>,
    /// Count of permanents actually destroyed or exiled by the most recent DestroyAll/ExileAll.
    /// Written by `Effect::DestroyAll` and `Effect::ExileAll`; read by `EffectAmount::LastEffectCount`.
    /// Used for follow-up effects like Fumigate ("gain 1 life for each creature destroyed this way").
    pub last_effect_count: u32,
}

impl EffectContext {
    /// Build a basic context from resolution data.
    pub fn new(controller: PlayerId, source: ObjectId, targets: Vec<SpellTarget>) -> Self {
        Self {
            controller,
            source,
            targets,
            target_remaps: HashMap::new(),
            kicker_times_paid: 0,
            was_overloaded: false,
            was_bargained: false,
            was_cleaved: false,
            evidence_collected: false,
            x_value: 0,
            gift_was_given: false,
            gift_opponent: None,
            last_effect_count: 0,
        }
    }

    /// Build a context with kicker status (CR 702.33d).
    pub fn new_with_kicker(
        controller: PlayerId,
        source: ObjectId,
        targets: Vec<SpellTarget>,
        kicker_times_paid: u32,
    ) -> Self {
        Self {
            controller,
            source,
            targets,
            target_remaps: HashMap::new(),
            kicker_times_paid,
            was_overloaded: false,
            was_bargained: false,
            was_cleaved: false,
            evidence_collected: false,
            x_value: 0,
            gift_was_given: false,
            gift_opponent: None,
            last_effect_count: 0,
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
            let raw_dmg = resolve_amount(state, amount, ctx).max(0) as u32;
            if raw_dmg == 0 {
                return;
            }
            let targets = resolve_effect_target_list(state, target, ctx);
            for resolved in targets {
                match resolved {
                    ResolvedTarget::Player(p) => {
                        // CR 615: check prevention before applying damage.
                        let damage_target = CombatDamageTarget::Player(p);
                        // CR 614.1: Apply damage-doubling replacement effects before prevention.
                        // Doubling is per-target so target-side filters (e.g., Twinflame Tyrant)
                        // can check whether the target is an opponent or their permanent.
                        let (dmg, doubling_events) =
                            crate::rules::replacement::apply_damage_doubling(
                                state,
                                ctx.source,
                                raw_dmg,
                                Some(&damage_target),
                            );
                        events.extend(doubling_events);
                        if dmg == 0 {
                            continue;
                        }
                        let (final_dmg, prev_events) =
                            crate::rules::replacement::apply_damage_prevention(
                                state,
                                ctx.source,
                                &damage_target,
                                dmg,
                            );
                        events.extend(prev_events);
                        if final_dmg > 0 {
                            // CR 702.90b / CR 120.3b: check source for infect.
                            let source_has_infect =
                                crate::rules::layers::calculate_characteristics(state, ctx.source)
                                    .map(|c| c.keywords.contains(&KeywordAbility::Infect))
                                    .unwrap_or(false);

                            if source_has_infect {
                                // CR 120.3b: infect damage to a player gives poison counters
                                // instead of causing life loss (CR 702.90b).
                                if let Some(player) = state.players.get_mut(&p) {
                                    player.poison_counters += final_dmg;
                                    // CR 702.54a: Bloodthirst counts infect damage even though
                                    // it causes poison counters rather than life loss.
                                    player.damage_received_this_turn += final_dmg;
                                }
                                events.push(GameEvent::DamageDealt {
                                    source: ctx.source,
                                    target: damage_target,
                                    amount: final_dmg,
                                });
                                events.push(GameEvent::PoisonCountersGiven {
                                    player: p,
                                    amount: final_dmg,
                                    source: ctx.source,
                                });
                            } else {
                                // CR 120.3a: normal damage causes life loss.
                                if let Some(player) = state.players.get_mut(&p) {
                                    player.life_total -= final_dmg as i32;
                                    // CR 702.137a: track life lost this turn for Spectacle.
                                    player.life_lost_this_turn += final_dmg;
                                    // CR 702.54a: track damage received this turn for Bloodthirst.
                                    player.damage_received_this_turn += final_dmg;
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

                        // CR 614.1: Apply damage-doubling replacement effects before prevention.
                        let (dmg, doubling_events) =
                            crate::rules::replacement::apply_damage_doubling(
                                state,
                                ctx.source,
                                raw_dmg,
                                Some(&damage_target),
                            );
                        events.extend(doubling_events);
                        if dmg == 0 {
                            continue;
                        }

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
                            } else if card_types.contains(&CardType::Creature) {
                                // CR 120.3d/e: check source for wither and/or infect keyword.
                                // CR 702.80a: wither applies to damage dealt to creatures.
                                // CR 702.90c: infect also places -1/-1 counters on creatures.
                                // CR 702.80c / CR 702.90e: both function from any zone.
                                let source_chars = crate::rules::layers::calculate_characteristics(
                                    state, ctx.source,
                                );
                                let source_has_wither = source_chars
                                    .as_ref()
                                    .map(|c| c.keywords.contains(&KeywordAbility::Wither))
                                    .unwrap_or(false);
                                let source_has_infect = source_chars
                                    .as_ref()
                                    .map(|c| c.keywords.contains(&KeywordAbility::Infect))
                                    .unwrap_or(false);

                                if let Some(obj) = state.objects.get_mut(&id) {
                                    if source_has_wither || source_has_infect {
                                        // CR 702.80a / CR 702.90c / CR 120.3d: wither and/or
                                        // infect damage to a creature places -1/-1 counters
                                        // instead of marking damage. Multiple instances are
                                        // redundant (CR 702.80d / CR 702.90f).
                                        let cur = obj
                                            .counters
                                            .get(
                                                &crate::state::types::CounterType::MinusOneMinusOne,
                                            )
                                            .copied()
                                            .unwrap_or(0);
                                        obj.counters.insert(
                                            crate::state::types::CounterType::MinusOneMinusOne,
                                            cur + final_dmg,
                                        );
                                        events.push(GameEvent::CounterAdded {
                                            object_id: id,
                                            counter:
                                                crate::state::types::CounterType::MinusOneMinusOne,
                                            count: final_dmg,
                                        });
                                    } else {
                                        // CR 120.3e: normal damage marking.
                                        obj.damage_marked += final_dmg;
                                    }
                                }
                            } else {
                                // CR 120.3e: non-creature, non-planeswalker permanents (e.g.,
                                // battles, CR 120.3h) mark damage normally. Wither does NOT
                                // apply (CR 702.80a — only creatures receive -1/-1 counters).
                                if let Some(obj) = state.objects.get_mut(&id) {
                                    obj.damage_marked += final_dmg;
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
                // CR 614.1: Apply life-loss doubling replacement effects before subtracting.
                let (final_loss, doubling_events) =
                    crate::rules::replacement::apply_life_loss_doubling(state, p, loss);
                events.extend(doubling_events);
                if let Some(ps) = state.players.get_mut(&p) {
                    ps.life_total -= final_loss as i32;
                    // CR 702.137a: track life lost this turn for Spectacle.
                    ps.life_lost_this_turn += final_loss;
                }
                events.push(GameEvent::LifeLost {
                    player: p,
                    amount: final_loss,
                });
            }
        }

        // CR 702.101a: Extort drain — each opponent loses `amount` life and the
        // controller gains life equal to the total life actually lost.
        //
        // Gain is based on actual life-total delta (not opponent count * amount),
        // so if an opponent's life total cannot change, the gain is reduced
        // accordingly (per ruling 2024-01-12: Platinum Emperion case).
        Effect::DrainLife { amount } => {
            // Clamp negative amounts to 0.
            let loss = resolve_amount(state, amount, ctx).max(0) as u32;
            if loss == 0 {
                return;
            }
            // Collect opponents of the controller (non-eliminated, non-controller players).
            let opponents = resolve_player_target_list(state, &PlayerTarget::EachOpponent, ctx);
            let mut total_lost: u32 = 0;
            for &p in &opponents {
                // CR 614.1: Apply life-loss doubling replacement effects before subtracting.
                let (final_loss, doubling_events) =
                    crate::rules::replacement::apply_life_loss_doubling(state, p, loss);
                events.extend(doubling_events);
                if let Some(ps) = state.players.get_mut(&p) {
                    let before = ps.life_total;
                    ps.life_total -= final_loss as i32;
                    // Actual loss = pre-loss total minus post-loss total, clamped to >=0.
                    let actual = (before - ps.life_total).max(0) as u32;
                    total_lost += actual;
                    // CR 702.137a: track life lost this turn for Spectacle.
                    ps.life_lost_this_turn += actual;
                }
                events.push(GameEvent::LifeLost {
                    player: p,
                    amount: final_loss,
                });
            }
            // Controller gains life equal to total actually lost by all opponents.
            if total_lost > 0 {
                if let Some(ps) = state.players.get_mut(&ctx.controller) {
                    ps.life_total += total_lost as i32;
                }
                events.push(GameEvent::LifeGained {
                    player: ctx.controller,
                    amount: total_lost,
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
            // CR 111.1 / CR 614.1: Apply token-creation replacement effects.
            let (token_count, repl_events) =
                crate::rules::replacement::apply_token_creation_replacement(
                    state,
                    ctx.controller,
                    spec.count,
                );
            events.extend(repl_events);
            for _ in 0..token_count {
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

        // CR 702.92a: Create a token and immediately attach the source Equipment to it.
        //
        // Used by Living Weapon. The create + attach happen atomically: SBAs do not fire
        // between creation and attachment (ruling: "The Germ token enters the battlefield
        // as a 0/0 creature and the Equipment becomes attached to it before state-based
        // actions would cause the token to die.").
        //
        // If multiple tokens are created (e.g., Doubling Season), the Equipment attaches
        // to the first one. Others are subject to SBAs normally (ruling 2020-08-07).
        Effect::CreateTokenAndAttachSource { spec } => {
            let mut first_token_id: Option<ObjectId> = None;
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
                    if first_token_id.is_none() {
                        first_token_id = Some(id);
                    }
                }
            }
            // Attach source Equipment to the first created token (CR 702.92a).
            // If Doubling Season creates extras, only the first gets equipped (ruling).
            if let Some(token_id) = first_token_id {
                let equip_id = ctx.source;
                // Verify source is still on the battlefield and is an Equipment.
                // CR 702.26b: phased-out permanents are treated as nonexistent.
                let source_on_bf = state
                    .objects
                    .get(&equip_id)
                    .map(|o| o.zone == ZoneId::Battlefield && o.is_phased_in())
                    .unwrap_or(false);
                if source_on_bf {
                    // Detach from previous (should not be attached, but defensive).
                    let prev_target_opt = state.objects.get(&equip_id).and_then(|o| o.attached_to);
                    if let Some(prev_target) = prev_target_opt {
                        if let Some(prev) = state.objects.get_mut(&prev_target) {
                            prev.attachments.retain(|&x| x != equip_id);
                        }
                    }
                    // Attach to token (CR 702.92a).
                    // CR 701.3c / CR 613.7e: new timestamp on attach.
                    state.timestamp_counter += 1;
                    let new_ts = state.timestamp_counter;
                    if let Some(equip_obj) = state.objects.get_mut(&equip_id) {
                        equip_obj.attached_to = Some(token_id);
                        equip_obj.timestamp = new_ts;
                    }
                    if let Some(target_obj) = state.objects.get_mut(&token_id) {
                        if !target_obj.attachments.contains(&equip_id) {
                            target_obj.attachments.push_back(equip_id);
                        }
                    }
                    events.push(GameEvent::EquipmentAttached {
                        equipment_id: equip_id,
                        target_id: token_id,
                        controller: ctx.controller,
                    });
                }
            }
        }

        // CR 701.16a: Investigate — create N Clue tokens sequentially.
        //
        // Ruling 2024-06-07: "If you're instructed to investigate multiple times,
        // those actions are sequential, meaning you'll create that many Clue tokens
        // one at a time." Each token creation is a separate event that "whenever you
        // create a token" triggers can respond to. Does nothing when count is 0.
        Effect::Investigate { count } => {
            let n = resolve_amount(state, count, ctx).max(0) as u32;
            if n > 0 {
                let spec = crate::cards::card_definition::clue_token_spec(1);
                // Create tokens one at a time (ruling 2024-06-07).
                for _ in 0..n {
                    let obj = make_token(&spec, ctx.controller);
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
                // CR 701.16a: Emit Investigated event for "whenever you investigate"
                // triggers (future cards like Lonis, Cryptozoologist).
                events.push(GameEvent::Investigated {
                    player: ctx.controller,
                    count: n,
                });
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

                    // CR 701.19a/614.8: Check regeneration shields before destruction.
                    // Self-replacement effects apply first (CR 614.15).
                    if let Some(shield_id) =
                        crate::rules::replacement::check_regeneration_shield(state, id)
                    {
                        let regen_events =
                            crate::rules::replacement::apply_regeneration(state, id, shield_id);
                        events.extend(regen_events);
                        continue; // Skip destruction -- permanent stays on battlefield
                    }

                    // CR 702.89a: Check umbra armor -- Aura saves the enchanted permanent.
                    // Unlike regeneration, the permanent is NOT tapped and NOT removed from combat.
                    // "Can't be regenerated" does NOT block umbra armor (separate mechanics -- ruling).
                    // TODO (CR 616.1): when both regen and umbra armor apply simultaneously, the
                    // controller should choose which applies first. Currently regen is checked first.
                    {
                        let auras = crate::rules::replacement::check_umbra_armor(state, id);
                        if !auras.is_empty() {
                            // Auto-select the first Aura. Multiple-Aura case: controller should
                            // choose (CR 616.1). TODO: NeedsChoice path for multiple umbra armors.
                            let aura_id = auras[0];
                            let umbra_events =
                                crate::rules::replacement::apply_umbra_armor(state, id, aura_id);
                            events.extend(umbra_events);
                            continue; // Skip destruction -- permanent stays on battlefield
                        }
                    }

                    let (card_types, owner, pre_death_controller, pre_death_counters) = state
                        .objects
                        .get(&id)
                        .map(|o| {
                            (
                                o.characteristics.card_types.clone(),
                                o.owner,
                                // CR 603.3a: capture controller before move_object_to_zone resets it.
                                o.controller,
                                // CR 702.79a: capture counters before move_object_to_zone resets them.
                                o.counters.clone(),
                            )
                        })
                        .unwrap_or_else(|| {
                            (
                                Default::default(),
                                ctx.controller,
                                ctx.controller,
                                Default::default(),
                            )
                        });

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
                                                controller: pre_death_controller,
                                                // CR 702.79a: last-known counter state.
                                                pre_death_counters: pre_death_counters.clone(),
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
                                        controller: pre_death_controller,
                                        // CR 702.79a: last-known counter state.
                                        pre_death_counters: pre_death_counters.clone(),
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

        // CR 701.8: Destroy all permanents matching the filter.
        // CR 702.12b: Indestructible permanents are skipped.
        // CR 701.19c: cant_be_regenerated=true bypasses regeneration shields.
        Effect::DestroyAll {
            filter,
            cant_be_regenerated,
        } => {
            // Snapshot the list of matching objects BEFORE any destructions
            // (CR 701.8: all checks happen against the pre-resolution game state).
            let ids_to_destroy: Vec<ObjectId> = state
                .objects
                .iter()
                .filter(|(_, obj)| {
                    obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && matches_filter(&obj.characteristics, filter)
                        && match filter.controller {
                            TargetController::Any => true,
                            TargetController::You => obj.controller == ctx.controller,
                            TargetController::Opponent => obj.controller != ctx.controller,
                        }
                })
                .map(|(&id, _)| id)
                .collect();

            let mut destroyed_count: u32 = 0;

            for id in ids_to_destroy {
                // CR 702.12b: Indestructible permanents can't be destroyed.
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

                // CR 701.19c: If cant_be_regenerated, skip regeneration shields.
                if !cant_be_regenerated {
                    if let Some(shield_id) =
                        crate::rules::replacement::check_regeneration_shield(state, id)
                    {
                        let regen_events =
                            crate::rules::replacement::apply_regeneration(state, id, shield_id);
                        events.extend(regen_events);
                        continue;
                    }
                }

                // CR 702.89a: Check umbra armor — Aura saves the enchanted permanent.
                {
                    let auras = crate::rules::replacement::check_umbra_armor(state, id);
                    if !auras.is_empty() {
                        let aura_id = auras[0];
                        let umbra_events =
                            crate::rules::replacement::apply_umbra_armor(state, id, aura_id);
                        events.extend(umbra_events);
                        continue;
                    }
                }

                let (card_types, owner, pre_death_controller, pre_death_counters) = state
                    .objects
                    .get(&id)
                    .map(|o| {
                        (
                            o.characteristics.card_types.clone(),
                            o.owner,
                            o.controller,
                            o.counters.clone(),
                        )
                    })
                    .unwrap_or_else(|| {
                        (
                            Default::default(),
                            ctx.controller,
                            ctx.controller,
                            Default::default(),
                        )
                    });

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
                                    destroyed_count += 1;
                                }
                                ZoneId::Command(_) => {
                                    // CR 903.9a: Commander goes to graveyard first, then
                                    // an SBA moves it to the command zone. The destruction
                                    // DID occur — count it so effects like Fumigate gain
                                    // life even for destroyed commanders.
                                    destroyed_count += 1;
                                }
                                _ => {
                                    if card_types.contains(&CardType::Creature) {
                                        events.push(GameEvent::CreatureDied {
                                            object_id: id,
                                            new_grave_id: new_id,
                                            controller: pre_death_controller,
                                            pre_death_counters: pre_death_counters.clone(),
                                        });
                                    } else {
                                        events.push(GameEvent::PermanentDestroyed {
                                            object_id: id,
                                            new_grave_id: new_id,
                                        });
                                    }
                                    destroyed_count += 1;
                                }
                            }
                        }
                    }
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
                        if let Ok((new_id, _old)) =
                            state.move_object_to_zone(id, ZoneId::Graveyard(owner))
                        {
                            if card_types.contains(&CardType::Creature) {
                                events.push(GameEvent::CreatureDied {
                                    object_id: id,
                                    new_grave_id: new_id,
                                    controller: pre_death_controller,
                                    pre_death_counters: pre_death_counters.clone(),
                                });
                            } else {
                                events.push(GameEvent::PermanentDestroyed {
                                    object_id: id,
                                    new_grave_id: new_id,
                                });
                            }
                            destroyed_count += 1;
                        }
                    }
                }
            }

            ctx.last_effect_count = destroyed_count;
        }

        // CR 406.2: Exile all permanents matching the filter.
        // Stores count in ctx.last_effect_count for follow-up effects.
        Effect::ExileAll { filter } => {
            // Snapshot matching objects before any exiles.
            let ids_to_exile: Vec<ObjectId> = state
                .objects
                .iter()
                .filter(|(_, obj)| {
                    obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && matches_filter(&obj.characteristics, filter)
                        && match filter.controller {
                            TargetController::Any => true,
                            TargetController::You => obj.controller == ctx.controller,
                            TargetController::Opponent => obj.controller != ctx.controller,
                        }
                })
                .map(|(&id, _)| id)
                .collect();

            let mut exiled_count: u32 = 0;

            for id in ids_to_exile {
                let owner = state
                    .objects
                    .get(&id)
                    .map(|o| o.owner)
                    .unwrap_or(ctx.controller);

                // CR 614: Check replacement effects before exiling.
                let action = crate::rules::replacement::check_zone_change_replacement(
                    state,
                    id,
                    ZoneType::Battlefield,
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
                            match dest {
                                ZoneId::Command(_) => {
                                    // Commander redirected — no exile event.
                                }
                                _ => {
                                    events.push(GameEvent::ObjectExiled {
                                        player: owner,
                                        object_id: id,
                                        new_exile_id: new_id,
                                    });
                                    exiled_count += 1;
                                }
                            }
                        }
                    }
                    crate::rules::replacement::ZoneChangeAction::ChoiceRequired {
                        player,
                        choices,
                        event_description,
                    } => {
                        use crate::state::replacement_effect::PendingZoneChange;
                        state.pending_zone_changes.push_back(PendingZoneChange {
                            object_id: id,
                            original_from: ZoneType::Battlefield,
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
                        if let Ok((new_id, _old)) = state.move_object_to_zone(id, ZoneId::Exile) {
                            events.push(GameEvent::ObjectExiled {
                                player: owner,
                                object_id: id,
                                new_exile_id: new_id,
                            });
                            exiled_count += 1;
                        }
                    }
                }
            }

            ctx.last_effect_count = exiled_count;
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
                                // CR 702.34a: If cast with flashback, exile instead of
                                // graveyard when countered by an effect.
                                // CR 702.133a: Jump-start also exiles when countered by an effect.
                                let destination = if stack_obj.cast_with_flashback
                                    || stack_obj.cast_with_jump_start
                                {
                                    crate::state::zone::ZoneId::Exile
                                } else {
                                    ZoneId::Graveyard(owner)
                                };
                                if let Ok((new_id, _)) =
                                    state.move_object_to_zone(source_object, destination)
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

        Effect::AddManaScaled {
            player,
            color,
            count,
        } => {
            let amount = resolve_amount(state, count, ctx).max(0) as u32;
            let players = resolve_player_target_list(state, player, ctx);
            for p in players {
                if let Some(ps) = state.players.get_mut(&p) {
                    ps.mana_pool.add(*color, amount);
                    events.push(GameEvent::ManaAdded {
                        player: p,
                        color: *color,
                        amount,
                    });
                }
            }
        }

        // CR 106.12: Add mana with a spending restriction.
        Effect::AddManaRestricted {
            player,
            mana,
            restriction,
        } => {
            let players = resolve_player_target_list(state, player, ctx);
            let resolved = resolve_mana_restriction(state, &Some(restriction.clone()), ctx);
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
                        add_mana_with_restriction(ps, color, amount, &resolved);
                        events.push(GameEvent::ManaAdded {
                            player: p,
                            color,
                            amount,
                        });
                    }
                }
            }
        }

        // CR 106.12: Add one mana of any color with a spending restriction.
        Effect::AddManaAnyColorRestricted {
            player,
            restriction,
        } => {
            let players = resolve_player_target_list(state, player, ctx);
            let resolved = resolve_mana_restriction(state, &Some(restriction.clone()), ctx);
            for p in players {
                if let Some(ps) = state.players.get_mut(&p) {
                    add_mana_with_restriction(ps, ManaColor::Colorless, 1, &resolved);
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
                    // CR 122.6 / CR 614.1: Apply counter-placement replacement effects
                    // before actually placing counters. The placer is the controller of
                    // the effect that is placing the counters.
                    let (modified_count, repl_events) =
                        crate::rules::replacement::apply_counter_replacement(
                            state,
                            ctx.controller,
                            id,
                            &counter,
                            count,
                        );
                    events.extend(repl_events);
                    if modified_count > 0 {
                        if let Some(obj) = state.objects.get_mut(&id) {
                            let cur = obj.counters.get(&counter).copied().unwrap_or(0);
                            obj.counters.insert(counter.clone(), cur + modified_count);
                            events.push(GameEvent::CounterAdded {
                                object_id: id,
                                counter: counter.clone(),
                                count: modified_count,
                            });
                        }
                    }
                }
            }
        }

        // CR 122: Add N counters to a target where N is an EffectAmount (e.g. LastEffectCount).
        Effect::AddCounterAmount {
            target,
            counter,
            count,
        } => {
            let counter = counter.clone();
            let count = resolve_amount(state, count, ctx).max(0) as u32;
            if count > 0 {
                let targets = resolve_effect_target_list(state, target, ctx);
                for resolved in targets {
                    if let ResolvedTarget::Object(id) = resolved {
                        let (modified_count, repl_events) =
                            crate::rules::replacement::apply_counter_replacement(
                                state,
                                ctx.controller,
                                id,
                                &counter,
                                count,
                            );
                        events.extend(repl_events);
                        if modified_count > 0 {
                            if let Some(obj) = state.objects.get_mut(&id) {
                                let cur = obj.counters.get(&counter).copied().unwrap_or(0);
                                obj.counters.insert(counter.clone(), cur + modified_count);
                                events.push(GameEvent::CounterAdded {
                                    object_id: id,
                                    counter: counter.clone(),
                                    count: modified_count,
                                });
                            }
                        }
                    }
                }
            }
        }

        // CR 500.8: Additional combat phase (CR 500.10a: only applies on active player's turn).
        Effect::AdditionalCombatPhase { followed_by_main } => {
            if state.turn.active_player == ctx.controller {
                if *followed_by_main {
                    // Push main phase first (consumed second due to LIFO pop_back).
                    // CR 505.1a: all additional main phases are postcombat main phases.
                    state
                        .turn
                        .additional_phases
                        .push_back(Phase::PostCombatMain);
                }
                // Push combat phase (consumed first due to LIFO).
                state.turn.additional_phases.push_back(Phase::Combat);
                events.push(
                    crate::rules::events::GameEvent::AdditionalCombatPhaseCreated {
                        controller: ctx.controller,
                        followed_by_main: *followed_by_main,
                    },
                );
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

        // CR 701.39a: Bolster N -- choose a creature controlled by the given player
        // with the least toughness (layer-aware), put N +1/+1 counters on it.
        // Bolster does NOT target; the creature is chosen at resolution time.
        // Ruling 2014-11-24: toughness is determined as the ability resolves.
        Effect::Bolster { player, count } => {
            let n = resolve_amount(state, count, ctx).max(0) as u32;
            if n == 0 {
                // Bolster 0 does nothing.
                return;
            }
            let players = resolve_player_target_list(state, player, ctx);
            for p in players {
                // CR 701.39a: Find all creatures controlled by this player on the
                // battlefield, then select the one with the least toughness.
                // Use calculate_characteristics for layer-aware toughness (ruling 2014-11-24).
                // CR 702.26b: phased-out permanents are treated as nonexistent.
                let creatures: Vec<(ObjectId, i32)> = state
                    .objects
                    .iter()
                    .filter(|(_, obj)| {
                        obj.zone == ZoneId::Battlefield && obj.is_phased_in() && obj.controller == p
                    })
                    .filter_map(|(&id, _)| {
                        let chars = crate::rules::layers::calculate_characteristics(state, id)?;
                        // Use layer-aware card_types to support animated non-creatures.
                        if !chars
                            .card_types
                            .contains(&crate::state::types::CardType::Creature)
                        {
                            return None;
                        }
                        chars.toughness.map(|t| (id, t))
                    })
                    .collect();

                // Find the minimum toughness value; if no creatures exist, bolster does nothing.
                let Some(min_toughness) = creatures.iter().map(|(_, t)| *t).min() else {
                    continue;
                };

                // Among tied creatures, choose the one with the smallest ObjectId
                // (deterministic fallback -- interactive choice deferred to M10+).
                let Some(chosen_id) = creatures
                    .iter()
                    .filter(|(_, t)| *t == min_toughness)
                    .map(|(id, _)| *id)
                    .min_by_key(|id| id.0)
                else {
                    continue;
                };

                // Place N +1/+1 counters on the chosen creature.
                if let Some(obj) = state.objects.get_mut(&chosen_id) {
                    let cur = obj
                        .counters
                        .get(&crate::state::types::CounterType::PlusOnePlusOne)
                        .copied()
                        .unwrap_or(0);
                    obj.counters
                        .insert(crate::state::types::CounterType::PlusOnePlusOne, cur + n);
                    events.push(GameEvent::CounterAdded {
                        object_id: chosen_id,
                        counter: crate::state::types::CounterType::PlusOnePlusOne,
                        count: n,
                    });
                }
            }
        }

        // ── Amass ─────────────────────────────────────────────────────────
        // CR 701.47a: Amass [subtype] N.
        //   1. If you don't control an Army creature, create a 0/0 black
        //      [subtype] Army creature token.
        //   2. Choose an Army creature you control (deterministic: smallest
        //      ObjectId among all Army creatures, consistent with Bolster).
        //   3. Put N +1/+1 counters on that creature.
        //   4. If the chosen Army isn't a [subtype], it becomes one in
        //      addition to its other types.
        // CR 701.47b: Always emits Amassed even if some/all actions failed.
        Effect::Amass { subtype, count } => {
            let n = resolve_amount(state, count, ctx).max(0) as u32;
            let controller = ctx.controller;

            // Step 1–2: Find existing Army creatures controlled by `controller`.
            // Uses calculate_characteristics for layer-aware type check (Changeling).
            // CR 702.26b: phased-out permanents are treated as nonexistent.
            let mut army_ids: Vec<ObjectId> = state
                .objects
                .iter()
                .filter(|(_, obj)| {
                    obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && obj.controller == controller
                })
                .filter_map(|(&id, _)| {
                    let chars = crate::rules::layers::calculate_characteristics(state, id)?;
                    // Must be a Creature with subtype "Army".
                    if !chars
                        .card_types
                        .contains(&crate::state::types::CardType::Creature)
                    {
                        return None;
                    }
                    if chars
                        .subtypes
                        .contains(&crate::state::types::SubType("Army".to_string()))
                    {
                        Some(id)
                    } else {
                        None
                    }
                })
                .collect();

            // If no Army exists, create a 0/0 black [subtype] Army token (CR 701.47a).
            // Ruling 2023-06-16: the token enters as 0/0 BEFORE receiving counters.
            // SBAs are not checked between token creation and counter placement.
            let army_id = if army_ids.is_empty() {
                let spec = crate::cards::card_definition::army_token_spec(subtype);
                let token = make_token(&spec, controller);
                if let Ok(id) = state.add_object(token, ZoneId::Battlefield) {
                    events.push(GameEvent::TokenCreated {
                        player: controller,
                        object_id: id,
                    });
                    events.push(GameEvent::PermanentEnteredBattlefield {
                        player: controller,
                        object_id: id,
                    });
                    army_ids.push(id);
                    id
                } else {
                    // Token creation failed (should not happen in normal play).
                    // CR 701.47b: still emit Amassed even if actions were impossible.
                    events.push(GameEvent::Amassed {
                        player: controller,
                        army_id: ObjectId(0),
                        count: n,
                    });
                    return;
                }
            } else {
                // Deterministic: choose the Army with the smallest ObjectId.
                // Ruling 2023-06-16: "you choose which Army creature to put the
                // counters on" — deterministic fallback defers interactive choice.
                let Some(&chosen) = army_ids.iter().min_by_key(|id| id.0) else {
                    return;
                };
                chosen
            };

            // Step 3: Place N +1/+1 counters on the chosen Army (if N > 0).
            if n > 0 {
                if let Some(obj) = state.objects.get_mut(&army_id) {
                    let cur = obj
                        .counters
                        .get(&crate::state::types::CounterType::PlusOnePlusOne)
                        .copied()
                        .unwrap_or(0);
                    obj.counters
                        .insert(crate::state::types::CounterType::PlusOnePlusOne, cur + n);
                    events.push(GameEvent::CounterAdded {
                        object_id: army_id,
                        counter: crate::state::types::CounterType::PlusOnePlusOne,
                        count: n,
                    });
                }
            }

            // Step 4: If the chosen Army isn't a [subtype], add the subtype (CR 701.47a).
            // This is a one-shot modification (not a continuous effect) — the subtype
            // is permanently added to the creature's characteristics.
            let army_subtype = crate::state::types::SubType(subtype.clone());
            if let Some(obj) = state.objects.get_mut(&army_id) {
                if !obj.characteristics.subtypes.contains(&army_subtype) {
                    obj.characteristics.subtypes.insert(army_subtype);
                }
            }

            // CR 701.47b: Always emit Amassed, even if counters or subtype change
            // were impossible (e.g., N=0 still creates the token and emits the event).
            events.push(GameEvent::Amassed {
                player: controller,
                army_id,
                count: n,
            });
        }

        // ── Zone ──────────────────────────────────────────────────────────
        Effect::MoveZone {
            target,
            to,
            controller_override,
        } => {
            // MR-M7-04: resolve zone using owner PlayerTarget (not always controller).
            // MR-M7-01: emit destination-correct event instead of always ObjectExiled.
            let targets = resolve_effect_target_list_indexed(state, target, ctx);
            for (idx_opt, resolved) in targets {
                if let ResolvedTarget::Object(id) = resolved {
                    let dest = resolve_zone_target(to, state, ctx);
                    if let Ok((new_id, _)) = state.move_object_to_zone(id, dest) {
                        // Apply controller override for "under your control" effects (e.g. Reanimate).
                        // move_object_to_zone always resets controller to owner; override after.
                        if let Some(override_player_target) = controller_override {
                            let override_players =
                                resolve_player_target_list(state, override_player_target, ctx);
                            if let (Some(new_obj), Some(&new_controller)) =
                                (state.objects.get_mut(&new_id), override_players.first())
                            {
                                new_obj.controller = new_controller;
                            }
                        }
                        if let Some(idx) = idx_opt {
                            ctx.target_remaps.insert(idx, new_id);
                        }
                        // CR 702.79a / CR 702.93a: If the moved object was the ability source
                        // (e.g. persist/undying moving from graveyard to battlefield), update
                        // ctx.source so subsequent effects (AddCounter) can find the new object.
                        if id == ctx.source {
                            ctx.source = new_id;
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
            shuffle_before_placing,
        } => {
            // M9+: interactive card search. For M7, deterministic fallback:
            // find the first matching card (by ObjectId, ascending) in the library.
            let players = resolve_player_target_list(state, player, ctx);
            for p in players {
                // CR 701.23 / CR 614.1: Check search restriction replacements.
                let (search_restriction, repl_events) =
                    crate::rules::replacement::apply_search_library_replacement(state, p);
                events.extend(repl_events);

                let lib_id = ZoneId::Library(p);
                let mut candidates: Vec<ObjectId> = state
                    .objects
                    .iter()
                    .filter(|(_, obj)| {
                        obj.zone == lib_id && matches_filter(&obj.characteristics, filter)
                    })
                    .map(|(id, _)| *id)
                    .collect();

                // If search is restricted to top N, sort by library position
                // and truncate. Library order is by ObjectId ascending (deterministic).
                if let Some(top_n) = search_restriction {
                    let mut all_lib: Vec<ObjectId> = state
                        .objects
                        .iter()
                        .filter(|(_, obj)| obj.zone == lib_id)
                        .map(|(id, _)| *id)
                        .collect();
                    all_lib.sort();
                    let top_ids: std::collections::HashSet<ObjectId> =
                        all_lib.into_iter().take(top_n as usize).collect();
                    candidates.retain(|id| top_ids.contains(id));
                }

                if let Some(&card_id) = candidates.iter().min_by_key(|&&id| id.0) {
                    // CR 701.23: "shuffle and put on top" pattern — shuffle FIRST, then place.
                    // Vampiric Tutor/Worldly Tutor ruling (2016-06-08): "The 'shuffle and put
                    // the card on top' is a single action." We implement this by shuffling while
                    // the card is still in the library, then moving it to the destination.
                    if *shuffle_before_placing {
                        let seed = state.timestamp_counter;
                        state.timestamp_counter += 1;
                        if let Some(zone) = state.zones.get_mut(&ZoneId::Library(p)) {
                            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
                            zone.shuffle(&mut rng);
                        }
                    }
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

        // CR 701.25: Surveil N -- deterministic fallback: put top N cards into graveyard
        // (interactive selection deferred to M10+).
        Effect::Surveil { player, count } => {
            let n = resolve_amount(state, count, ctx).max(0) as usize;
            let players = resolve_player_target_list(state, player, ctx);
            for p in players {
                // CR 701.25c: surveil 0 produces no event
                if n == 0 {
                    continue;
                }
                let lib_zone = ZoneId::Library(p);
                let graveyard_zone = ZoneId::Graveyard(p);
                // Collect the top N cards of the library (ordered from top).
                let top_ids: Vec<ObjectId> = state
                    .zones
                    .get(&lib_zone)
                    .map(|z| z.object_ids())
                    .unwrap_or_default()
                    .into_iter()
                    .take(n)
                    .collect();
                // Deterministic fallback: move all looked-at cards to graveyard.
                // Sort by ObjectId ascending for determinism.
                let mut to_graveyard = top_ids.clone();
                to_graveyard.sort_by_key(|id| id.0);
                let actual_count = to_graveyard.len();
                for id in to_graveyard {
                    let _ = state.move_object_to_zone(id, graveyard_zone);
                }
                // CR 701.25d: event fires even if some actions were impossible
                // (e.g., library had fewer than N cards).
                events.push(GameEvent::Surveilled {
                    player: p,
                    count: actual_count as u32,
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
                            kicker_times_paid: ctx.kicker_times_paid,
                            was_overloaded: ctx.was_overloaded,
                            was_bargained: ctx.was_bargained,
                            was_cleaved: ctx.was_cleaved,
                            evidence_collected: ctx.evidence_collected,
                            x_value: ctx.x_value,
                            gift_was_given: ctx.gift_was_given,
                            gift_opponent: ctx.gift_opponent,
                            last_effect_count: ctx.last_effect_count,
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
                            kicker_times_paid: ctx.kicker_times_paid,
                            was_overloaded: ctx.was_overloaded,
                            was_bargained: ctx.was_bargained,
                            was_cleaved: ctx.was_cleaved,
                            evidence_collected: ctx.evidence_collected,
                            x_value: ctx.x_value,
                            gift_was_given: ctx.gift_was_given,
                            gift_opponent: ctx.gift_opponent,
                            last_effect_count: ctx.last_effect_count,
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

        // CR 701.17a: Sacrifice permanents — the specified player sacrifices N permanents.
        //
        // Sacrifice is NOT destruction: it bypasses indestructible (CR 701.17a).
        // The player chooses which permanents to sacrifice; deterministic fallback
        // (M10+ will support interactive choice): sacrifice in ObjectId ascending order.
        // If the player controls fewer than N permanents, they sacrifice all they control.
        // Sacrifice is a zone change to the owner's graveyard; "dies" triggers fire normally.
        Effect::SacrificePermanents { player, count } => {
            let player_ids = resolve_player_target_list(state, player, ctx);
            let n = resolve_amount(state, count, ctx).max(0) as usize;
            for pid in player_ids {
                // Collect the player's battlefield permanents sorted by ObjectId ascending
                // for deterministic ordering (interactive choice deferred to M10+).
                // CR 702.26b: phased-out permanents are treated as nonexistent.
                let mut controlled: Vec<ObjectId> = state
                    .objects
                    .iter()
                    .filter(|(_, obj)| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj.controller == pid
                    })
                    .map(|(id, _)| *id)
                    .collect();
                controlled.sort_unstable();

                // Sacrifice min(n, count) permanents.
                let to_sacrifice = controlled.into_iter().take(n).collect::<Vec<_>>();
                for id in to_sacrifice {
                    // CR 701.17a: sacrifice is NOT destruction — no indestructible check.
                    let (card_types, owner, pre_sacrifice_controller, pre_death_counters) =
                        match state.objects.get(&id) {
                            Some(obj) => (
                                obj.characteristics.card_types.clone(),
                                obj.owner,
                                obj.controller,
                                // CR 702.79a: capture counters before move_object_to_zone resets them.
                                obj.counters.clone(),
                            ),
                            None => continue,
                        };

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
                                        // Commander redirected to command zone — no sacrifice event.
                                    }
                                    _ => {
                                        if card_types.contains(&CardType::Creature) {
                                            events.push(GameEvent::CreatureDied {
                                                object_id: id,
                                                new_grave_id: new_id,
                                                controller: pre_sacrifice_controller,
                                                // CR 702.79a: last-known counter state.
                                                pre_death_counters: pre_death_counters.clone(),
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
                            // No replacement — sacrifice to graveyard normally.
                            if let Ok((new_id, _old)) =
                                state.move_object_to_zone(id, ZoneId::Graveyard(owner))
                            {
                                if card_types.contains(&CardType::Creature) {
                                    events.push(GameEvent::CreatureDied {
                                        object_id: id,
                                        new_grave_id: new_id,
                                        controller: pre_sacrifice_controller,
                                        // CR 702.79a: last-known counter state.
                                        pre_death_counters: pre_death_counters.clone(),
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

        // CR 701.60a: Suspect -- set the suspected designation on the target permanent.
        // A suspected permanent has menace and "This creature can't block" (CR 701.60c).
        // Suspecting an already-suspected permanent is a no-op (CR 701.60d).
        Effect::Suspect { target } => {
            let targets = resolve_effect_target_list(state, target, ctx);
            for resolved in targets {
                if let ResolvedTarget::Object(id) = resolved {
                    if let Some(obj) = state.objects.get_mut(&id) {
                        // CR 701.60d: A suspected permanent can't become suspected again.
                        if !obj.designations.contains(Designations::SUSPECTED) {
                            obj.designations.insert(Designations::SUSPECTED);
                            events.push(GameEvent::CreatureSuspected {
                                object_id: id,
                                controller: ctx.controller,
                            });
                        }
                    }
                }
            }
        }

        // CR 701.60a: Unsuspect -- remove the suspected designation from the target
        // permanent. Clears `is_suspected`, removing the menace grant and unblocking
        // the can't-block restriction.
        Effect::Unsuspect { target } => {
            let targets = resolve_effect_target_list(state, target, ctx);
            for resolved in targets {
                if let ResolvedTarget::Object(id) = resolved {
                    if let Some(obj) = state.objects.get_mut(&id) {
                        if obj.designations.contains(Designations::SUSPECTED) {
                            obj.designations.remove(Designations::SUSPECTED);
                            events.push(GameEvent::CreatureUnsuspected {
                                object_id: id,
                                controller: ctx.controller,
                            });
                        }
                    }
                }
            }
        }

        // CR 724.1/724.3: Target player becomes the monarch.
        // Sets state.monarch, replacing any previous monarch.
        Effect::BecomeMonarch { player } => {
            let players = resolve_player_target_list(state, player, ctx);
            if let Some(&target_player) = players.first() {
                // CR 724.3: Only one player can be the monarch at a time.
                state.monarch = Some(target_player);
                events.push(GameEvent::PlayerBecameMonarch {
                    player: target_player,
                });
            }
        }

        // CR 106.12 support: Set chosen_creature_type on the source permanent.
        // Deterministic fallback: picks the most common creature subtype among
        // creatures the controller controls, or the provided default.
        Effect::ChooseCreatureType { default } => {
            let chosen = {
                // Find the most common creature subtype among creatures the controller controls.
                let mut type_counts: std::collections::HashMap<SubType, usize> =
                    std::collections::HashMap::new();
                for obj in state.objects.values() {
                    if obj.controller == ctx.controller
                        && matches!(obj.zone, ZoneId::Battlefield)
                        && obj.characteristics.card_types.contains(&CardType::Creature)
                    {
                        for st in &obj.characteristics.subtypes {
                            *type_counts.entry(st.clone()).or_insert(0usize) += 1;
                        }
                    }
                }
                type_counts
                    .into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(st, _)| st)
                    .unwrap_or_else(|| default.clone())
            };
            if let Some(obj) = state.objects.get_mut(&ctx.source) {
                obj.chosen_creature_type = Some(chosen);
            }
        }

        // CR 701.19a: Regenerate -- create a one-shot regeneration shield on the
        // target permanent. The shield is a UntilEndOfTurn replacement effect that
        // intercepts the next WouldBeDestroyed event for this specific permanent.
        Effect::Regenerate { target } => {
            use crate::state::continuous_effect::EffectDuration;
            use crate::state::replacement_effect::{
                ObjectFilter, ReplacementEffect, ReplacementModification, ReplacementTrigger,
            };

            let targets = resolve_effect_target_list(state, target, ctx);
            for resolved in &targets {
                if let ResolvedTarget::Object(id) = resolved {
                    let id = *id;
                    // Verify the target is on the battlefield (and not phased out).
                    // CR 702.26b: phased-out permanents are treated as nonexistent.
                    let on_battlefield = state
                        .objects
                        .get(&id)
                        .map(|o| o.zone == ZoneId::Battlefield && o.is_phased_in())
                        .unwrap_or(false);
                    if !on_battlefield {
                        continue;
                    }

                    let regen_id = state.next_replacement_id();
                    state.replacement_effects.push_back(ReplacementEffect {
                        id: regen_id,
                        source: Some(id), // The permanent being protected
                        controller: ctx.controller,
                        duration: EffectDuration::UntilEndOfTurn,
                        is_self_replacement: true, // CR 614.15: self-replacement
                        trigger: ReplacementTrigger::WouldBeDestroyed {
                            filter: ObjectFilter::SpecificObject(id),
                        },
                        modification: ReplacementModification::Regenerate,
                    });

                    events.push(GameEvent::RegenerationShieldCreated {
                        object_id: id,
                        shield_id: regen_id,
                        controller: ctx.controller,
                    });
                }
            }
        }

        // CR 701.34a: Proliferate -- add one counter of each kind to each
        // permanent on the battlefield and each player that already has counters.
        // Simplified: auto-select all eligible (interactive choice deferred to M10+).
        //
        // Ruling 2023-02-04: "You can't choose cards in any zone other than the
        // battlefield, even if they have counters on them."
        // Ruling 2023-02-04: "triggers even if you chose no permanents or players."
        //
        // WARNING: The auto-select-all model can produce game-losing states that a
        // real player would never choose. For example, a player with 9 poison counters
        // who casts a proliferate spell will have their own poison incremented to 10,
        // triggering the "lose the game" SBA (CR 704.5c). In real MTG, no player would
        // choose to proliferate their own poison. CR 701.34a says "choose ANY NUMBER" --
        // the controller may choose zero or any subset of eligible permanents/players.
        // TODO(M10+): When interactive choice is added, players must be able to opt out
        // of adding harmful counters (poison, -1/-1) to themselves or their own permanents.
        Effect::Proliferate => {
            let controller = ctx.controller;

            // CR 701.34 / CR 614.1: Check for proliferate-doubling replacements (Tekuthal).
            let (proliferate_times, prolif_repl_events) =
                crate::rules::replacement::apply_proliferate_replacement(state, controller);
            events.extend(prolif_repl_events);

            for _prolif_iter in 0..proliferate_times {
                // 1. Iterate all permanents on the battlefield with at least one counter.
                //    CR ruling 2023-02-04: "You can't choose cards in any zone other
                //    than the battlefield, even if they have counters on them."
                // CR 702.26b: phased-out permanents are treated as nonexistent.
                let battlefield_objects: Vec<(
                    ObjectId,
                    Vec<(crate::state::types::CounterType, u32)>,
                )> = state
                    .objects
                    .iter()
                    .filter(|(_, obj)| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && !obj.counters.is_empty()
                    })
                    .map(|(id, obj)| {
                        let counter_types: Vec<(crate::state::types::CounterType, u32)> = obj
                            .counters
                            .iter()
                            .map(|(ct, &count)| (ct.clone(), count))
                            .collect();
                        (*id, counter_types)
                    })
                    .collect();

                // Add one counter of each kind to each eligible permanent.
                // CR 122.6: counter-placement replacements apply to each proliferate addition.
                for (obj_id, counter_types) in &battlefield_objects {
                    for (counter_type, _) in counter_types {
                        let (modified_count, repl_events) =
                            crate::rules::replacement::apply_counter_replacement(
                                state,
                                controller,
                                *obj_id,
                                counter_type,
                                1,
                            );
                        events.extend(repl_events);
                        if modified_count > 0 {
                            if let Some(obj) = state.objects.get_mut(obj_id) {
                                let cur = obj.counters.get(counter_type).copied().unwrap_or(0);
                                obj.counters
                                    .insert(counter_type.clone(), cur + modified_count);
                                events.push(GameEvent::CounterAdded {
                                    object_id: *obj_id,
                                    counter: counter_type.clone(),
                                    count: modified_count,
                                });
                            }
                        }
                    }
                }

                // 2. Iterate all players with poison counters (the only player counter type).
                //    CR 701.34a: "players that have a counter" -- poison_counters > 0.
                //
                // NOTE: Only poison counters are currently tracked on PlayerState. CR 122.1
                // recognizes additional player counter types (experience from Commander 2015,
                // energy, rad counters from CR 727). CounterType::Experience and
                // CounterType::Energy exist in the type system but have no corresponding
                // PlayerState fields. When those fields are added to PlayerState, update
                // this loop to also proliferate them.
                let eligible_players: Vec<crate::state::player::PlayerId> = state
                    .players
                    .iter()
                    .filter(|(_, ps)| !ps.has_lost && ps.poison_counters > 0)
                    .map(|(id, _)| *id)
                    .collect();

                for pid in &eligible_players {
                    if let Some(player) = state.players.get_mut(pid) {
                        player.poison_counters += 1;
                        events.push(GameEvent::PoisonCountersGiven {
                            player: *pid,
                            amount: 1,
                            source: ctx.source,
                        });
                    }
                }

                // 3. Always emit Proliferated event (ruling 2023-02-04:
                //    "triggers even if you chose no permanents or players").
                events.push(GameEvent::Proliferated {
                    controller,
                    permanents_affected: battlefield_objects.len() as u32,
                    players_affected: eligible_players.len() as u32,
                });
            } // end proliferate_times loop
        }

        // CR 701.57a: Discover N — exile cards from the top of the specified
        // player's library until you exile a nonland card with mana value <= N.
        // You may cast that card without paying its mana cost. If you don't cast
        // it, put that card into your hand. Put the remaining exiled cards on the
        // bottom of your library in a random order.
        //
        // Key differences from Cascade (CR 702.85):
        // - MV threshold is <= N (not < spell_MV like Cascade)
        // - Declined card goes to hand (Cascade puts all non-cast cards on library bottom)
        //
        // CR 701.57b: Always completes even if no qualifying card is found
        // (empty library, all lands, etc.).
        //
        // Deterministic fallback: always casts the discovered card (interactive
        // "may cast" choice deferred to M10+).
        Effect::Discover { player, n } => {
            // Resolve the PlayerTarget to a single PlayerId. Discover is always
            // performed by one player (the controller or a specified opponent).
            let players = resolve_player_target_list(state, player, ctx);
            if let Some(player_id) = players.into_iter().next() {
                let (discover_events, _result_id) =
                    crate::rules::copy::resolve_discover(state, player_id, *n);
                events.extend(discover_events);
            }
        }

        // CR 701.40a: Manifest the top card of a player's library.
        // The card is placed onto the battlefield face-down as a 2/2 creature with no
        // text, no name, no subtypes, and no mana cost. ETB abilities do not trigger
        // (CR 708.3 — the permanent enters face-down). If the library is empty, do nothing.
        Effect::Manifest {
            player: player_target,
        } => {
            let manifest_player = match player_target {
                PlayerTarget::Controller | PlayerTarget::EachPlayer => ctx.controller,
                PlayerTarget::DeclaredTarget { index } => {
                    // For manifest effects that target a player, use the declared target.
                    // In practice, Manifest is almost always on Controller.
                    let _ = index;
                    ctx.controller
                }
                PlayerTarget::EachOpponent => {
                    // Manifest an opponent's card (unusual but rules-legal).
                    // Default to the first opponent.
                    state
                        .players
                        .keys()
                        .find(|&&pid| pid != ctx.controller)
                        .copied()
                        .unwrap_or(ctx.controller)
                }
                PlayerTarget::ControllerOf(_) | PlayerTarget::OwnerOf(_) => ctx.controller,
            };
            let lib_id = ZoneId::Library(manifest_player);
            let top_card = state.zones.get(&lib_id).and_then(|z| z.top());
            if let Some(top_id) = top_card {
                if let Ok((new_id, _)) = state.move_object_to_zone(top_id, ZoneId::Battlefield) {
                    // CR 708.2 / CR 708.3: Set face-down status; the card enters as a 2/2 creature.
                    // The layer system handles characteristic override.
                    if let Some(obj) = state.objects.get_mut(&new_id) {
                        obj.controller = manifest_player;
                        obj.owner = manifest_player;
                        obj.status.face_down = true;
                        obj.face_down_as = Some(crate::state::types::FaceDownKind::Manifest);
                    }
                    // CR 708.3: ETB abilities do NOT trigger (face-down entry).
                    // PermanentEnteredBattlefield is still emitted (global triggers CAN fire
                    // if they watch for any creature entering — they see a 2/2).
                    events.push(GameEvent::PermanentEnteredBattlefield {
                        player: manifest_player,
                        object_id: new_id,
                    });
                }
            }
            // If library is empty, the effect does nothing (CR 701.40f).
        }

        // CR 701.58a: Cloak the top card of a player's library.
        // Like Manifest (CR 701.40a), but the face-down creature also has ward {2}
        // (CR 701.58a) while face-down. The ward {2} is added by the layer system
        // when face_down_as == Some(Cloak).
        Effect::Cloak {
            player: player_target,
        } => {
            let cloak_player = match player_target {
                PlayerTarget::Controller | PlayerTarget::EachPlayer => ctx.controller,
                PlayerTarget::DeclaredTarget { index } => {
                    let _ = index;
                    ctx.controller
                }
                PlayerTarget::EachOpponent => state
                    .players
                    .keys()
                    .find(|&&pid| pid != ctx.controller)
                    .copied()
                    .unwrap_or(ctx.controller),
                PlayerTarget::ControllerOf(_) | PlayerTarget::OwnerOf(_) => ctx.controller,
            };
            let lib_id = ZoneId::Library(cloak_player);
            let top_card = state.zones.get(&lib_id).and_then(|z| z.top());
            if let Some(top_id) = top_card {
                if let Ok((new_id, _)) = state.move_object_to_zone(top_id, ZoneId::Battlefield) {
                    // CR 701.58a: Set face-down status with Cloak kind.
                    // The layer system adds ward {2} for Cloak permanents.
                    if let Some(obj) = state.objects.get_mut(&new_id) {
                        obj.controller = cloak_player;
                        obj.owner = cloak_player;
                        obj.status.face_down = true;
                        obj.face_down_as = Some(crate::state::types::FaceDownKind::Cloak);
                    }
                    // CR 708.3: ETB abilities do NOT trigger (face-down entry).
                    events.push(GameEvent::PermanentEnteredBattlefield {
                        player: cloak_player,
                        object_id: new_id,
                    });
                }
            }
            // If library is empty, the effect does nothing.
        }

        Effect::Nothing => {}

        // CR 701.49: Venture into the dungeon.
        //
        // Invokes the three-case venture logic (no dungeon, mid-dungeon, bottommost room).
        // Deterministic fallback: LostMineOfPhandelver when choosing a new dungeon.
        // After advancing the marker, a RoomAbility SOK is pushed onto the stack (CR 309.4c).
        Effect::VentureIntoDungeon => {
            let controller = ctx.controller;
            if let Ok(venture_events) =
                crate::rules::engine::handle_venture_into_dungeon(state, controller, false)
            {
                events.extend(venture_events);
            }
        }

        // CR 725.2: Take the initiative.
        //
        // Sets `has_initiative = Some(controller)` on GameState, emits InitiativeTaken,
        // and immediately ventures into the Undercity (CR 725.2 inherent trigger).
        Effect::TakeTheInitiative => {
            let controller = ctx.controller;
            state.has_initiative = Some(controller);
            events.push(GameEvent::InitiativeTaken { player: controller });
            // CR 725.2: Taking the initiative also ventures into the Undercity.
            if let Ok(venture_events) =
                crate::rules::engine::handle_venture_into_dungeon(state, controller, true)
            {
                events.extend(venture_events);
            }
        }

        // CR 701.54a-c: The Ring tempts you.
        //
        // Advances the controller's ring level (cap at 4), emits RingTempted,
        // then chooses a creature the controller controls as their ring-bearer.
        // Deterministic fallback: creature with the lowest ObjectId.
        // If no creature is available, ring level still advances but no ring-bearer
        // is chosen (CR 701.54a — the temptation still occurs).
        Effect::TheRingTemptsYou => {
            let controller = ctx.controller;
            if let Ok(ring_events) = crate::rules::engine::handle_ring_tempts_you(state, controller)
            {
                events.extend(ring_events);
            }
        }

        // CR 701.42a: Meld the source permanent with its meld pair partner.
        //
        // 1. Look up the source's MeldPair to find the partner card_id
        // 2. Find the partner on the battlefield (same owner+controller)
        // 3. Exile both cards
        // 4. Create a new permanent with meld_component set to the partner's card_id
        //    The layer system handles melded face characteristics (CR 712.8g)
        //
        // CR 701.42c: If conditions aren't met, nothing happens.
        Effect::Meld => {
            let source_id = ctx.source;
            let controller = ctx.controller;

            // Look up source card's meld pair info.
            let meld_info = state
                .objects
                .get(&source_id)
                .and_then(|obj| obj.card_id.clone())
                .and_then(|cid| {
                    state.card_registry.get(cid.clone()).and_then(|def| {
                        def.meld_pair
                            .as_ref()
                            .map(|mp| (cid, mp.pair_card_id.clone(), mp.melded_card_id.clone()))
                    })
                });

            if let Some((source_card_id, pair_card_id, _melded_card_id)) = meld_info {
                // Find the partner on the battlefield — must be owned AND controlled
                // by the same player (CR 712.4a: "you both own and control").
                let partner_id = state
                    .objects
                    .values()
                    .find(|obj| {
                        obj.zone == crate::state::zone::ZoneId::Battlefield
                            && obj.card_id.as_ref() == Some(&pair_card_id)
                            && obj.owner == controller
                            && obj.controller == controller
                    })
                    .map(|obj| obj.id);

                // Also verify source is still on the battlefield and owned+controlled.
                let source_valid = state
                    .objects
                    .get(&source_id)
                    .map(|obj| {
                        obj.zone == crate::state::zone::ZoneId::Battlefield
                            && obj.owner == controller
                            && obj.controller == controller
                    })
                    .unwrap_or(false);

                if let Some(partner_obj_id) = partner_id {
                    if source_valid {
                        // Exile both cards transiently (zone-change bookkeeping for CR 400.7).
                        // The exiled objects will be removed after the melded permanent is
                        // created — they are phantom intermediaries, not real exile zone
                        // residents (CR 701.42a: cards go directly onto the battlefield combined).
                        let exile_zone = crate::state::zone::ZoneId::Exile;
                        let exiled_source_id = state
                            .move_object_to_zone(source_id, exile_zone)
                            .map(|(new_id, _)| new_id)
                            .ok();
                        let exiled_partner_id = state
                            .move_object_to_zone(partner_obj_id, exile_zone)
                            .map(|(new_id, _)| new_id)
                            .ok();

                        // Create the melded permanent on the battlefield.
                        // The primary object uses the source card's identity;
                        // meld_component stores the partner's CardId for zone-change splitting.
                        // The layer system reads meld_component to apply melded face chars.
                        let melded_id = state.next_object_id();
                        state.timestamp_counter += 1;

                        // Start with default characteristics — the layer system
                        // will replace them with the melded back face (CR 712.8g).
                        let melded_obj = crate::state::game_object::GameObject {
                            id: melded_id,
                            card_id: Some(source_card_id),
                            characteristics: crate::state::game_object::Characteristics::default(),
                            controller,
                            owner: controller,
                            zone: crate::state::zone::ZoneId::Battlefield,
                            status: crate::state::game_object::ObjectStatus::default(),
                            counters: im::OrdMap::new(),
                            attachments: im::Vector::new(),
                            attached_to: None,
                            damage_marked: 0,
                            deathtouch_damage: false,
                            is_token: false,
                            timestamp: state.timestamp_counter,
                            has_summoning_sickness: true,
                            goaded_by: im::Vector::new(),
                            kicker_times_paid: 0,
                            cast_alt_cost: None,
                            foretold_turn: 0,
                            was_unearthed: false,
                            myriad_exile_at_eoc: false,
                            decayed_sacrifice_at_eoc: false,
                            ring_block_sacrifice_at_eoc: false,
                            exiled_by_hideaway: None,
                            encore_sacrifice_at_end_step: false,
                            encore_must_attack: None,
                            encore_activated_by: None,
                            is_plotted: false,
                            plotted_turn: 0,
                            is_prototyped: false,
                            was_bargained: false,
                            evidence_collected: false,
                            phased_out_indirectly: false,
                            phased_out_controller: None,
                            creatures_devoured: 0,
                            champion_exiled_card: None,
                            paired_with: None,
                            tribute_was_paid: false,
                            x_value: 0,
                            squad_count: 0,
                            offspring_paid: false,
                            gift_was_given: false,
                            gift_opponent: None,
                            encoded_cards: im::Vector::new(),
                            haunting_target: None,
                            merged_components: im::Vector::new(),
                            is_transformed: false,
                            last_transform_timestamp: 0,
                            was_cast_disturbed: false,
                            craft_exiled_cards: im::Vector::new(),
                            chosen_creature_type: None,
                            face_down_as: None,
                            loyalty_ability_activated_this_turn: false,
                            class_level: 0,
                            designations: crate::state::game_object::Designations::default(),
                            meld_component: Some(pair_card_id),
                        };

                        // Add to battlefield zone.
                        if let Some(zone_set) = state
                            .zones
                            .get_mut(&crate::state::zone::ZoneId::Battlefield)
                        {
                            zone_set.insert(melded_id);
                        }
                        state.objects.insert(melded_id, melded_obj);

                        // Remove the two phantom exile objects that were created as
                        // zone-change intermediaries. CR 701.42a puts both cards onto the
                        // battlefield combined — they should not persist in exile.
                        for phantom_id in
                            [exiled_source_id, exiled_partner_id].into_iter().flatten()
                        {
                            state.objects.remove(&phantom_id);
                            if let Some(exile_set) =
                                state.zones.get_mut(&crate::state::zone::ZoneId::Exile)
                            {
                                exile_set.remove(&phantom_id);
                            }
                        }

                        events.push(
                            crate::rules::events::GameEvent::PermanentEnteredBattlefield {
                                player: controller,
                                object_id: melded_id,
                            },
                        );
                    }
                }
                // CR 701.42c: If partner not found or conditions not met, nothing happens.
            }
        }

        // CR 702.75a / CR 607.2a: Play the card exiled face-down by this permanent's
        // Hideaway ETB trigger without paying its mana cost.
        //
        // Searches the exile zone for an object where:
        //   - obj.exiled_by_hideaway == Some(ctx.source)
        //   - obj.status.face_down == true
        // If found: turns it face-up, clears exiled_by_hideaway, and moves it to the
        // battlefield (for permanents) or handles it as a simplified free cast.
        //
        // Deterministic fallback: always plays the card (does not decline).
        // CR 118.9: Playing without paying mana cost is an alternative cost.
        // CR 701.13: "Play" includes playing lands AND casting spells.
        Effect::PlayExiledCard => {
            let source_id = ctx.source;
            let controller = ctx.controller;

            // Find the hideaway-exiled card in exile.
            let hideaway_card_id = state
                .objects
                .iter()
                .find(|(_, obj)| {
                    obj.zone == ZoneId::Exile
                        && obj.exiled_by_hideaway == Some(source_id)
                        && obj.status.face_down
                })
                .map(|(id, _)| *id);

            if let Some(card_id) = hideaway_card_id {
                // Determine if the card is a land or a spell.
                let is_land = state
                    .objects
                    .get(&card_id)
                    .map(|obj| obj.characteristics.card_types.contains(&CardType::Land))
                    .unwrap_or(false);

                // Turn face-up and clear the hideaway link.
                if let Some(obj) = state.objects.get_mut(&card_id) {
                    obj.status.face_down = false;
                    obj.exiled_by_hideaway = None;
                }

                if is_land {
                    // Play the land: move directly to battlefield.
                    // CR 701.13: Playing a land bypasses the stack.
                    match state.move_object_to_zone(card_id, ZoneId::Battlefield) {
                        Ok((new_id, _)) => {
                            if let Some(obj) = state.objects.get_mut(&new_id) {
                                obj.controller = controller;
                                obj.owner = controller;
                            }
                            events.push(GameEvent::PermanentEnteredBattlefield {
                                player: controller,
                                object_id: new_id,
                            });
                        }
                        Err(_) => {
                            // Card disappeared from exile — ability does nothing.
                        }
                    }
                } else {
                    // Cast as permanent without paying mana cost.
                    // Simplified: move directly to the battlefield for permanent cards,
                    // or to the graveyard for instants/sorceries (they "resolve" immediately).
                    // CR 118.9: alternative cost of {0}.
                    let is_permanent = state
                        .objects
                        .get(&card_id)
                        .map(|obj| {
                            let ct = &obj.characteristics.card_types;
                            ct.contains(&CardType::Creature)
                                || ct.contains(&CardType::Artifact)
                                || ct.contains(&CardType::Enchantment)
                                || ct.contains(&CardType::Planeswalker)
                        })
                        .unwrap_or(false);

                    if is_permanent {
                        if let Ok((new_id, _)) =
                            state.move_object_to_zone(card_id, ZoneId::Battlefield)
                        {
                            if let Some(obj) = state.objects.get_mut(&new_id) {
                                obj.controller = controller;
                                obj.owner = controller;
                            }
                            events.push(GameEvent::PermanentEnteredBattlefield {
                                player: controller,
                                object_id: new_id,
                            });
                        }
                    } else {
                        // Instant/sorcery: move to graveyard (simplified resolution).
                        let graveyard_zone = ZoneId::Graveyard(controller);
                        if let Ok((new_id, _)) = state.move_object_to_zone(card_id, graveyard_zone)
                        {
                            events.push(GameEvent::ObjectPutInGraveyard {
                                player: controller,
                                object_id: card_id,
                                new_grave_id: new_id,
                            });
                        }
                    }
                }
            }
            // If no matching exiled card found, the ability does nothing (CR 607.2a).
        }

        // CR 702.6a / CR 701.3a: Attach the source Equipment to the target creature.
        //
        // On resolution:
        // 1. Detach from any previously equipped creature (CR 301.5c: can't equip more than one).
        // 2. Set source.attached_to = target; add source to target.attachments.
        // 3. Update Equipment timestamp (CR 701.3c, CR 613.7e).
        //
        // If the target is no longer a creature on the battlefield under the activating player's
        // control, or if the equipment would equip itself (CR 301.5c), the effect is skipped
        // for that pair.
        Effect::AttachEquipment { equipment, target } => {
            let equip_resolved = resolve_effect_target_list(state, equipment, ctx);
            let target_resolved = resolve_effect_target_list(state, target, ctx);

            for equip_res in &equip_resolved {
                let equip_id = match equip_res {
                    ResolvedTarget::Object(id) => *id,
                    _ => continue,
                };
                for target_res in &target_resolved {
                    let target_id = match target_res {
                        ResolvedTarget::Object(id) => *id,
                        _ => continue,
                    };

                    // CR 301.5c: Equipment can't equip itself.
                    if equip_id == target_id {
                        continue;
                    }

                    // Validate: target must be a creature on the battlefield controlled
                    // by the ability's controller.
                    //
                    // CR 702.6a: "target creature you control."
                    // Use layer-computed characteristics so that permanents whose types
                    // were changed by a continuous effect (e.g. animated artifacts, or
                    // creatures stripped of their type by Humility) are evaluated
                    // correctly (Finding 1 fix — layer-aware creature type check).
                    // CR 702.26b: phased-out permanents are treated as nonexistent.
                    let target_on_battlefield_and_controlled = state
                        .objects
                        .get(&target_id)
                        .map(|obj| {
                            obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && obj.controller == ctx.controller
                        })
                        .unwrap_or(false);
                    let target_is_creature = {
                        let layer_chars =
                            crate::rules::layers::calculate_characteristics(state, target_id)
                                .or_else(|| {
                                    state
                                        .objects
                                        .get(&target_id)
                                        .map(|o| o.characteristics.clone())
                                });
                        layer_chars
                            .map(|chars| chars.card_types.contains(&CardType::Creature))
                            .unwrap_or(false)
                    };
                    let target_valid = target_on_battlefield_and_controlled && target_is_creature;

                    if !target_valid {
                        // CR 701.3b: Can't attach to an illegal target; do nothing.
                        continue;
                    }

                    // CR 701.3b: Already attached to the same target — do nothing.
                    if state
                        .objects
                        .get(&equip_id)
                        .and_then(|o| o.attached_to)
                        .map(|att| att == target_id)
                        .unwrap_or(false)
                    {
                        continue;
                    }

                    // Detach from previous creature (CR 301.5c: can't equip more than one).
                    let prev_target_opt = state.objects.get(&equip_id).and_then(|o| o.attached_to);
                    if let Some(prev_target) = prev_target_opt {
                        if let Some(prev) = state.objects.get_mut(&prev_target) {
                            prev.attachments.retain(|&x| x != equip_id);
                        }
                    }

                    // Attach to new target.
                    // CR 701.3c / CR 613.7e: new timestamp on reattach.
                    state.timestamp_counter += 1;
                    let new_ts = state.timestamp_counter;
                    if let Some(equip_obj) = state.objects.get_mut(&equip_id) {
                        equip_obj.attached_to = Some(target_id);
                        equip_obj.timestamp = new_ts;
                    }
                    if let Some(target_obj) = state.objects.get_mut(&target_id) {
                        if !target_obj.attachments.contains(&equip_id) {
                            target_obj.attachments.push_back(equip_id);
                        }
                    }

                    events.push(GameEvent::EquipmentAttached {
                        equipment_id: equip_id,
                        target_id,
                        controller: ctx.controller,
                    });

                    // CR 702.151b: If the Equipment has the Reconfigure keyword (by any means),
                    // set is_reconfigured = true so it stops being a creature while attached.
                    // Ruling 2022-02-18: effect persists even if keyword is later removed.
                    let has_reconfigure =
                        crate::rules::layers::calculate_characteristics(state, equip_id)
                            .map(|chars| {
                                chars.keywords.iter().any(|k| {
                                    matches!(k, crate::state::types::KeywordAbility::Reconfigure)
                                })
                            })
                            .unwrap_or(false);
                    if has_reconfigure {
                        if let Some(equip_obj) = state.objects.get_mut(&equip_id) {
                            equip_obj.designations.insert(Designations::RECONFIGURED);
                        }
                    }
                }
            }
        }
        // CR 702.67a / CR 701.3a: Fortify -- attach the source Fortification to
        // the target land controlled by the activating player.
        //
        // On resolution:
        // 1. Detach from any previously fortified land (CR 301.6 via 301.5c analog).
        // 2. Set source.attached_to = target; add source to target.attachments.
        // 3. Update Fortification timestamp (CR 701.3c, CR 613.7e).
        //
        // If the target is no longer a land on the battlefield under the activating
        // player's control, the effect is skipped.
        Effect::AttachFortification {
            fortification,
            target,
        } => {
            let equip_resolved = resolve_effect_target_list(state, fortification, ctx);
            let target_resolved = resolve_effect_target_list(state, target, ctx);

            for equip_res in &equip_resolved {
                let equip_id = match equip_res {
                    ResolvedTarget::Object(id) => *id,
                    _ => continue,
                };
                for target_res in &target_resolved {
                    let target_id = match target_res {
                        ResolvedTarget::Object(id) => *id,
                        _ => continue,
                    };

                    // CR 301.6 (via 301.5c analog): A Fortification that is also a
                    // creature can't fortify a land; a Fortification can't fortify
                    // itself (trivially true since it's an artifact, not a land).
                    if equip_id == target_id {
                        continue;
                    }

                    // CR 301.6: A Fortification that's also a creature can't fortify a land.
                    // This can happen via animation (e.g. March of the Machines).
                    let source_is_creature = {
                        let layer_chars =
                            crate::rules::layers::calculate_characteristics(state, equip_id)
                                .or_else(|| {
                                    state
                                        .objects
                                        .get(&equip_id)
                                        .map(|o| o.characteristics.clone())
                                });
                        layer_chars
                            .map(|chars| chars.card_types.contains(&CardType::Creature))
                            .unwrap_or(false)
                    };
                    if source_is_creature {
                        continue; // CR 301.6: creature Fortification can't fortify
                    }

                    // Validate: target must be a land on the battlefield controlled
                    // by the ability's controller.
                    //
                    // CR 702.67a: "target land you control."
                    // Use layer-computed characteristics so that permanents whose types
                    // were changed by a continuous effect are evaluated correctly.
                    // CR 702.26b: phased-out permanents are treated as nonexistent.
                    let target_on_battlefield_and_controlled = state
                        .objects
                        .get(&target_id)
                        .map(|obj| {
                            obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && obj.controller == ctx.controller
                        })
                        .unwrap_or(false);
                    let target_is_land = {
                        let layer_chars =
                            crate::rules::layers::calculate_characteristics(state, target_id)
                                .or_else(|| {
                                    state
                                        .objects
                                        .get(&target_id)
                                        .map(|o| o.characteristics.clone())
                                });
                        layer_chars
                            .map(|chars| chars.card_types.contains(&CardType::Land))
                            .unwrap_or(false)
                    };
                    let target_valid = target_on_battlefield_and_controlled && target_is_land;

                    if !target_valid {
                        // CR 701.3b: Can't attach to an illegal target; do nothing.
                        continue;
                    }

                    // CR 701.3b: Already attached to the same land — do nothing.
                    if state
                        .objects
                        .get(&equip_id)
                        .and_then(|o| o.attached_to)
                        .map(|att| att == target_id)
                        .unwrap_or(false)
                    {
                        continue;
                    }

                    // Detach from previous land (CR 301.6 via 301.5c analog: can't
                    // fortify more than one land).
                    let prev_target_opt = state.objects.get(&equip_id).and_then(|o| o.attached_to);
                    if let Some(prev_target) = prev_target_opt {
                        if let Some(prev) = state.objects.get_mut(&prev_target) {
                            prev.attachments.retain(|&x| x != equip_id);
                        }
                    }

                    // Attach to new land.
                    // CR 701.3c / CR 613.7e: new timestamp on reattach.
                    state.timestamp_counter += 1;
                    let new_ts = state.timestamp_counter;
                    if let Some(equip_obj) = state.objects.get_mut(&equip_id) {
                        equip_obj.attached_to = Some(target_id);
                        equip_obj.timestamp = new_ts;
                    }
                    if let Some(target_obj) = state.objects.get_mut(&target_id) {
                        if !target_obj.attachments.contains(&equip_id) {
                            target_obj.attachments.push_back(equip_id);
                        }
                    }

                    events.push(GameEvent::FortificationAttached {
                        fortification_id: equip_id,
                        target_id,
                        controller: ctx.controller,
                    });
                }
            }
        }
        // CR 702.151a: Reconfigure unattach -- "[Cost]: Unattach this permanent."
        //
        // On resolution:
        // 1. Resolve the equipment target (should be EffectTarget::Source).
        // 2. Check that the equipment is on the battlefield and has attached_to.
        // 3. Clear attached_to on the equipment.
        // 4. Remove the equipment from the target's attachments.
        // 5. Clear is_reconfigured flag (creature type is restored; CR 702.151b).
        // 6. Emit EquipmentUnattached event.
        Effect::DetachEquipment { equipment } => {
            let equip_resolved = resolve_effect_target_list(state, equipment, ctx);

            for equip_res in &equip_resolved {
                let equip_id = match equip_res {
                    ResolvedTarget::Object(id) => *id,
                    _ => continue,
                };

                // Verify equipment is on the battlefield.
                let on_battlefield = state
                    .objects
                    .get(&equip_id)
                    .map(|obj| obj.zone == ZoneId::Battlefield)
                    .unwrap_or(false);
                if !on_battlefield {
                    continue;
                }

                // Get the current attachment target.
                let target_id_opt = state.objects.get(&equip_id).and_then(|obj| obj.attached_to);
                let Some(target_id) = target_id_opt else {
                    // Not attached; do nothing (CR 702.151a: "Activate only if attached").
                    continue;
                };

                // Clear attached_to on the equipment.
                if let Some(equip_obj) = state.objects.get_mut(&equip_id) {
                    equip_obj.attached_to = None;
                    // CR 702.151b: Clear the reconfigure flag; creature type is restored.
                    equip_obj.designations.remove(Designations::RECONFIGURED);
                }

                // Remove equipment from target's attachments.
                if let Some(target_obj) = state.objects.get_mut(&target_id) {
                    target_obj.attachments.retain(|&x| x != equip_id);
                }

                events.push(GameEvent::EquipmentUnattached {
                    object_id: equip_id,
                });
            }
        }
        // CR 701.50a/e: Connive N — the permanent's controller draws N cards,
        // discards N cards, then puts a +1/+1 counter on the permanent for each
        // nonland card discarded this way.
        // CR 701.50b: The permanent "connives" even if some actions were impossible.
        // CR 701.50c: If the permanent left the battlefield, no counter is placed.
        Effect::Connive { target, count } => {
            let n = resolve_amount(state, count, ctx).max(0) as usize;
            let targets = resolve_effect_target_list(state, target, ctx);

            for resolved in targets {
                if let ResolvedTarget::Object(creature_id) = resolved {
                    // CR 701.50a: The permanent's CONTROLLER draws and discards.
                    // CR 701.50c: If the permanent left the battlefield, fall back
                    // to ctx.controller so the controller still draws/discards.
                    let controller = state
                        .objects
                        .get(&creature_id)
                        .map(|obj| obj.controller)
                        .unwrap_or(ctx.controller);

                    // Step 1: Draw N cards (CR 701.50e).
                    for _ in 0..n {
                        let draw_evts = draw_one_card(state, controller);
                        events.extend(draw_evts);
                    }

                    // Step 2: Discard N cards and count nonland discards.
                    // Cannot reuse discard_cards helper — we need per-card type info
                    // to determine the counter count. Inline the discard logic here.
                    let hand_zone = ZoneId::Hand(controller);
                    let mut nonland_count: u32 = 0;

                    for _ in 0..n {
                        // Deterministic: discard the card with the smallest ObjectId.
                        let card_id = state
                            .objects
                            .iter()
                            .filter(|(_, obj)| obj.zone == hand_zone)
                            .map(|(&id, _)| id)
                            .min_by_key(|id| id.0);

                        if let Some(card_id) = card_id {
                            // CR 701.50a: Check if the discarded card is nonland
                            // BEFORE moving it (card types survive in hand only).
                            let is_nonland = state
                                .objects
                                .get(&card_id)
                                .map(|obj| {
                                    !obj.characteristics.card_types.contains(&CardType::Land)
                                })
                                .unwrap_or(false);

                            if is_nonland {
                                nonland_count += 1;
                            }

                            // CR 702.35a: Check for Madness before zone change.
                            let obj_card_id =
                                state.objects.get(&card_id).and_then(|o| o.card_id.clone());
                            let has_madness = state
                                .objects
                                .get(&card_id)
                                .map(|obj| {
                                    obj.characteristics
                                        .keywords
                                        .contains(&KeywordAbility::Madness)
                                })
                                .unwrap_or(false);

                            let destination = if has_madness {
                                ZoneId::Exile
                            } else {
                                ZoneId::Graveyard(controller)
                            };

                            if let Ok((new_id, _)) = state.move_object_to_zone(card_id, destination)
                            {
                                // CR ruling: CardDiscarded fires even when card goes to exile.
                                events.push(GameEvent::CardDiscarded {
                                    player: controller,
                                    object_id: card_id,
                                    new_id,
                                });

                                if has_madness {
                                    // CR 702.35a: Look up madness cost and queue trigger.
                                    let madness_cost = obj_card_id.as_ref().and_then(|cid| {
                                        state.card_registry.get(cid.clone()).and_then(|def| {
                                            def.abilities.iter().find_map(|a| {
                                                if let crate::cards::card_definition::AbilityDefinition::Madness { cost } = a {
                                                    Some(cost.clone())
                                                } else {
                                                    None
                                                }
                                            })
                                        })
                                    });
                                    state.pending_triggers.push_back(PendingTrigger {
                                        source: new_id,
                                        ability_index: 0,
                                        controller,
                                        kind: PendingTriggerKind::Madness,
                                        triggering_event: None,
                                        entering_object_id: None,
                                        targeting_stack_id: None,
                                        triggering_player: None,
                                        exalted_attacker_id: None,
                                        defending_player_id: None,
                                        madness_exiled_card: Some(new_id),
                                        madness_cost,
                                        miracle_revealed_card: None,
                                        miracle_cost: None,
                                        modular_counter_count: None,
                                        evolve_entering_creature: None,
                                        suspend_card_id: None,
                                        hideaway_count: None,
                                        partner_with_name: None,
                                        ingest_target_player: None,
                                        flanking_blocker_id: None,
                                        rampage_n: None,
                                        provoke_target_creature: None,
                                        renown_n: None,
                                        poisonous_n: None,
                                        poisonous_target_player: None,
                                        enlist_enlisted_creature: None,
                                        encore_activator: None,
                                        echo_cost: None,
                                        cumulative_upkeep_cost: None,
                                        recover_cost: None,
                                        recover_card: None,
                                        graft_entering_creature: None,
                                        backup_abilities: None,
                                        backup_n: None,
                                        champion_filter: None,
                                        champion_exiled_card: None,
                                        soulbond_pair_target: None,
                                        squad_count: None,
                                        gift_opponent: None,
                                        cipher_encoded_card_id: None,
                                        cipher_encoded_object_id: None,
                                        haunt_source_object_id: None,
                                        haunt_source_card_id: None,
                                    });
                                }
                            }
                        }
                    }

                    // Step 3: Place +1/+1 counters on the conniving permanent.
                    // CR 701.50c: Only if the permanent is still on the battlefield.
                    // CR 702.26b: phased-out permanents are treated as nonexistent.
                    let creature_on_battlefield = state
                        .objects
                        .get(&creature_id)
                        .map(|o| o.zone == ZoneId::Battlefield && o.is_phased_in())
                        .unwrap_or(false);

                    if nonland_count > 0 && creature_on_battlefield {
                        if let Some(obj) = state.objects.get_mut(&creature_id) {
                            let cur = obj
                                .counters
                                .get(&crate::state::types::CounterType::PlusOnePlusOne)
                                .copied()
                                .unwrap_or(0);
                            obj.counters.insert(
                                crate::state::types::CounterType::PlusOnePlusOne,
                                cur + nonland_count,
                            );
                            events.push(GameEvent::CounterAdded {
                                object_id: creature_id,
                                counter: crate::state::types::CounterType::PlusOnePlusOne,
                                count: nonland_count,
                            });
                        }
                    }

                    // CR 701.50b: The permanent "connives" regardless of whether
                    // actions were possible. Emit Connived for trigger support.
                    events.push(GameEvent::Connived {
                        object_id: creature_id,
                        player: controller,
                        counters_placed: if creature_on_battlefield {
                            nonland_count
                        } else {
                            0
                        },
                    });
                }
            }
        }
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
        // CR 702.26b: phased-out permanents are treated as nonexistent.
        EffectTarget::AllCreatures => state
            .objects
            .iter()
            .filter(|(_, obj)| {
                obj.zone == ZoneId::Battlefield
                    && obj.is_phased_in()
                    && obj.characteristics.card_types.contains(&CardType::Creature)
            })
            .map(|(&id, _)| (None, ResolvedTarget::Object(id)))
            .collect(),
        EffectTarget::AllPermanents => state
            .objects
            .iter()
            .filter(|(_, obj)| obj.zone == ZoneId::Battlefield && obj.is_phased_in())
            .map(|(&id, _)| (None, ResolvedTarget::Object(id)))
            .collect(),
        EffectTarget::AllPermanentsMatching(filter) => state
            .objects
            .iter()
            .filter(|(_, obj)| {
                obj.zone == ZoneId::Battlefield
                    && obj.is_phased_in()
                    && matches_filter(&obj.characteristics, filter)
                    && match filter.controller {
                        TargetController::Any => true,
                        TargetController::You => obj.controller == ctx.controller,
                        TargetController::Opponent => obj.controller != ctx.controller,
                    }
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
        PlayerTarget::OwnerOf(effect_target) => {
            // CR 108.3: The owner of a card is the player who started the game with it in
            // their deck. Used for bounce effects that say "return to its owner's hand."
            let targets = resolve_effect_target_list(state, effect_target, ctx);
            targets
                .into_iter()
                .filter_map(|t| {
                    if let ResolvedTarget::Object(id) = t {
                        state.objects.get(&id).map(|obj| obj.owner)
                    } else {
                        None
                    }
                })
                .collect()
        }
    }
}

// ── Mana restriction helpers ──────────────────────────────────────────────────

/// Resolve a chosen-type mana restriction to a concrete restriction by looking up
/// `chosen_creature_type` from the source permanent.
fn resolve_mana_restriction(
    state: &GameState,
    restriction: &Option<ManaRestriction>,
    ctx: &EffectContext,
) -> Option<ManaRestriction> {
    match restriction {
        None => None,
        Some(ManaRestriction::ChosenTypeCreaturesOnly)
        | Some(ManaRestriction::ChosenTypeSpellsOnly) => {
            // Look up the chosen creature type from the source permanent.
            let chosen = state
                .object(ctx.source)
                .ok()
                .and_then(|obj| obj.chosen_creature_type.clone());
            match chosen {
                Some(st) => match restriction.as_ref().unwrap() {
                    // CR 106.6: "creature spell of the chosen type" requires both creature
                    // AND subtype check. Use CreatureWithSubtype to enforce this.
                    ManaRestriction::ChosenTypeCreaturesOnly => {
                        Some(ManaRestriction::CreatureWithSubtype(st))
                    }
                    // ChosenTypeSpellsOnly: any spell of the subtype (no creature requirement).
                    _ => Some(ManaRestriction::SubtypeOnly(st)),
                },
                // No chosen type set — fall back to creature-only as a safe restriction
                None => match restriction.as_ref().unwrap() {
                    ManaRestriction::ChosenTypeCreaturesOnly => {
                        Some(ManaRestriction::CreatureSpellsOnly)
                    }
                    _ => None,
                },
            }
        }
        Some(r) => Some(r.clone()),
    }
}

/// Add mana to a player's pool, with optional restriction (CR 106.12).
fn add_mana_with_restriction(
    ps: &mut crate::state::player::PlayerState,
    color: ManaColor,
    amount: u32,
    restriction: &Option<ManaRestriction>,
) {
    match restriction {
        Some(r) => ps.mana_pool.add_restricted(color, amount, r.clone()),
        None => ps.mana_pool.add(color, amount),
    }
}

// ── Amount resolution ─────────────────────────────────────────────────────────

/// Resolve an `EffectAmount` to a concrete integer value.
fn resolve_amount(state: &GameState, amount: &EffectAmount, ctx: &EffectContext) -> i32 {
    match amount {
        EffectAmount::Fixed(n) => *n,
        // CR 107.3m: X resolves to the value chosen at cast time, stored in ctx.x_value.
        EffectAmount::XValue => ctx.x_value as i32,
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
        EffectAmount::PermanentCount { filter, controller } => {
            // Count permanents on the battlefield matching filter, controlled by the
            // resolved player. Phased-out permanents are excluded (CR 702.26d).
            let players = resolve_player_target_list(state, controller, ctx);
            state
                .objects
                .values()
                .filter(|obj| {
                    obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && players.contains(&obj.controller)
                        && matches_filter(&obj.characteristics, filter)
                })
                .count() as i32
        }
        EffectAmount::DevotionTo(color) => {
            // CR 700.5: A player's devotion to [color] is the number of mana symbols
            // of that color among the mana costs of permanents that player controls.
            let controller = resolve_player_target_list(state, &PlayerTarget::Controller, ctx);
            let controller_id = controller.first().copied().unwrap_or(ctx.controller);
            state
                .objects
                .values()
                .filter(|obj| {
                    obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && obj.controller == controller_id
                })
                .map(|obj| {
                    obj.characteristics
                        .mana_cost
                        .as_ref()
                        .map(|mc| {
                            // CR 700.5: basic color pips
                            let base = match color {
                                Color::White => mc.white as i32,
                                Color::Blue => mc.blue as i32,
                                Color::Black => mc.black as i32,
                                Color::Red => mc.red as i32,
                                Color::Green => mc.green as i32,
                            };
                            // CR 700.5: hybrid mana symbols count toward each of their colors.
                            // HybridMana::ColorColor(c1, c2) — counts if either color matches.
                            // HybridMana::GenericColor(c) — counts if c matches.
                            let hybrid_count: i32 = mc
                                .hybrid
                                .iter()
                                .map(|h| {
                                    let mc_color = match color {
                                        Color::White => ManaColor::White,
                                        Color::Blue => ManaColor::Blue,
                                        Color::Black => ManaColor::Black,
                                        Color::Red => ManaColor::Red,
                                        Color::Green => ManaColor::Green,
                                    };
                                    match h {
                                        HybridMana::ColorColor(c1, c2) => {
                                            if *c1 == mc_color || *c2 == mc_color {
                                                1
                                            } else {
                                                0
                                            }
                                        }
                                        HybridMana::GenericColor(c) => {
                                            if *c == mc_color {
                                                1
                                            } else {
                                                0
                                            }
                                        }
                                    }
                                })
                                .sum();
                            // CR 700.5: Phyrexian mana symbols count toward their color(s).
                            // PhyrexianMana::Single(c) — counts if c matches.
                            // PhyrexianMana::Hybrid(c1, c2) — counts if either matches.
                            let phyrexian_count: i32 = mc
                                .phyrexian
                                .iter()
                                .map(|p| {
                                    let mc_color = match color {
                                        Color::White => ManaColor::White,
                                        Color::Blue => ManaColor::Blue,
                                        Color::Black => ManaColor::Black,
                                        Color::Red => ManaColor::Red,
                                        Color::Green => ManaColor::Green,
                                    };
                                    match p {
                                        PhyrexianMana::Single(c) => {
                                            if *c == mc_color {
                                                1
                                            } else {
                                                0
                                            }
                                        }
                                        PhyrexianMana::Hybrid(c1, c2) => {
                                            if *c1 == mc_color || *c2 == mc_color {
                                                1
                                            } else {
                                                0
                                            }
                                        }
                                    }
                                })
                                .sum();
                            base + hybrid_count + phyrexian_count
                        })
                        .unwrap_or(0)
                })
                .sum()
        }
        EffectAmount::CounterCount { target, counter } => {
            // Count counters of a specific type on the target permanent.
            let targets = resolve_effect_target_list(state, target, ctx);
            targets
                .into_iter()
                .filter_map(|t| {
                    if let ResolvedTarget::Object(id) = t {
                        state
                            .objects
                            .get(&id)
                            .map(|obj| *obj.counters.get(counter).unwrap_or(&0) as i32)
                    } else {
                        None
                    }
                })
                .next()
                .unwrap_or(0)
        }
        // Reads the count of permanents actually destroyed/exiled by the preceding
        // DestroyAll or ExileAll effect (stored in ctx.last_effect_count).
        EffectAmount::LastEffectCount => ctx.last_effect_count as i32,
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

pub fn make_token(
    spec: &crate::cards::card_definition::TokenSpec,
    controller: PlayerId,
) -> GameObject {
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

    let mut supertypes = OrdSet::new();
    for st in &spec.supertypes {
        supertypes.insert(*st);
    }

    let mut mana_abilities = im::Vector::new();
    for ma in &spec.mana_abilities {
        mana_abilities.push_back(ma.clone());
    }

    // CR 111.10b: propagate non-mana activated abilities (e.g. Food's sacrifice-for-life).
    let activated_abilities: Vec<crate::state::game_object::ActivatedAbility> =
        spec.activated_abilities.clone();

    let characteristics = Characteristics {
        name: spec.name.clone(),
        power: Some(spec.power),
        toughness: Some(spec.toughness),
        supertypes,
        card_types,
        keywords,
        subtypes,
        colors,
        mana_abilities,
        activated_abilities,
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
        goaded_by: im::Vector::new(),
        kicker_times_paid: 0,
        cast_alt_cost: None,
        foretold_turn: 0,
        was_unearthed: false,
        myriad_exile_at_eoc: false,
        decayed_sacrifice_at_eoc: false,
        ring_block_sacrifice_at_eoc: false,
        exiled_by_hideaway: None,
        // CR 701.60b: tokens are not suspected by default.
        encore_sacrifice_at_end_step: false,
        encore_must_attack: None,
        encore_activated_by: None,
        is_plotted: false,
        plotted_turn: 0,
        is_prototyped: false,
        was_bargained: false,
        evidence_collected: false,
        phased_out_indirectly: false,
        phased_out_controller: None,
        creatures_devoured: 0,
        champion_exiled_card: None,
        paired_with: None,
        tribute_was_paid: false,
        // CR 107.3m: Tokens are never cast, so x_value is always 0.
        x_value: 0,
        // CR 702.157a: Tokens are never cast, so squad_count is always 0.
        squad_count: 0,
        offspring_paid: false,
        // CR 702.174a: Tokens are never cast, so gift fields are false/None.
        gift_was_given: false,
        gift_opponent: None,
        // CR 702.171b: Tokens are not saddled by default.
        encoded_cards: im::Vector::new(),
        // CR 702.55b: Tokens have no haunting relationship.
        haunting_target: None,
        // CR 702.151b: Tokens are not reconfigured by default.
        // CR 729.2: Tokens are not part of a merged permanent by default.
        merged_components: im::Vector::new(),
        // CR 712.8a: Tokens and new permanents start untransformed.
        is_transformed: false,
        last_transform_timestamp: 0,
        was_cast_disturbed: false,
        craft_exiled_cards: im::Vector::new(),
        chosen_creature_type: None,
        face_down_as: None,
        loyalty_ability_activated_this_turn: false,
        class_level: 0,
        designations: Designations::default(),
        meld_component: None,
    }
}

// ── Card draw helper ──────────────────────────────────────────────────────────

/// Draw one card for a player (CR 121.1). Returns events.
fn draw_one_card(state: &mut GameState, player: PlayerId) -> Vec<GameEvent> {
    // CR 614.11: Check WouldDraw replacement effects before performing the draw.
    // Shared logic lives in `replacement::check_would_draw_replacement` (MR-M8-07).
    // CR 702.52: Also checks for dredge-eligible cards in the graveyard.
    {
        use crate::rules::replacement::{self, DrawAction};
        match replacement::check_would_draw_replacement(state, player) {
            DrawAction::Proceed => {}
            DrawAction::Skip(event) => return vec![event],
            DrawAction::NeedsChoice(event) => {
                // CR 616.1: Multiple WouldDraw replacements apply — defer the draw.
                return vec![event];
            }
            DrawAction::DredgeAvailable(event) => {
                // CR 702.52: Dredge options available — pause for player choice.
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
                let mut events = vec![GameEvent::CardDrawn {
                    player,
                    new_object_id: new_id,
                }];
                // CR 702.94a: Check if the just-drawn card has miracle and is the first draw.
                if let Some(miracle_event) =
                    crate::rules::miracle::check_miracle_eligible(state, player, new_id)
                {
                    events.push(miracle_event);
                }
                events
            } else {
                vec![]
            }
        }
    }
}

/// Discard `n` cards from a player's hand (first by ObjectId, deterministic).
///
/// CR 702.35a: If a discarded card has `KeywordAbility::Madness`, it is exiled
/// instead of going to the graveyard. The `CardDiscarded` event still fires (per
/// CR ruling: "A card with madness that's discarded counts as having been discarded
/// even though it's put into exile rather than a graveyard"). A `MadnessTrigger` is
/// pushed onto the stack so the owner may cast the card for its madness cost.
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
            // CR 702.35a: Check if the card has Madness before zone change.
            let obj_card_id = state.objects.get(&card_id).and_then(|o| o.card_id.clone());
            let has_madness = state
                .objects
                .get(&card_id)
                .map(|obj| {
                    obj.characteristics
                        .keywords
                        .contains(&KeywordAbility::Madness)
                })
                .unwrap_or(false);

            let destination = if has_madness {
                ZoneId::Exile
            } else {
                ZoneId::Graveyard(player)
            };

            if let Ok((new_id, _)) = state.move_object_to_zone(card_id, destination) {
                // CR ruling: CardDiscarded always fires, even when card goes to exile.
                events.push(GameEvent::CardDiscarded {
                    player,
                    object_id: card_id,
                    new_id,
                });

                if has_madness {
                    // CR 702.35a: Look up the madness cost and queue the trigger via
                    // pending_triggers so flush_pending_triggers properly signals priority.
                    let madness_cost = obj_card_id.as_ref().and_then(|cid| {
                        state.card_registry.get(cid.clone()).and_then(|def| {
                            def.abilities.iter().find_map(|a| {
                                if let crate::cards::card_definition::AbilityDefinition::Madness {
                                    cost,
                                } = a
                                {
                                    Some(cost.clone())
                                } else {
                                    None
                                }
                            })
                        })
                    });
                    state.pending_triggers.push_back(PendingTrigger {
                        source: new_id,
                        ability_index: 0,
                        controller: player,
                        kind: PendingTriggerKind::Madness,
                        triggering_event: None,
                        entering_object_id: None,
                        targeting_stack_id: None,
                        triggering_player: None,
                        exalted_attacker_id: None,
                        defending_player_id: None,
                        madness_exiled_card: Some(new_id),
                        madness_cost,
                        miracle_revealed_card: None,
                        miracle_cost: None,
                        modular_counter_count: None,
                        evolve_entering_creature: None,
                        suspend_card_id: None,
                        hideaway_count: None,
                        partner_with_name: None,
                        ingest_target_player: None,
                        flanking_blocker_id: None,
                        rampage_n: None,
                        provoke_target_creature: None,
                        renown_n: None,
                        poisonous_n: None,
                        poisonous_target_player: None,
                        enlist_enlisted_creature: None,
                        encore_activator: None,
                        echo_cost: None,
                        cumulative_upkeep_cost: None,
                        recover_cost: None,
                        recover_card: None,
                        graft_entering_creature: None,
                        backup_abilities: None,
                        backup_n: None,
                        champion_filter: None,
                        champion_exiled_card: None,
                        soulbond_pair_target: None,
                        squad_count: None,
                        gift_opponent: None,
                        cipher_encoded_card_id: None,
                        cipher_encoded_object_id: None,
                        haunt_source_object_id: None,
                        haunt_source_card_id: None,
                    });
                }
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
    // OR-semantics subtype filter: card must have at least one of the listed subtypes.
    if !filter.has_subtypes.is_empty()
        && !filter
            .has_subtypes
            .iter()
            .any(|st| chars.subtypes.contains(st))
    {
        return false;
    }
    // CR 702.124j: exact-name filter for "search for a card named [name]"
    if let Some(name) = &filter.has_name {
        if &chars.name != name {
            return false;
        }
    }
    // Mana value (CMC) filters — uses ManaCost::mana_value() (CR 202.3).
    if let Some(max) = filter.max_cmc {
        let mv = chars
            .mana_cost
            .as_ref()
            .map(|c| c.mana_value())
            .unwrap_or(0);
        if mv > max {
            return false;
        }
    }
    if let Some(min) = filter.min_cmc {
        let mv = chars
            .mana_cost
            .as_ref()
            .map(|c| c.mana_value())
            .unwrap_or(0);
        if mv < min {
            return false;
        }
    }
    // OR-semantics card type filter: must have at least one of the listed types.
    if !filter.has_card_types.is_empty()
        && !filter
            .has_card_types
            .iter()
            .any(|ct| chars.card_types.contains(ct))
    {
        return false;
    }
    true
}

// ── Condition checking ────────────────────────────────────────────────────────

pub(crate) fn check_condition(
    state: &GameState,
    condition: &Condition,
    ctx: &EffectContext,
) -> bool {
    match condition {
        Condition::Always => true,
        Condition::ControllerLifeAtLeast(n) => state
            .players
            .get(&ctx.controller)
            .map(|ps| ps.life_total >= *n as i32)
            .unwrap_or(false),
        // CR 702.26b: phased-out permanents are treated as nonexistent.
        Condition::SourceOnBattlefield => state
            .objects
            .get(&ctx.source)
            .map(|obj| obj.zone == ZoneId::Battlefield && obj.is_phased_in())
            .unwrap_or(false),
        Condition::YouControlPermanent(filter) => state.objects.values().any(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.is_phased_in()
                && obj.controller == ctx.controller
                && matches_filter(&obj.characteristics, filter)
        }),
        Condition::OpponentControlsPermanent(filter) => state.objects.values().any(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.is_phased_in()
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
        // CR 701.46a: "if ~ has no [counter] counters on it" — used by Adapt.
        // If the source no longer exists (e.g., removed in response), treat as
        // "no counters" (safe default — the AddCounter effect will silently no-op).
        Condition::SourceHasNoCountersOfType { counter } => state
            .objects
            .get(&ctx.source)
            .map(|obj| obj.counters.get(counter).copied().unwrap_or(0) == 0)
            .unwrap_or(true),
        // CR 702.33d: "if this spell was kicked" — true when kicker_times_paid > 0.
        Condition::WasKicked => ctx.kicker_times_paid > 0,
        // CR 702.96a: "if this spell's overload cost was paid" — true when overloaded.
        Condition::WasOverloaded => ctx.was_overloaded,
        // CR 702.166b: "if this spell was bargained" — true when bargain cost was paid.
        Condition::WasBargained => ctx.was_bargained,
        // CR 702.148a: "if this spell's cleave cost was paid" — true when cleaved.
        Condition::WasCleaved => ctx.was_cleaved,
        // CR 701.59c: "if evidence was collected" — true when collect evidence cost was paid.
        Condition::EvidenceWasCollected => ctx.evidence_collected,
        // CR 702.174b: "if this spell's gift cost was paid" — true when gift cost was paid.
        Condition::GiftWasGiven => ctx.gift_was_given,
        // CR 207.2c (Corrupted ability word): "if an opponent has N or more poison counters."
        // In multiplayer Commander, true if ANY living opponent of the controller has >= N
        // poison counters. Eliminated opponents (has_lost == true) are excluded.
        Condition::OpponentHasPoisonCounters(n) => state
            .players
            .iter()
            .any(|(pid, ps)| *pid != ctx.controller && !ps.has_lost && ps.poison_counters >= *n),
        // CR 309.7: "as long as you've completed a dungeon" — true when dungeons_completed > 0.
        Condition::CompletedADungeon => state
            .players
            .get(&ctx.controller)
            .map(|ps| ps.dungeons_completed > 0)
            .unwrap_or(false),
        // CR 309.7: "if you've completed [specific dungeon]" — checks dungeons_completed_set.
        // Used by Acererak: `!CompletedSpecificDungeon(TombOfAnnihilation)`.
        Condition::CompletedSpecificDungeon(dungeon_id) => state
            .players
            .get(&ctx.controller)
            .map(|ps| ps.dungeons_completed_set.contains(dungeon_id))
            .unwrap_or(false),
        // Logical negation of any condition (CR 603.4: intervening-if can express "haven't").
        // Used by Acererak's ETB: "if you haven't completed Tomb of Annihilation".
        Condition::Not(inner) => !check_condition(state, inner, ctx),
        // CR 701.54c: "if the Ring has tempted you N or more times" — true when ring_level >= n.
        // Used for cards that check the ring level (e.g., Frodo, Sauron's Bane at level 4).
        Condition::RingHasTemptedYou(n) => state
            .players
            .get(&ctx.controller)
            .map(|ps| ps.ring_level >= *n)
            .unwrap_or(false),
        // Logical disjunction: true if either condition holds.
        Condition::Or(a, b) => check_condition(state, a, ctx) || check_condition(state, b, ctx),
        // ── ETB condition variants (PB-2) ────────────────────────────────────
        //
        // CR 614.1c: "enters tapped unless [condition]" — these are evaluated
        // at ETB time with a minimal EffectContext (controller + source = entering object).

        // "unless you control a [Plains/Island/etc.]" — check-lands, castles.
        // True if the controller controls a land with ANY of the listed subtypes.
        Condition::ControlLandWithSubtypes(subtypes) => state.objects.values().any(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.is_phased_in()
                && obj.controller == ctx.controller
                && obj.characteristics.card_types.contains(&CardType::Land)
                && subtypes
                    .iter()
                    .any(|st| obj.characteristics.subtypes.contains(st))
        }),
        // "unless you control N or fewer other lands" — fast-lands.
        // True if the controller controls <= N OTHER lands (excluding source).
        Condition::ControlAtMostNOtherLands(n) => {
            let other_land_count = state
                .objects
                .iter()
                .filter(|(&id, obj)| {
                    id != ctx.source
                        && obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && obj.controller == ctx.controller
                        && obj.characteristics.card_types.contains(&CardType::Land)
                })
                .count();
            other_land_count <= *n as usize
        }
        // "unless you have two or more opponents" — bond-lands.
        // True if the controller has >= 2 living opponents.
        Condition::HaveTwoOrMoreOpponents => {
            let opponent_count = state
                .players
                .iter()
                .filter(|(pid, ps)| **pid != ctx.controller && !ps.has_lost)
                .count();
            opponent_count >= 2
        }
        // "you may reveal a [type] card from your hand" — reveal-lands.
        // Deterministic: auto-reveal if hand contains a card with ANY of the listed subtypes.
        Condition::CanRevealFromHandWithSubtype(subtypes) => {
            let hand_zone = ZoneId::Hand(ctx.controller);
            state.objects.values().any(|obj| {
                obj.zone == hand_zone
                    && subtypes
                        .iter()
                        .any(|st| obj.characteristics.subtypes.contains(st))
            })
        }
        // "unless you control N or more basic lands" — battle-lands.
        // True if the controller controls >= N basic lands on the battlefield.
        Condition::ControlBasicLandsAtLeast(n) => {
            let basic_land_count = state
                .objects
                .values()
                .filter(|obj| {
                    obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && obj.controller == ctx.controller
                        && obj.characteristics.card_types.contains(&CardType::Land)
                        && obj
                            .characteristics
                            .supertypes
                            .contains(&crate::state::types::SuperType::Basic)
                })
                .count();
            basic_land_count >= *n as usize
        }
        // "unless you control N or more other lands" — slow-lands.
        // True if the controller controls >= N OTHER lands (excluding source).
        Condition::ControlAtLeastNOtherLands(n) => {
            let other_land_count = state
                .objects
                .iter()
                .filter(|(&id, obj)| {
                    id != ctx.source
                        && obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && obj.controller == ctx.controller
                        && obj.characteristics.card_types.contains(&CardType::Land)
                })
                .count();
            other_land_count >= *n as usize
        }
        // "unless you control N or more other [subtype]s" — Mystic Sanctuary, Witch's Cottage.
        // True if the controller controls >= N OTHER lands with the given subtype (excluding source).
        Condition::ControlAtLeastNOtherLandsWithSubtype { count, subtype } => {
            let matching_count = state
                .objects
                .iter()
                .filter(|(&id, obj)| {
                    id != ctx.source
                        && obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && obj.controller == ctx.controller
                        && obj.characteristics.subtypes.contains(subtype)
                })
                .count();
            matching_count >= *count as usize
        }
        // "unless you control a legendary creature" — Minas Tirith.
        Condition::ControlLegendaryCreature => state.objects.values().any(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.is_phased_in()
                && obj.controller == ctx.controller
                && obj.characteristics.card_types.contains(&CardType::Creature)
                && obj
                    .characteristics
                    .supertypes
                    .contains(&SuperType::Legendary)
        }),
        // "unless you control a [creature subtype]" — Temple of the Dragon Queen.
        Condition::ControlCreatureWithSubtype(subtype) => state.objects.values().any(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.is_phased_in()
                && obj.controller == ctx.controller
                && obj.characteristics.card_types.contains(&CardType::Creature)
                && obj.characteristics.subtypes.contains(subtype)
        }),
        // CR 702.131c: the city's blessing is permanent once gained.
        Condition::HasCitysBlessing => state
            .players
            .get(&ctx.controller)
            .map(|p| p.has_citys_blessing)
            .unwrap_or(false),
        // CR 500.8: True when not in an extra combat phase (first combat phase of turn).
        Condition::IsFirstCombatPhase => !state.turn.in_extra_combat,
    }
}

// ── ForEach collection ────────────────────────────────────────────────────────

fn collect_for_each(state: &GameState, over: &ForEachTarget, ctx: &EffectContext) -> Vec<ObjectId> {
    match over {
        // CR 702.26b: phased-out permanents are treated as nonexistent.
        ForEachTarget::EachCreature => state
            .objects
            .iter()
            .filter(|(_, obj)| {
                obj.zone == ZoneId::Battlefield
                    && obj.is_phased_in()
                    && obj.characteristics.card_types.contains(&CardType::Creature)
            })
            .map(|(&id, _)| id)
            .collect(),
        ForEachTarget::EachCreatureYouControl => state
            .objects
            .iter()
            .filter(|(_, obj)| {
                obj.zone == ZoneId::Battlefield
                    && obj.is_phased_in()
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
                    && obj.is_phased_in()
                    && obj.controller != ctx.controller
                    && obj.characteristics.card_types.contains(&CardType::Creature)
            })
            .map(|(&id, _)| id)
            .collect(),
        // CR 702.96a: TargetFilter.controller is applied here to enforce "you don't control"
        // constraints from overload cards and similar effects.
        ForEachTarget::EachPermanentMatching(filter) => state
            .objects
            .iter()
            .filter(|(_, obj)| {
                if obj.zone != ZoneId::Battlefield {
                    return false;
                }
                if !matches_filter(&obj.characteristics, filter) {
                    return false;
                }
                match filter.controller {
                    TargetController::Any => true,
                    TargetController::You => obj.controller == ctx.controller,
                    TargetController::Opponent => obj.controller != ctx.controller,
                }
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

        // CR 702.91a: All attacking creatures except the source (battle cry source).
        // Queries combat.attackers at resolution time, excludes ctx.source so the
        // battle cry creature itself does not receive the bonus.
        ForEachTarget::EachOtherAttackingCreature => {
            if let Some(ref combat) = state.combat {
                combat
                    .attackers
                    .keys()
                    .filter(|&&id| id != ctx.source)
                    .copied()
                    .collect()
            } else {
                vec![]
            }
        }
        // All attacking creatures including the source.
        // Used by 'untap all attacking creatures' effects (Karlach, Hellkite Charger).
        ForEachTarget::EachAttackingCreature => {
            if let Some(ref combat) = state.combat {
                combat.attackers.keys().copied().collect()
            } else {
                vec![]
            }
        }
    }
}
