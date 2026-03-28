// Witch's Cottage — ({T}: Add {B}.) This land enters tapped unless you control three or more
// other Swamps. When this land enters untapped, you may put target creature card from your
// graveyard on top of your library.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("witchs-cottage"),
        name: "Witch's Cottage".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Swamp"]),
        oracle_text: "({T}: Add {B}.)\nThis land enters tapped unless you control three or more other Swamps.\nWhen this land enters untapped, you may put target creature card from your graveyard on top of your library.".to_string(),
        abilities: vec![
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::ControlAtLeastNOtherLandsWithSubtype {
                    count: 3,
                    subtype: SubType("Swamp".to_string()),
                }),
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            // TODO: Triggered — When this land enters untapped, you may put target creature card from your graveyard on top of your library.
        ],
        ..Default::default()
    }
}
