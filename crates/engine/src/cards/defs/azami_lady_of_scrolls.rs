// Azami, Lady of Scrolls — {2}{U}{U}{U} Legendary Creature — Human Wizard 0/2
// Tap an untapped Wizard you control: Draw a card.
//
// DSL gap: "Tap an untapped Wizard you control" as activated ability cost requires
//   Cost::TapAnotherCreatureWithSubtype (no such Cost variant; only Cost::Tap taps this permanent).
// W5 policy: cannot faithfully express tapping another creature of a type as cost — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("azami-lady-of-scrolls"),
        name: "Azami, Lady of Scrolls".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 3, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Wizard"]),
        oracle_text: "Tap an untapped Wizard you control: Draw a card.".to_string(),
        power: Some(0),
        toughness: Some(2),
        abilities: vec![
            // TODO: Tap an untapped Wizard you control: Draw a card.
            //   (Cost enum lacks TapAnotherCreatureWithSubtype variant)
        ],
        ..Default::default()
    }
}
