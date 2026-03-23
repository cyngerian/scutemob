// Izoni, Thousand-Eyed — {2}{B}{B}{G}{G}, Legendary Creature — Elf Shaman 2/3
// Undergrowth — When Izoni enters, create a 1/1 black and green Insect creature token
// for each creature card in your graveyard.
// {B}{G}, Sacrifice another creature: You gain 1 life and draw a card.
//
// TODO: Token count = creature cards in graveyard — EffectAmount::CardCount with
//   creature filter in graveyard. TokenSpec.count is fixed u32.
// TODO: "Sacrifice another creature" cost not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("izoni-thousand-eyed"),
        name: "Izoni, Thousand-Eyed".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, green: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elf", "Shaman"]),
        oracle_text: "Undergrowth — When Izoni, Thousand-Eyed enters, create a 1/1 black and green Insect creature token for each creature card in your graveyard.\n{B}{G}, Sacrifice another creature: You gain 1 life and draw a card.".to_string(),
        power: Some(2),
        toughness: Some(3),
        // TODO: Variable token count + sacrifice-other cost not expressible.
        abilities: vec![],
        ..Default::default()
    }
}
