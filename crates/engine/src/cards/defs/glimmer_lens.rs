// Glimmer Lens — {1}{W}, Artifact — Equipment
// For Mirrodin! (When this Equipment enters, create a 2/2 red Rebel creature token,
// then attach this to it.)
// Whenever equipped creature and at least one other creature attack, draw a card.
// Equip {1}{W}
//
// TODO: "For Mirrodin!" — ETB token + auto-attach not expressible.
// TODO: "Equipped creature + another attack" trigger not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("glimmer-lens"),
        name: "Glimmer Lens".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: full_types(&[], &[CardType::Artifact], &["Equipment"]),
        oracle_text: "For Mirrodin! (When this Equipment enters, create a 2/2 red Rebel creature token, then attach this to it.)\nWhenever equipped creature and at least one other creature attack, draw a card.\nEquip {1}{W}".to_string(),
        abilities: vec![
            // TODO: For Mirrodin! + equipped attack trigger not expressible.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}
