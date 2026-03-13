// Goblin Warchief — {1}{R}{R}, Creature — Goblin Warrior 2/2
// Goblin spells you cast cost {1} less to cast.
// Goblins you control have haste.
//
// TODO: DSL gap — "Goblin spells you cast cost {1} less to cast" requires a
// SpellCostReduction effect filtered by subtype (Goblin). No cost-reduction
// continuous effect with subtype filter exists in the DSL.
//
// TODO: DSL gap — "Goblins you control have haste" requires a keyword-grant continuous
// effect filtered to creatures you control with a specific subtype (Goblin).
// EffectFilter::AllCreatures applies to all creatures; there is no
// EffectFilter::CreaturesYouControlWithSubtype variant. Both abilities are omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-warchief"),
        name: "Goblin Warchief".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 2, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Goblin spells you cast cost {1} less to cast.\nGoblins you control have haste.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        ..Default::default()
    }
}
