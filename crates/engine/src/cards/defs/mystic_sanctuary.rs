// Mystic Sanctuary — ({T}: Add {U}.) This land enters tapped unless you control three or more
// other Islands. When this land enters untapped, you may put target instant or sorcery card from
// your graveyard on top of your library.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mystic-sanctuary"),
        name: "Mystic Sanctuary".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Island"]),
        oracle_text: "({T}: Add {U}.)\nThis land enters tapped unless you control three or more other Islands.\nWhen this land enters untapped, you may put target instant or sorcery card from your graveyard on top of your library.".to_string(),
        abilities: vec![
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::ControlAtLeastNOtherLandsWithSubtype {
                    count: 3,
                    subtype: SubType("Island".to_string()),
                }),
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: Triggered — When this land enters untapped, you may put target instant or sorcery card from your graveyard on top of your library.
        ],
        ..Default::default()
    }
}
