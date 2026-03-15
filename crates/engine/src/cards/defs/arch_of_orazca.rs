// Arch of Orazca — Land, Ascend; {T}: Add {C}; {5},{T}: Draw a card (city's blessing required)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("arch-of-orazca"),
        name: "Arch of Orazca".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "Ascend (If you control ten or more permanents, you get the city's blessing for the rest of the game.)\n{T}: Add {C}.\n{5}, {T}: Draw a card. Activate only if you have the city's blessing.".to_string(),
        abilities: vec![
            // Ascend keyword — triggers city's blessing check
            AbilityDefinition::Keyword(KeywordAbility::Ascend),
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // {5}, {T}: Draw a card. Activate only if you have the city's blessing.
            // Note: Oracle says "activate only if" (activation restriction). Modeled as
            // conditional effect — cost is still paid without blessing. Proper activation
            // restriction requires legal_actions filter (PB-18 stax framework).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 5, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::Conditional {
                    condition: Condition::HasCitysBlessing,
                    if_true: Box::new(Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    }),
                    if_false: Box::new(Effect::Nothing),
                },
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
