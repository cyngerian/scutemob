// Malakir Bloodwitch — {3}{B}{B}, Creature — Vampire Shaman 4/4
// Flying, protection from white
// When this creature enters, each opponent loses life equal to the number of
// Vampires you control. You gain life equal to the life lost this way.
use crate::cards::helpers::*;
use crate::state::types::ProtectionQuality;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("malakir-bloodwitch"),
        name: "Malakir Bloodwitch".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: creature_types(&["Vampire", "Shaman"]),
        oracle_text: "Flying, protection from white\nWhen this creature enters, each opponent loses life equal to the number of Vampires you control. You gain life equal to the life lost this way.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(
                ProtectionQuality::FromColor(Color::White),
            )),
            // CR 702.101a: DrainLife — each opponent loses N, controller gains total lost.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrainLife {
                    amount: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            has_subtype: Some(SubType("Vampire".to_string())),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
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
