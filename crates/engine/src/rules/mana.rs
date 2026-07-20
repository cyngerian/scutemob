//! Mana ability activation (CR 605).
//!
//! Mana abilities are activated abilities that produce mana and don't target.
//! They do not use the stack — they activate and resolve immediately.
//! They can be activated any time a player has priority (CR 605.3b).
//!
//! SR-34: an activation cost may also include a mana component (`ManaAbility::mana_cost`,
//! e.g. a Signet's `{1}`) and/or a life component (`ManaAbility::life_cost`, e.g. a horizon
//! land's "Pay 1 life") in addition to `{T}` / sacrifice-self. A cost component that needs a
//! caller-supplied `ObjectId` (discard a card, sacrifice *another* permanent, remove a
//! counter) has no channel through `Command::TapForMana` and is out of scope — those
//! abilities stay stack-using activated abilities.
//!
//! SR-36: production may also be dynamic (`ManaAbility::scaled_amount`, e.g. Gaea's
//! Cradle's "for each creature you control") rather than the fixed `produces` count.
use super::events::{CombatDamageTarget, GameEvent};
use crate::cards::card_definition::{
    AbilityDefinition, Effect, ManaSourceFilter, TriggerCondition,
};
use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::player::PlayerId;
use crate::state::replacement_effect::{
    ChosenColorRef, ReplacementManaSourceFilter, ReplacementModification, ReplacementTrigger,
};
use crate::state::stubs::{GameRestriction, PendingTrigger, PendingTriggerKind};
use crate::state::types::{CardType, KeywordAbility, ManaColor};
use crate::state::zone::ZoneId;
use crate::state::GameState;
/// Handle a TapForMana command: activate a mana ability by tapping a permanent.
///
/// Validates priority, battlefield presence, controller, ability existence,
/// and tap status. Taps the permanent (if required), adds mana to the pool.
///
/// Per CR 605.5, activating a mana ability is a special action. The player
/// retains priority and `players_passed` is not reset.
pub fn handle_tap_for_mana(
    state: &mut GameState,
    player: PlayerId,
    source: ObjectId,
    ability_index: usize,
    chosen_color: Option<ManaColor>,
    hybrid_choices: Vec<crate::state::game_object::HybridManaPayment>,
    phyrexian_life_payments: Vec<bool>,
) -> Result<Vec<GameEvent>, GameStateError> {
    // 1. Validate player has priority (CR 605.3b).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // 1b. PB-18 review Finding 2: Check restrictions that block mana ability activation.
    //
    // CR 605.3: "Activating an activated mana ability follows the rules for activating
    // any other activated ability." Therefore Stony Silence / Collector Ouphe block
    // mana abilities of artifacts (per ruling: "including mana abilities").
    // Grand Abolisher prevents opponents from activating mana abilities of artifacts,
    // creatures, or enchantments during the controller's turn.
    //
    // Per Finding 3 (zone scope): only artifacts ON THE BATTLEFIELD are affected.
    // "Stony Silence's ability affects only artifacts on the battlefield."
    {
        let active_player = state.turn.active_player;
        // Determine source types (battlefield-only check per Finding 3).
        let source_zone = state.objects.get(&source).map(|o| o.zone);
        let source_on_bf = matches!(source_zone, Some(ZoneId::Battlefield));
        let source_is_artifact = source_on_bf
            && crate::rules::layers::calculate_characteristics(state, source)
                .map(|chars| chars.card_types.contains(&CardType::Artifact))
                .unwrap_or(false);
        let source_is_restricted_type = source_on_bf
            && crate::rules::layers::calculate_characteristics(state, source)
                .map(|chars| {
                    chars.card_types.contains(&CardType::Artifact)
                        || chars.card_types.contains(&CardType::Creature)
                        || chars.card_types.contains(&CardType::Enchantment)
                })
                .unwrap_or(false);
        for restriction in state.restrictions.iter() {
            // Skip restrictions whose source is no longer on the battlefield.
            let restriction_source_on_bf = state
                .objects
                .get(&restriction.source)
                .map(|o| matches!(o.zone, ZoneId::Battlefield))
                .unwrap_or(false);
            if !restriction_source_on_bf {
                continue;
            }
            let controller = restriction.controller;
            #[allow(clippy::collapsible_match)]
            match &restriction.restriction {
                // Collector Ouphe / Stony Silence: blocks ALL activated abilities of artifacts
                // including mana abilities (CR 605.3 + ruling).
                GameRestriction::ArtifactAbilitiesCantBeActivated => {
                    if source_is_artifact {
                        return Err(GameStateError::InvalidCommand(
                            "restriction: activated abilities of artifacts can't be activated, \
                             including mana abilities (CR 605.3, Stony Silence)"
                                .into(),
                        ));
                    }
                }
                // Grand Abolisher / Myrel: opponents can't activate mana abilities of
                // artifact/creature/enchantment permanents during controller's turn.
                GameRestriction::OpponentsCantCastOrActivateDuringYourTurn => {
                    if active_player == controller
                        && player != controller
                        && source_is_restricted_type
                    {
                        return Err(GameStateError::InvalidCommand(
                            "restriction: opponents can't activate abilities of artifacts, \
                             creatures, or enchantments during your turn, including mana abilities \
                             (CR 605.3, Grand Abolisher)"
                                .into(),
                        ));
                    }
                }
                _ => {}
            }
        }
    }
    // 2. Fetch a clone of the source object to avoid borrow conflicts.
    let obj = state.object(source)?.clone();
    // PB-EF8: the ability-fetch block (formerly step 5) must run BEFORE the
    // zone/controller legality check, because the from-hand branch below keys on
    // `ability.exile_self_from_hand`, which is not known until the ability is fetched.
    // Nothing between the old step 2 and old step 5 mutates state (steps 3-4 were pure
    // validation), so this reorder is behavior-preserving for the battlefield path.
    //
    // Fetch the mana ability via layer-resolved characteristics.
    // CR 613.1f: Use calc'd chars so granted abilities (Cryptolith Rite, Chromatic Lantern)
    // and ability-removal (Humility) both apply. W3-LC audit fix.
    // SR-14 IMPOSSIBLE: `source` was validated present via `state.object(source)?`
    // above with no intervening zone change, so calc cannot be None here. This holds
    // for a hand source too — `expect_characteristics` works off-battlefield (a hand
    // card's `mana_abilities` are its base enriched list, since no layers apply
    // off-battlefield), exactly as the existing graveyard-activated path relies on.
    let resolved_chars = crate::rules::layers::expect_characteristics(state, source);
    let ability = resolved_chars
        .mana_abilities
        .get(ability_index)
        .ok_or(GameStateError::InvalidAbilityIndex {
            object_id: source,
            index: ability_index,
        })?
        .clone();
    // 2b. PB-EF12 (CR 605.3b / CR 106.1b): a mana ability resolves immediately and never
    //     uses the stack, so any colour choice for `any_color: true` production must be
    //     supplied on THIS command, not deferred to a stack-based choice. Pure validation,
    //     before any mutation. `Colorless` is a mana TYPE, not a colour (CR 106.1b) — it is
    //     outside the legal option set for "add one mana of any color" (CR 111.10a) and is
    //     rejected, same as omitting the choice entirely.
    let resolved_color: Option<ManaColor> = if ability.any_color {
        match chosen_color {
            Some(ManaColor::Colorless) => {
                return Err(GameStateError::InvalidCommand(
                    "colorless is not a color; an any-color mana ability must choose one of \
                     White, Blue, Black, Red, or Green (CR 106.1b, CR 111.10a)"
                        .into(),
                ));
            }
            Some(c) => Some(c),
            None => {
                return Err(GameStateError::InvalidCommand(
                    "an any-color mana ability requires a chosen_color on TapForMana — mana \
                     abilities never use the stack (CR 605.3b), so the color choice is made \
                     at activation, not deferred"
                        .into(),
                ));
            }
        }
    } else {
        if chosen_color.is_some() {
            return Err(GameStateError::InvalidCommand(
                "chosen_color was supplied for a fixed-colour mana ability".into(),
            ));
        }
        None
    };
    // 3-4. Zone/controller legality (PB-EF8: branches on the ability, not a fixed rule).
    if ability.exile_self_from_hand {
        // CR 602.1/602.2/605.1a: a from-hand mana ability (Spirit Guides). The source
        // must be in the activating player's hand (mirrors the Channel check in
        // handle_activate_ability, abilities.rs — a hand card's controller is its
        // owner, so check owner).
        if obj.zone != ZoneId::Hand(player) {
            return Err(GameStateError::InvalidCommand(
                "from-hand mana ability can only be activated from hand (CR 602.2, 605.1a)".into(),
            ));
        }
        if obj.owner != player {
            return Err(GameStateError::InvalidCommand(
                "you can only activate a from-hand mana ability on a card you own".into(),
            ));
        }
    } else {
        // 3. Validate source is on the battlefield.
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::ObjectNotOnBattlefield(source));
        }
        // 4. Validate player controls the source.
        if obj.controller != player {
            return Err(GameStateError::NotController {
                player,
                object_id: source,
            });
        }
    }
    let mut events = Vec::new();
    // PB-RS2 (CR 107.4e/107.4f via CR 605.1a/602.2b): flatten hybrid/Phyrexian choices
    // ONCE, here, before any legality check — a mana ability's activation cost is its
    // analog to a spell's mana cost/an activated ability's activation cost, and it must
    // go through the same flatten `casting.rs` uses for spells. Pre-PB-RS2, this legality
    // check and the payment site below both read `ability.mana_cost` RAW, so a pure
    // `{B/R}` pip (mana_value()==1, passing the `> 0` gate) was charged as an all-zero
    // cost — every filter land's `{B/R},{T}: ...` ability was a free "Add two mana"
    // (OOS-RS-2, the live-wrong roster this PB exists to fix). Flattening once and
    // reusing the result at the payment site (step 6b) avoids the two sites drifting,
    // which is structurally how that bug arose.
    let (flat_mana_cost, phyrexian_life) = match &ability.mana_cost {
        Some(mana_cost) if !mana_cost.hybrid.is_empty() || !mana_cost.phyrexian.is_empty() => {
            let (flat, life) = crate::rules::casting::flatten_hybrid_phyrexian(
                mana_cost,
                &hybrid_choices,
                &phyrexian_life_payments,
            )?;
            (Some(flat), life)
        }
        Some(mana_cost) => (Some(mana_cost.clone()), 0),
        None => (None, 0),
    };
    // 5b. SR-34: cost legality check (CR 118.3, 119.4). Pure validation, no mutation —
    //     an unaffordable Signet/horizon-land activation touches nothing (CR 732 is free
    //     here regardless, since `process_command` takes `GameState` by value and only
    //     returns it on `Ok`, but validating first keeps the transaction visibly clean).
    if let Some(ref flat_cost) = flat_mana_cost {
        if flat_cost.mana_value() > 0 {
            let player_state = state.player(player)?;
            if !player_state.mana_pool.can_spend(flat_cost, None) {
                return Err(GameStateError::InsufficientMana);
            }
        }
    }
    // CR 119.4b: players can always pay 0 life, no matter what their life total is — so
    // the check must short-circuit on `combined_life_cost > 0` rather than reading `>=`
    // unguarded. CR 119.4 + CR 601.2h/602.2b (PB-RS2): check the COMBINED total of
    // `ability.life_cost` and a Phyrexian pip paid with life against life_total ONCE, not
    // each independently — the cost's components may be paid in any order, and CR 119.4
    // gates "the amount of the payment" for the whole cost. A player at 3 life activating
    // a "Pay 2 life" ability with a `{G/P}` paid with life may not pay a combined 4.
    let combined_life_cost = ability.life_cost + phyrexian_life;
    if combined_life_cost > 0 {
        let player_state = state.player(player)?;
        if player_state.life_total < combined_life_cost as i32 {
            return Err(GameStateError::InsufficientLife {
                player,
                required: combined_life_cost,
                actual: player_state.life_total,
            });
        }
    }
    // 5b2. PB-OS11 (CR 602.2c / CR 118.3): remove-counter cost legality check. Pure
    //     validation, before any mutation — mirrors the mana_cost / life_cost checks
    //     above (an unaffordable Workhorse/Gemstone Array activation touches nothing).
    if let Some((ref counter, count)) = ability.remove_counter {
        let current = state
            .objects
            .get(&source)
            .and_then(|o| o.counters.get(counter).copied())
            .unwrap_or(0);
        if current < count {
            return Err(GameStateError::InvalidCommand(format!(
                "mana ability requires removing {count} {counter:?} counter(s) but only \
                 {current} present (CR 118.3)"
            )));
        }
    }
    // 5c. SR-37 / SF-10 (CR 602.5b): "Activate only if [condition]". CR 605.1a keeps a
    //     conditioned ability a mana ability, so `mana_ability_lowering` still lowers it —
    //     but the restriction must be enforced here, exactly as `handle_activate_ability`
    //     enforces it for a stack-using ability. Pure validation, before any mutation:
    //     Tainted Field's `{T}: Add {W}` arm must fail when its controller has no Swamp.
    //     `activation_condition` is `None` for the overwhelming majority, so this branch is
    //     a no-op on virtually every activation.
    if let Some(condition) = &ability.activation_condition {
        let ctx = crate::effects::EffectContext::new(player, source, vec![]);
        if !crate::effects::check_condition(state, condition, &ctx) {
            return Err(GameStateError::InvalidCommand(
                "mana ability activation condition not met (CR 602.5b)".into(),
            ));
        }
    }
    // 6. If the ability requires tapping: validate not already tapped, then tap.
    if ability.requires_tap {
        if obj.status.tapped {
            return Err(GameStateError::PermanentAlreadyTapped(source));
        }
        // CR 302.6 / CR 702.10: Summoning sickness prevents using {T} mana abilities
        // on creatures unless they have haste.
        // CR 613.1d/613.1f: Use layer-resolved types and keywords so animated
        // permanents (e.g., Nissa-animated lands) respect summoning sickness and
        // layer-granted haste (e.g., Fervor) is recognized.
        // SR-14 IMPOSSIBLE: `source` still present (validated above, tapped only later).
        let tap_chars = crate::rules::layers::expect_characteristics(state, source);
        if tap_chars.card_types.contains(&CardType::Creature)
            && obj.has_summoning_sickness
            && !tap_chars.keywords.contains(&KeywordAbility::Haste)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "object {:?} has summoning sickness and cannot tap for mana (no haste)",
                source
            )));
        }
        let obj_mut = state.object_mut(source)?;
        obj_mut.status.tapped = true;
        events.push(GameEvent::PermanentTapped {
            player,
            object_id: source,
        });
    }
    // 6b. SR-34: pay the mana and life components of the activation cost (CR 601.2h:
    //     costs in this group may be paid "in any order" — none of tap/mana/life/
    //     sacrifice-self's legality depends on another's result). This sits BEFORE the
    //     SR-28 snapshot below and BEFORE the sacrifice-self cost (step 7): neither
    //     mutation touches the source's characteristics or zone, so the snapshot is
    //     byte-identical on either side of this step, and placing it here keeps the
    //     snapshot's boundary — "cost components that cannot move the source, then the
    //     one that can" — true by construction rather than incidentally, for the next
    //     cost component that DOES move an object (e.g. Krark-Clan Ironworks, out of
    //     SR-34 scope — see SF-9 in the findings doc).
    if let Some(ref flat_cost) = flat_mana_cost {
        if flat_cost.mana_value() > 0 {
            let player_state = state.player_mut(player)?;
            player_state.mana_pool.spend(flat_cost, None);
            events.push(GameEvent::ManaCostPaid {
                player,
                // Emit the ORIGINAL (unflattened) cost for event consumers — mirrors
                // casting.rs's ManaCostPaid emission, which carries hybrid/Phyrexian
                // pip info rather than the flattened shape.
                cost: ability
                    .mana_cost
                    .clone()
                    .expect("flat_mana_cost is Some only when ability.mana_cost is Some"),
            });
        }
    }
    if ability.life_cost > 0 {
        let player_state = state.player_mut(player)?;
        player_state.life_total -= ability.life_cost as i32;
        events.push(GameEvent::LifeLost {
            player,
            amount: ability.life_cost,
        });
    }
    // CR 107.4f (PB-RS2): pay life for a Phyrexian pip paid with life. Legality
    // (including the combined check with `ability.life_cost`) was already validated
    // above, before any mutation.
    if phyrexian_life > 0 {
        let player_state = state.player_mut(player)?;
        player_state.life_total -= phyrexian_life as i32;
        events.push(GameEvent::LifeLost {
            player,
            amount: phyrexian_life,
        });
    }
    // 6d. PB-OS11 (CR 602.2c / CR 118.3): pay a remove-counter cost for a mana
    //     ability (Workhorse "Remove a +1/+1 counter: Add {C}"; also Gemstone
    //     Array / Druids' Repository's "Remove a charge counter" — PB-OS11
    //     backfill). Self-referential; no zone move — the source stays wherever it
    //     was (battlefield). Legality already validated at step 5b2, before any
    //     mutation.
    if let Some((ref counter, count)) = ability.remove_counter {
        let obj_mut = state.object_mut(source)?;
        let current = obj_mut.counters.get(counter).copied().unwrap_or(0);
        let new_count = current.saturating_sub(count);
        if new_count == 0 {
            obj_mut.counters.remove(counter);
        } else {
            obj_mut.counters.insert(counter.clone(), new_count);
        }
        events.push(GameEvent::CounterRemoved {
            object_id: source,
            counter: counter.clone(),
            count,
        });
    }
    // 6c. SR-36 (CR 605.1a): resolve a dynamic mana amount (Gaea's Cradle, Elvish
    //     Archdruid, Priest of Titania, Marwyn the Nurturer, Circle of Dreams Druid,
    //     Cabal Coffers, Cabal Stronghold, Crypt of Agadeem). `resolve_amount`'s
    //     `PermanentCount` arm (`effects/mod.rs`) filters on zone/phased-in only, never
    //     on tapped status, so resolving after the {T} tap in step 6 is safe even for the
    //     creature sources that count creatures (they count themselves too, tapped or
    //     not). Must run BEFORE step 7's sacrifice cost: Marwyn's count is
    //     `EffectAmount::PowerOf(EffectTarget::Source)`, which needs `source` to still be
    //     a live ObjectId (CR 400.7) — none of the current roster combines a scaled
    //     amount with `sacrifice_self`, but this ordering keeps that combination correct
    //     if one is ever authored.
    let resolved_scaled_amount: Option<u32> = ability.scaled_amount.as_ref().map(|amt| {
        let ctx = crate::effects::EffectContext::new(player, source, vec![]);
        crate::effects::resolve_amount(state, amt, &ctx).max(0) as u32
    });
    // SR-28 (CR 106.12a / CR 106.12b): Snapshot the source's layer-resolved
    // characteristics BEFORE the sacrifice cost step below. A {T}+Sacrifice mana
    // source (Treasure — CR 111.10a; Crystal Vein; Dwarven Ruins) is a dead ObjectId
    // (CR 400.7) by the time the mana-production replacement filter (step 7b) and the
    // "tapped for mana" trigger filter (step 10) run. Per CR 106.12a/106.12b those
    // filters must still see the source as it last existed on the battlefield, so both
    // read this snapshot rather than live state. The snapshot is taken after the {T}
    // tap (step 6) — tapping does not change type/subtype/color — and before any zone
    // change, so it is the source's last-known-information (CR 603.10a semantics).
    // `calculate_characteristics` returns `None` only if the object is absent, which
    // cannot happen here: `source` was validated present at step 3 with no intervening
    // zone change. Kept as `Option` so `mana_source_matches` / the replacement filter
    // fizzle cleanly (rather than panic) if a future caller path ever taps a gone source.
    let source_pre_cost_chars = crate::rules::layers::calculate_characteristics(state, source);
    // 7. Pay sacrifice cost if required (CR 111.10a: Treasure tokens).
    //    Sacrifice is a cost paid before mana is produced (CR 602.2c).
    //    After the zone move, `source` is a dead ObjectId (CR 400.7).
    if ability.sacrifice_self {
        let (
            is_creature,
            owner,
            pre_death_controller,
            pre_death_counters,
            pre_death_power_mana,
            mana_sac_pre_chars,
        ) = {
            let obj = state.object(source)?;
            // CR 613.1d: Use layer-resolved types for sacrifice creature check
            // (animated artifacts/lands are creatures per layer 4).
            // `pre_chars_opt` is threaded into the death event as an
            // Option<Characteristics> LKI snapshot (CR 603.10a), so it stays an Option.
            let pre_chars_opt = crate::rules::layers::calculate_characteristics(state, source);
            // SR-14 IMPOSSIBLE: `source` validated present via `state.object(source)?`
            // just above; the sacrifice move has not happened yet, so calc is not None.
            let sac_chars = crate::rules::layers::expect_characteristics(state, source);
            let lki_power = sac_chars.power.or(obj.characteristics.power);
            (
                sac_chars.card_types.contains(&CardType::Creature),
                obj.owner,
                obj.controller,
                obj.counters.clone(),
                lki_power,
                pre_chars_opt,
            )
        };
        let (new_id, _) = state.move_object_to_zone(source, ZoneId::Graveyard(owner))?;
        if is_creature {
            events.push(GameEvent::CreatureDied {
                object_id: source,
                new_grave_id: new_id,
                controller: pre_death_controller,
                pre_death_counters,
                pre_death_power: pre_death_power_mana,
                pre_death_characteristics: mana_sac_pre_chars,
            });
        } else {
            events.push(GameEvent::PermanentDestroyed {
                object_id: source,
                new_grave_id: new_id,
                // CR 603.10a: pass LKI counters for WhenLeavesBattlefield triggers.
                pre_lba_counters: pre_death_counters.clone(),
                // CR 603.10a: LKI power for SourcePowerAtLastKnownInformation.
                pre_lba_power: pre_death_power_mana,
            });
        }
    }
    // 7c. PB-EF8 (CR 118 cost + CR 400.7): exile the source card from hand as the
    //     activation cost. After the move `source` is a dead ObjectId; the card is a new
    //     object in exile. This is the ability's exhaustion mechanism (it cannot be
    //     activated twice — the card is no longer in hand). Mutually exclusive with
    //     `sacrifice_self` (no card has both).
    if ability.exile_self_from_hand {
        let (new_exile_id, _) = state.move_object_to_zone(source, ZoneId::Exile)?;
        events.push(GameEvent::ObjectExiled {
            player,
            object_id: source,
            new_exile_id,
            pre_lba_counters: imbl::OrdMap::new(), // hand card: no battlefield counters
            pre_lba_power: None,                   // not a leaves-battlefield event; no LKI power
        });
    }
    // 7b. Apply mana-production replacement effects (CR 106.12b).
    //     Only applies to mana abilities with {T} in cost (CR 106.12).
    //     Returns (multiplier, additions) where additions is a list of (color, amount)
    //     to append to the mana pool after multiplication (CR 106.6a).
    let (mana_multiplier, mana_additions) = if ability.requires_tap {
        // Compute base mana preview for color-filter checks in apply_mana_production_replacements.
        // SR-36: a scaled ability's `produces` amount is a `1`-per-colour marker, not the
        // real count (see `ManaAbility::scaled_amount`'s doc comment) — substitute the
        // resolved amount so a multiplier replacement (Nyxbloom Ancient) multiplies the
        // real count, not the marker.
        let mut base_preview: Vec<(ManaColor, u32)> = Vec::new();
        if ability.any_color {
            // PB-EF12: preview the chosen colour (not Colorless) so a colour-filter mana
            // replacement (e.g. Caged Sun naming a colour) matches the real choice.
            let color =
                resolved_color.expect("validated above: any_color ability requires Some(c)");
            base_preview.push((color, 1));
        } else {
            for (color, amount) in &ability.produces {
                let amount = resolved_scaled_amount.unwrap_or(*amount);
                base_preview.push((*color, amount));
            }
        }
        apply_mana_production_replacements(state, player, &base_preview, &source_pre_cost_chars)
    } else {
        (1u32, Vec::new())
    };
    // 8. Add produced mana to the player's pool (multiplied by replacement effects).
    //    CR 111.10a: `any_color` produces 1 mana of any color, chosen at activation
    //    (PB-EF12; validated + resolved above as `resolved_color`).
    let mut mana_produced: Vec<(ManaColor, u32)> = Vec::new();
    if ability.any_color {
        // CR 111.10a: "Add one mana of any color."
        let color = resolved_color.expect("validated above: any_color ability requires Some(c)");
        let amount = mana_multiplier;
        let player_state = state.player_mut(player)?;
        player_state.mana_pool.add(color, amount);
        events.push(GameEvent::ManaAdded {
            player,
            color,
            amount,
            source: Some(source),
        });
        mana_produced.push((color, amount));
    } else {
        let player_state = state.player_mut(player)?;
        for (color, base_amount) in &ability.produces {
            // SR-36: substitute the resolved dynamic amount for the `1`-per-colour marker.
            let base_amount = resolved_scaled_amount.unwrap_or(*base_amount);
            let amount = base_amount * mana_multiplier;
            player_state.mana_pool.add(*color, amount);
            events.push(GameEvent::ManaAdded {
                player,
                color: *color,
                amount,
                source: Some(source),
            });
            mana_produced.push((*color, amount));
        }
    }
    // 8b. Apply additive mana additions (CR 106.6a — e.g., Caged Sun / Gauntlet of Power).
    //     These are added to the pool after the base mana (and multiplier) has been applied.
    //     Each entry is one mana of the chosen color (replacement source's chosen_color).
    for (add_color, add_amount) in &mana_additions {
        if *add_amount > 0 {
            let player_state = state.player_mut(player)?;
            player_state.mana_pool.add(*add_color, *add_amount);
            events.push(GameEvent::ManaAdded {
                player,
                color: *add_color,
                amount: *add_amount,
                source: Some(source),
            });
            mana_produced.push((*add_color, *add_amount));
        }
    }
    // 9. Pain land damage: deal damage to controller as part of the mana ability.
    //    CR 605: this is part of the mana ability resolution, not a separate trigger.
    if ability.damage_to_controller > 0 {
        let player_state = state.player_mut(player)?;
        player_state.life_total -= ability.damage_to_controller as i32;
        events.push(GameEvent::DamageDealt {
            source,
            target: CombatDamageTarget::Player(player),
            amount: ability.damage_to_controller,
        });
    }
    // 10. Fire triggered mana abilities (CR 605.4a / CR 106.12a).
    //     Only fires for tap-cost mana abilities (CR 106.12: "tap for mana").
    //     Triggered mana abilities (no target) resolve immediately.
    //     Normal triggered abilities (has targets, e.g., Forbidden Orchard) go on the stack.
    if ability.requires_tap {
        fire_mana_triggered_abilities(
            state,
            player,
            source,
            &mana_produced,
            &source_pre_cost_chars,
            &mut events,
        );
    }
    // 11. Player retains priority. players_passed is unchanged.
    //    (CR 605.5: mana abilities are special actions; they do not reset priority.)
    Ok(events)
}
/// CR 106.12b / CR 106.6a: Check for mana production replacement effects.
///
/// Returns `(multiplier, additions)` where:
/// - `multiplier`: product of all `MultiplyMana` replacements active for this player
///   (Multiple Nyxbloom Ancients: 3 * 3 = 9x. Multiple Mana Reflections: 2 * 2 = 4x.)
/// - `additions`: list of `(ManaColor, amount)` to add to the pool (CR 106.6a additivity).
///   Used by Caged Sun / Gauntlet of Power ("add an additional one mana of that color").
///
/// `base_mana` is the mana the ability would produce before replacements (color-filter check).
/// `source_pre_cost_chars` is the tapped source's layer-resolved characteristics snapshotted
/// BEFORE the sacrifice cost (SR-28) — read by source_filter checks so a {T}+Sacrifice
/// land (now a dead ObjectId, CR 400.7) is still recognized per CR 106.12b.
fn apply_mana_production_replacements(
    state: &GameState,
    player: PlayerId,
    base_mana: &[(ManaColor, u32)],
    source_pre_cost_chars: &Option<crate::state::game_object::Characteristics>,
) -> (u32, Vec<(ManaColor, u32)>) {
    let mut multiplier = 1u32;
    let mut additions: Vec<(ManaColor, u32)> = Vec::new();
    for effect in state.replacement_effects.iter() {
        if let ReplacementTrigger::ManaWouldBeProduced {
            controller,
            color_filter,
            source_filter,
        } = &effect.trigger
        {
            if *controller != player {
                continue;
            }
            // Check source_filter: does this replacement apply to the tapped permanent?
            if let Some(sf) = source_filter {
                let passes_source_filter = match sf {
                    ReplacementManaSourceFilter::Any => true,
                    // SR-28 (CR 106.12b): read the pre-cost snapshot, not live state. A
                    // {T}+Sacrifice land (Dwarven Ruins, Crystal Vein) is already a dead
                    // ObjectId (CR 400.7) here — the previous live `state.objects.get`
                    // lookup returned `None` and silently dropped the replacement (e.g.
                    // Caged Sun missed its +1). The snapshot is `Some` for any real tap.
                    ReplacementManaSourceFilter::AnyLand => source_pre_cost_chars
                        .as_ref()
                        .map(|c| c.card_types.contains(&crate::state::types::CardType::Land))
                        .unwrap_or(false),
                };
                if !passes_source_filter {
                    continue;
                }
            }
            // Check color_filter: does this replacement only fire for a specific color?
            if let Some(cf) = color_filter {
                let required_color = match cf {
                    ChosenColorRef::SelfChosen => {
                        // Read chosen_color from the replacement's source (Caged Sun etc.),
                        // not from the tapped land.
                        effect
                            .source
                            .and_then(|sid| state.objects.get(&sid))
                            .and_then(|o| o.chosen_color)
                    }
                    ChosenColorRef::Fixed(c) => Some(*c),
                };
                // Check if any of the base_mana produced matches the required color.
                let mana_color_for_comparison = required_color.map(|c| match c {
                    crate::state::types::Color::White => ManaColor::White,
                    crate::state::types::Color::Blue => ManaColor::Blue,
                    crate::state::types::Color::Black => ManaColor::Black,
                    crate::state::types::Color::Red => ManaColor::Red,
                    crate::state::types::Color::Green => ManaColor::Green,
                });
                let color_matches = mana_color_for_comparison
                    .map(|mc| base_mana.iter().any(|(bc, amt)| *bc == mc && *amt > 0))
                    .unwrap_or(false);
                if !color_matches {
                    continue;
                }
                // This filter passed — apply modification.
                // Skip inactive sources (source no longer on battlefield).
                let source_on_bf = effect
                    .source
                    .map(|src| {
                        state
                            .objects
                            .get(&src)
                            .map(|o| matches!(o.zone, ZoneId::Battlefield))
                            .unwrap_or(false)
                    })
                    .unwrap_or(false);
                if !source_on_bf {
                    continue;
                }
                match &effect.modification {
                    ReplacementModification::MultiplyMana(n) => {
                        multiplier = multiplier.saturating_mul(*n);
                    }
                    ReplacementModification::AddOneManaOfChosenColor => {
                        // Add one mana of the chosen color (from the replacement source).
                        // CR 106.6a: "additional one mana of that color" per trigger event.
                        if let Some(mc) = mana_color_for_comparison {
                            additions.push((mc, 1));
                        }
                    }
                    _ => {}
                }
            } else {
                // No color filter — unconditional replacement (Mana Reflection / Nyxbloom).
                let source_on_bf = effect
                    .source
                    .map(|src| {
                        state
                            .objects
                            .get(&src)
                            .map(|o| matches!(o.zone, ZoneId::Battlefield))
                            .unwrap_or(false)
                    })
                    .unwrap_or(false);
                if source_on_bf {
                    if let ReplacementModification::MultiplyMana(n) = &effect.modification {
                        multiplier = multiplier.saturating_mul(*n);
                    }
                }
            }
        }
    }
    (multiplier, additions)
}
/// CR 605.4a / CR 106.12a: Fire triggered abilities that trigger from tapping a permanent
/// for mana. Called after mana is added to the pool.
///
/// - Triggered mana abilities (no targets, produces mana) resolve immediately per CR 605.4a.
/// - Triggered abilities with targets (e.g., Forbidden Orchard) go on the stack per CR 605.5a.
///
/// The `source` is the permanent that was tapped for mana.
/// `mana_produced` is the list of (color, amount) pairs the ability produced (post-multiplier).
/// `source_pre_cost_chars` is the tapped source's layer-resolved characteristics snapshotted
/// BEFORE the sacrifice cost (SR-28) — threaded into `mana_source_matches` so a
/// {T}+Sacrifice source (a dead ObjectId here, CR 400.7) still matches per CR 106.12a.
fn fire_mana_triggered_abilities(
    state: &mut GameState,
    player: PlayerId,
    source: ObjectId,
    mana_produced: &[(ManaColor, u32)],
    source_pre_cost_chars: &Option<crate::state::game_object::Characteristics>,
    events: &mut Vec<GameEvent>,
) {
    // Collect permanents on the battlefield with WhenTappedForMana triggered abilities.
    // We snapshot IDs first to avoid borrow conflicts.
    let battlefield_ids: Vec<ObjectId> = state
        .objects
        .values()
        .filter(|o| matches!(o.zone, ZoneId::Battlefield) && o.controller == player)
        .map(|o| o.id)
        .collect();
    for trigger_source_id in battlefield_ids {
        // Get the card_id for registry lookup. PB-OS4b (CR 712.8d/e): also read
        // `is_transformed` here (single lookup, SR-25 ratchet) -- a transformed
        // permanent's WhenTappedForMana triggers come from its back face.
        // `ability_idx` below is a dense index into the effective list; the
        // CardDefETB consumer this pushes to re-derives against
        // `effective_abilities(obj.is_transformed)` at resolution time.
        let (card_id, source_is_transformed) = match state.objects.get(&trigger_source_id) {
            Some(o) => (o.card_id.clone(), o.is_transformed),
            None => continue,
        };
        let card_id = match card_id {
            Some(cid) => cid,
            None => continue,
        };
        // Look up the card definition.
        let registry = state.card_registry.clone();
        let def = match registry.get(card_id) {
            Some(d) => d.clone(),
            None => continue,
        };
        for (ability_idx, ability) in def
            .effective_abilities(source_is_transformed)
            .iter()
            .enumerate()
        {
            let (source_filter, effect, targets) = match ability {
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenTappedForMana { source_filter },
                    effect,
                    targets,
                    ..
                } => (source_filter, effect, targets),
                _ => continue,
            };
            // Check if the tapped source matches the filter.
            if !mana_source_matches(
                state,
                source,
                trigger_source_id,
                source_filter,
                source_pre_cost_chars,
            ) {
                continue;
            }
            // Determine if this is a triggered mana ability (CR 605.1b):
            // no targets + could add mana → resolves immediately (CR 605.4a).
            // Has targets → goes on the stack as a normal triggered ability (CR 605.5a).
            if targets.is_empty() && is_mana_producing_effect(effect) {
                // Triggered mana ability: resolve immediately (CR 605.4a).
                use crate::effects::{execute_effect, EffectContext};
                let dummy_source = trigger_source_id;
                let mut ctx = EffectContext::new(player, dummy_source, vec![]);
                ctx.mana_produced = Some(mana_produced.to_vec());
                let mut mana_events = execute_effect(state, effect, &mut ctx);
                // Tag ManaAdded events with no source (triggered mana is not the original tap).
                // Per Nyxbloom ruling: triggered mana abilities are NOT multiplied.
                events.append(&mut mana_events);
            } else {
                // Targeted / non-mana triggered ability: push to stack (CR 605.5a — an
                // ability with a target is NOT a mana ability).
                //
                // OOS-EF6-1: `ability_idx` here is the RAW index into `def.abilities` (this
                // loop iterates `def.abilities.iter().enumerate()` directly, never the
                // runtime `characteristics.triggered_abilities` vec — `enrich_spec_from_def`
                // has no `WhenTappedForMana` conversion block). `PendingTriggerKind::Normal`
                // resolves targets by reading `characteristics.triggered_abilities[ability_index]`
                // (a DIFFERENT, runtime index space), so a `Normal`-kind trigger here would
                // find nothing and any declared `targets` (e.g. Forbidden Orchard's `target
                // opponent`) would silently resolve to no target. `CardDefETB` is the sibling
                // kind whose target-resolution and effect-resolution both read
                // `def.abilities.get(trigger.ability_index)` — the raw index this loop already
                // holds — so it is the correct kind here even though this is not an ETB event
                // (mirrors the PB-EF3 / EF-W-MISS-10 index-space fix on the attack-trigger
                // path). No new PendingTriggerKind variant; `CardDefETB` is pre-existing.
                let mut trigger = PendingTrigger::blank(
                    trigger_source_id,
                    player,
                    PendingTriggerKind::CardDefETB,
                );
                trigger.ability_index = ability_idx;
                state.pending_triggers.push_back(trigger);
            }
        }
    }
}
/// Check if the tapped permanent (`source`) matches the `ManaSourceFilter` on the
/// trigger source (`trigger_source_id`). The trigger source is the permanent whose
/// ability is firing (e.g., Mirari's Wake, Wild Growth, Forbidden Orchard).
///
/// SR-28 (CR 106.12a): the characteristic-reading arms (Land / LandSubtype / Creature /
/// AnyPermanent) read `source_pre_cost_chars` — the source's layer-resolved characteristics
/// snapshotted before any sacrifice cost — rather than live state, because by the time this
/// fires a {T}+Sacrifice source is a dead ObjectId (CR 400.7) whose live `calculate_characteristics`
/// is `None`. The snapshot is `Some` for any legitimately-tapped permanent. The `EnchantedLand`
/// and `This` arms compare against `trigger_source_id` (still live) and need no snapshot.
fn mana_source_matches(
    state: &GameState,
    source: ObjectId,
    trigger_source_id: ObjectId,
    filter: &ManaSourceFilter,
    source_pre_cost_chars: &Option<crate::state::game_object::Characteristics>,
) -> bool {
    match filter {
        ManaSourceFilter::Land => {
            // Source must be a land controlled by the trigger source's controller.
            source_pre_cost_chars
                .as_ref()
                .map(|c| c.card_types.contains(&CardType::Land))
                .unwrap_or(false)
        }
        ManaSourceFilter::LandSubtype(subtype) => {
            // Source must be a land with the specific subtype.
            source_pre_cost_chars
                .as_ref()
                .map(|c| c.card_types.contains(&CardType::Land) && c.subtypes.contains(subtype))
                .unwrap_or(false)
        }
        ManaSourceFilter::Creature => {
            // Source must be a creature.
            source_pre_cost_chars
                .as_ref()
                .map(|c| c.card_types.contains(&CardType::Creature))
                .unwrap_or(false)
        }
        ManaSourceFilter::AnyPermanent => {
            // Any permanent tapped for mana matches. The snapshot is `Some` iff the source
            // was a real permanent when tapped (CR 106.12a) — this also matches a sacrificed
            // source (e.g. a Treasure), whose live battlefield lookup would now be `None`.
            source_pre_cost_chars.is_some()
        }
        ManaSourceFilter::EnchantedLand => {
            // The trigger source (Aura) must be attached to the tapped permanent.
            state
                .objects
                .get(&trigger_source_id)
                .and_then(|o| o.attached_to)
                .map(|attached_id| attached_id == source)
                .unwrap_or(false)
        }
        ManaSourceFilter::This => {
            // The trigger source IS the tapped permanent.
            trigger_source_id == source
        }
    }
}
/// Returns true if the effect can produce mana (making this ability a triggered mana ability
/// per CR 605.1b when it also has no targets). Only checks top-level mana-producing effects.
fn is_mana_producing_effect(effect: &Effect) -> bool {
    matches!(
        effect,
        Effect::AddMana { .. }
            | Effect::AddManaAnyColor { .. }
            | Effect::AddManaMatchingType { .. }
            | Effect::AddManaChoice { .. }
            | Effect::AddManaFilterChoice { .. }
            | Effect::AddManaScaled { .. }
            | Effect::AddManaRestricted { .. }
            | Effect::AddManaAnyColorRestricted { .. }
            | Effect::AddManaOfAnyColorAmount { .. }
            | Effect::AddManaOfChosenColor { .. }
    )
}
