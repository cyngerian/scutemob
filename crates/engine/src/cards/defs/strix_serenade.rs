// Strix Serenade — {U}, Instant
// Counter target artifact, creature, or planeswalker spell. Its controller
// creates a 2/2 blue Bird creature token with flying.
//
// TODO: DSL gap — CounterSpell targets any spell; TargetRequirement lacks a
// multi-type filter (artifact OR creature OR planeswalker). The "its
// controller creates a token" also targets the controller of the countered
// spell (not the caster), which is not expressible in the DSL.
// Abilities are omitted to avoid incorrect behavior.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("strix-serenade"),
        name: "Strix Serenade".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target artifact, creature, or planeswalker spell. Its controller creates a 2/2 blue Bird creature token with flying.".to_string(),
        abilities: vec![
            // TODO: DSL gap — multi-type target filter (artifact/creature/planeswalker)
            // and "its controller" token creation not expressible.
        ],
        ..Default::default()
    }
}
