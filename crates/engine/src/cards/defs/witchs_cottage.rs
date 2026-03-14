// Witch's Cottage
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("witchs-cottage"),
        name: "Witch's Cottage".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Swamp"]),
        oracle_text: "({T}: Add {B}.)\nThis land enters tapped unless you control three or more other Swamps.\nWhen this land enters untapped, you may put target creature card from your graveyard on top of your library.".to_string(),
        abilities: vec![
            // TODO: ({T}: Add {B}.)
            // TODO: Triggered — When this land enters untapped, you may put target creature card from your grave
        ],
        ..Default::default()
    }
}
