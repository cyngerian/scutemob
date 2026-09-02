// Vampire Gourmand — {1}{B}, Creature — Vampire 2/2
// Whenever this creature attacks, you may sacrifice another creature. If you do, draw a
// card and this creature can't be blocked this turn.
//
// Authored below (CR 118.12 / 109.1 / CR 509.1). The claimed blocker is FALSE at HEAD:
// `Cost::Sacrifice(TargetFilter { exclude_self: true, .. })` DOES have exclude-self semantics
// — `can_pay_optional_cost` threads `source: Option<ObjectId>` (effects/mod.rs:9331-9337) into
// `sacrifice_permanents_for_player` (effects/mod.rs:9404-9445), and
// `eligible_sacrifice_targets` enforces `tf.exclude_self && source == Some(id)`
// (effects/mod.rs:9053-9081) — the identical shape closed on `disciple_of_freyalise.rs` (PB-EF1)
// and `ruthless_technomancer.rs` (this batch). Vampire Gourmand can never illegally sacrifice
// itself with `exclude_self: true`. "Can't be blocked this turn" is a temporary
// `ApplyContinuousEffect` granting `KeywordAbility::CantBeBlocked` (precedent:
// `rogues_passage.rs`), scoped to the source via `EffectFilter::Source`
// (continuous_effect.rs:129-133), not `DeclaredTarget` — there is no target here, the ability
// grants evasion to itself.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vampire-gourmand"),
        name: "Vampire Gourmand".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 1,
            ..Default::default()
        }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Whenever this creature attacks, you may sacrifice another creature. If you \
                      do, draw a card and this creature can't be blocked this turn."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 118.12 / 109.1 / CR 509.1: "Whenever this creature attacks, you may sacrifice
            // another creature. If you do, draw a card and this creature can't be blocked this
            // turn."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::MayPayThenEffect {
                    cost: Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::You,
                        exclude_self: true,
                        ..Default::default()
                    }),
                    payer: PlayerTarget::Controller,
                    then: Box::new(Effect::Sequence(vec![
                        Effect::DrawCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(1),
                        },
                        Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: crate::state::EffectLayer::Ability,
                                modification: crate::state::LayerModification::AddKeyword(
                                    KeywordAbility::CantBeBlocked,
                                ),
                                filter: crate::state::EffectFilter::Source,
                                duration: crate::state::EffectDuration::UntilEndOfTurn,
                                condition: None,
                            }),
                        },
                    ])),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        // PB-DX45 (`scutemob-217`, 2026-09-02): PROMOTED partial -> Complete. This
        // def's marker named exactly one blocker — `MayPayThenEffect` being
        // pay-when-able, so the printed "you may" was auto-taken — and that is the
        // defect PB-DX45 closed (`OOS-DX24-9` = `OOS-DX27-5`). The sacrifice is now
        // a real CR 118.12 decision on PB-DP9's CR 608.2d channel, offered to the
        // controller through `EffectChoiceQuestion::PayOptionalCost` and declinable
        // from every client. The ruling that promotes it is recorded in
        // `memory/decisions.md`; it also holds `disciple_of_freyalise` at Complete
        // and leaves `ruthless_technomancer` at partial, whose marker names a
        // DIFFERENT, still-live gap (its activated variable-X sacrifice cost).
        //
        // The residual is stated rather than left implicit: WHICH creature is
        // sacrificed is still the engine's lowest-`ObjectId` pick (`OOS-DX45-1`).
        // That does not bar `Complete` — the same auto-pick governs the whole
        // `Effect::SacrificePermanents` family (Fleshbag Marauder and its nine
        // siblings), which has shipped `Complete` throughout — and the ruling says
        // so in terms rather than leaving the two conventions to be compared.
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
