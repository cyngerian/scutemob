// Boggart Shenanigans — {2}{R} Kindred Enchantment — Goblin
// Whenever another Goblin you control is put into a graveyard from the battlefield,
// you may have this enchantment deal 1 damage to target player or planeswalker.
//
// The trigger + filter + targeted 1 damage are all expressible today. The residual blocker
// is the "you may" optionality: this is a targeted "may" with no pay-cost, and the DSL has no
// non-gated primitive for it (Effect::MayPayThenEffect requires an actual cost; Effect::Choose
// is a gated stub). Because the "may" is target-selecting and player-facing (the controller can
// decline entirely, not just an always-taken upside), it is NOT modeled here — the ability is
// authored as the mandatory ping and the card is kept partial. Same optionality gap that keeps
// Sun Titan / Edric / Fecundity partial.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("boggart-shenanigans"),
        name: "Boggart Shenanigans".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Kindred, CardType::Enchantment], &["Goblin"]),
        oracle_text: "Whenever another Goblin you control is put into a graveyard from the \
                      battlefield, you may have this enchantment deal 1 damage to target player \
                      or planeswalker."
            .to_string(),
        abilities: vec![
            // CR 603.10a: "another Goblin you control is put into a graveyard from the
            // battlefield" = a Goblin you control dies, excluding this enchantment (which is
            // never itself a Goblin creature that can die from the battlefield, but exclude_self
            // matches the "another" wording faithfully).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: true,
                    nontoken_only: false,
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Goblin".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::DealDamage {
                    source: None,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPlayerOrPlaneswalker],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::partial(
            "Trigger, Goblin filter, and targeted 1 damage are all correct. The oracle 'you MAY \
             have this deal 1 damage to target player or planeswalker' cannot be expressed: it is \
             a target-selecting optional effect with no pay-cost, and there is no non-gated \
             optionality primitive (MayPayThenEffect needs a cost; Effect::Choose is a gated \
             stub). Authored as the mandatory ping; the decline option is dropped. Same 'you may' \
             optionality gap that keeps Sun Titan / Edric / Fecundity partial.",
        ),
        ..Default::default()
    }
}
