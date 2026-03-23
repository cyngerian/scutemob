// World Shaper — {3}{G}, Creature — Merfolk Shaman 3/3
// Whenever this creature attacks, you may mill three cards.
// When this creature dies, return all land cards from your graveyard to the battlefield tapped.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("world-shaper"),
        name: "World Shaper".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: creature_types(&["Merfolk", "Shaman"]),
        oracle_text: "Whenever World Shaper attacks, you may mill three cards.\nWhen World Shaper dies, return all land cards from your graveyard to the battlefield tapped.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // TODO: DSL gap — attack trigger with mill 3. Effect::Mill exists but
            // WhenAttacks + mill combo not tested.
            // TODO: DSL gap — "When this creature dies, return all land cards from your
            // graveyard to the battlefield tapped." WhenDies trigger + mass zone move
            // with land filter from GY.
        ],
        ..Default::default()
    }
}
