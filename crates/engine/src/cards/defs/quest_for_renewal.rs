// Quest for Renewal — {1}{G}, Enchantment
// Whenever a creature you control becomes tapped, you may put a quest counter
// on Quest for Renewal.
// As long as there are four or more quest counters on it, untap all creatures
// you control during each other player's untap step.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("quest-for-renewal"),
        name: "Quest for Renewal".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control becomes tapped, you may put a quest counter on Quest for Renewal.\nAs long as there are four or more quest counters on Quest for Renewal, untap all creatures you control during each other player's untap step.".to_string(),
        abilities: vec![
            // TODO: "creature becomes tapped" trigger not in TriggerCondition.
            // TODO: "untap all during other players' untap step" static not in DSL.
        ],
        ..Default::default()
    }
}
