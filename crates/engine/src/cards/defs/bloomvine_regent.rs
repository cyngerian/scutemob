// Bloomvine Regent // Claim Territory — {3}{G}{G} Creature — Dragon 4/5
// Flying
// Whenever this creature or another Dragon you control enters, you gain 3 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloomvine-regent"),
        name: "Bloomvine Regent // Claim Territory".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhenever this creature or another Dragon you control enters, you gain 3 life.".to_string(),
        power: Some(4),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // "Whenever this creature or another Dragon you control enters, you gain 3 life."
            // WheneverCreatureEntersBattlefield fires on AnyPermanentEntersBattlefield events,
            // which includes the source itself entering. Dragon subtype + controller_you filter
            // correctly matches self (a Dragon) and other Dragons you control.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Dragon".to_string())),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(3),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
