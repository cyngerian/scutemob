// Emeria, the Sky Ruin — Land
// This land enters tapped.
// At the beginning of your upkeep, if you control seven or more Plains, you may
// return target creature card from your graveyard to the battlefield.
// {T}: Add {W}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("emeria-the-sky-ruin"),
        name: "Emeria, the Sky Ruin".to_string(),
        mana_cost: None,
        // PB-DX3b fix cycle (review Finding 1): NOT Legendary. MCP's type line is `Land`,
        // not `Legendary Land` — verified independently three ways (this batch's own
        // lookup_card, the reviewer's, and a control test: Gaea's Cradle is genuinely
        // `Legendary Land`, Valakut, the Molten Pinnacle is genuinely `Land`; Emeria is in
        // Valakut's Zendikar cycle, which is nonlegendary despite the comma name). A
        // spurious `Legendary` supertype would wrongly apply CR 704.5j (legend rule) to a
        // duplicate Emeria the real card permits.
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\nAt the beginning of your upkeep, if you control \
                      seven or more Plains, you may return target creature card from your \
                      graveyard to the battlefield.\n{T}: Add {W}."
            .to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement — this land enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // CR 603.4: Upkeep trigger — if you control 7+ Plains, you MAY return a
            // creature card from your graveyard to the battlefield.
            //
            // PB-DX3b (OOS-DX3-1): the def's former note said the intervening-if needed
            // a `Condition::YouControlNOrMorePermanentsWithSubtype` variant that "does
            // not exist yet" — stale. `Condition::YouControlNOrMoreWithFilter { count: 7,
            // filter: has_subtype Plains }` says exactly this and is queue-time evaluable
            // (checked at rules/turn_actions.rs's AtBeginningOfYourUpkeep CardDef sweep,
            // PB-DP6, and re-checked at resolution, InterveningIf::CardDef, PB-DX1). Fixed
            // below — this closes the LIVE-WRONG half: pre-fix, Emeria reanimated a
            // creature every upkeep regardless of Plains count (this def was `Complete`
            // only by `#[default]` — see the completeness note below).
            //
            // Emeria has no Plains subtype herself (a plain Land, no basic land types,
            // and — fix cycle Finding 1 — not Legendary either), so she never counts
            // toward her own threshold and `exclude_self` is unnecessary — deliberately
            // left unset rather than set to a no-op true.
            //
            // The printed "you MAY return" clause is still NOT implemented: there is no
            // free-optional effect in the DSL. `Effect::MayPayThenEffect` requires a
            // `Cost` (its `then` runs only if the — deterministic, non-interactive —
            // payment succeeds); a `Cost::None`-style "free cost" would always trivially
            // "pay" and `then` would fire every time regardless, which is not a real
            // choice and is byte-for-byte the same observable behaviour as the
            // unconditional effect below. `Effect::MayPayOrElse` is the tax shape (CR
            // 118.12a) and is a documented STUB besides. PB-DP9's `pending_effect_choice`
            // interactive channel serves only search/scry/surveil (CR 608.2d), not a bare
            // "you may [effect]" choice. So the reanimation below still fires
            // unconditionally once the intervening-if passes — see the explicit `partial`
            // marker on this def; do not read `Complete` into this ability.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Battlefield { tapped: false },
                    controller_override: None,
                },
                intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
                    count: 7,
                    filter: TargetFilter {
                        has_subtype: Some(SubType("Plains".to_string())),
                        ..Default::default()
                    },
                }),
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                })],

                modes: None,
                trigger_zone: None,
            },
            // {T}: Add {W}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        // PB-DX3b: this def was `Complete` only by `#[default]` (Completeness derives
        // `#[default] Complete`, so ANY def ending `..Default::default()` without an
        // explicit `completeness:` field is Complete and deck-legal — the same trap that
        // made `aurelia_the_warleader` live-wrong before PB-DX1). Nobody ever asserted
        // Emeria was complete. Made explicit here, and set to `partial` because the
        // printed "you may" is genuinely unimplemented (see the ability comment above) —
        // per Completeness's own contract, "some clauses are implemented and at least one
        // is not" is Partial, not Complete. The intervening-if fix above closes the
        // live-wrong half (unconditional reanimation); this marker honestly records the
        // remaining gap instead of silently covering it.
        completeness: Completeness::partial(
            "The upkeep trigger's intervening-if (7+ Plains) is now correctly gated (PB-DX3b, \
             Condition::YouControlNOrMoreWithFilter). Still unimplemented: the printed 'you MAY \
             return' is authored as an unconditional MoveZone once the intervening-if passes, \
             because the DSL has no free-optional effect — MayPayThenEffect requires a Cost and a \
             free one would always trivially pay, which is not a real choice; MayPayOrElse is a \
             documented STUB (SR-33); PB-DP9's pending_effect_choice channel serves only \
             search/scry/surveil. ALSO fixed this fix cycle (review Finding 1): the def \
             previously carried a spurious `Legendary` supertype not present on the MCP type line \
             (`Land`, not `Legendary Land` — control-verified against Gaea's Cradle vs. Valakut, \
             the Molten Pinnacle, Emeria's own Zendikar cycle-mate); `types` now reads \
             `types(&[CardType::Land])`. Same shape as OOS-DP10-8 (Smuggler's Copter's 'you may \
             draw' authored as an unconditional Sequence).",
        ),
        ..Default::default()
    }
}
