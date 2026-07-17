// Guardian Project — {3}{G}, Enchantment
// Whenever a nontoken creature you control enters, if it doesn't have the same name as
// another creature you control or a creature card in your graveyard, draw a card.
//
// TODO: "nontoken" filter — TargetFilter lacks non_token field.
// TODO: Intervening-if "doesn't share name" — name-uniqueness condition not in DSL.
// Implementing as unconditional creature-ETB draw (overbroad approximation).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("guardian-project"),
        name: "Guardian Project".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a nontoken creature you control enters, if it doesn't have the \
                      same name as another creature you control or a creature card in your \
                      graveyard, draw a card."
            .to_string(),
        abilities: vec![
            // TODO: Should be nontoken only + name-uniqueness intervening-if.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::known_wrong(
            "draws on EVERY creature-you-control ETB. Missing (a) the nontoken restriction and \
             (b) the 'doesn't share a name with another creature you control or a creature card \
             in your graveyard' intervening-if — no name-uniqueness Condition exists. Overdraws \
             vs. the real card. (TargetFilter.is_nontoken exists but is ignored by matches_filter \
             — confirm the ETB trigger path checks it.)",
        ),
        ..Default::default()
    }
}
