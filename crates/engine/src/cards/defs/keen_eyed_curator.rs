// Keen-Eyed Curator — {G}{G}, Creature — Raccoon Scout 3/3
// As long as there are four or more card types among cards exiled with this creature,
// it gets +4/+4 and has trample.
// {1}: Exile target card from a graveyard.
// TODO: DSL gap — conditional static buff based on counting distinct card types among
// exiled cards attached to this permanent requires a count_threshold pattern not in DSL.
// The activated exile ability also needs an EffectTarget for graveyard cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("keen-eyed-curator"),
        name: "Keen-Eyed Curator".to_string(),
        mana_cost: Some(ManaCost { green: 2, ..Default::default() }),
        types: creature_types(&["Raccoon", "Scout"]),
        oracle_text: "As long as there are four or more card types among cards exiled with this creature, it gets +4/+4 and has trample.\n{1}: Exile target card from a graveyard.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![],
        // TODO: conditional +4/+4 and trample when 4+ card types exiled with this creature
        // TODO: {1}: exile target card from a graveyard
        ..Default::default()
    }
}
