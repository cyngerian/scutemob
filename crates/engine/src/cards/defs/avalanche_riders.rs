// Avalanche Riders — {3}{R}, Creature — Human Nomad 2/2; Haste; Echo {3}{R};
// When ~ enters, destroy target land.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("avalanche-riders"),
        name: "Avalanche Riders".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: creature_types(&["Human", "Nomad"]),
        oracle_text:
            "Haste\nEcho {3}{R} (At the beginning of your upkeep, if this came under your \
             control since the beginning of your last upkeep, sacrifice it unless you pay its \
             echo cost.)\nWhen this creature enters, destroy target land."
                .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            AbilityDefinition::Keyword(KeywordAbility::Echo(ManaCost {
                generic: 3,
                red: 1,
                ..Default::default()
            })),
            AbilityDefinition::Echo {
                cost: ManaCost { generic: 3, red: 1, ..Default::default() },
            },
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}
