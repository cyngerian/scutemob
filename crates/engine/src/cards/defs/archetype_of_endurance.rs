// Archetype of Endurance — {6}{G}{G}, Enchantment Creature — Boar 6/5
// Creatures you control have hexproof.
// Creatures your opponents control lose hexproof and can't have or gain hexproof.
// TODO: DSL gap — granting hexproof to all creatures you control and stripping it from opponents'
// creatures requires a continuous effect on a filter; no ApplyContinuousEffect with
// EffectFilter::CreaturesYouControl granting Hexproof is currently supported.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("archetype-of-endurance"),
        name: "Archetype of Endurance".to_string(),
        mana_cost: Some(ManaCost { generic: 6, green: 2, ..Default::default() }),
        types: full_types(
            &[],
            &[CardType::Enchantment, CardType::Creature],
            &["Boar"],
        ),
        oracle_text: "Creatures you control have hexproof.\nCreatures your opponents control lose hexproof and can't have or gain hexproof.".to_string(),
        power: Some(6),
        toughness: Some(5),
        abilities: vec![],
        // TODO: grant hexproof to all creatures you control (continuous effect, layer 6)
        // TODO: strip hexproof from opponents' creatures (continuous effect, layer 6)
        ..Default::default()
    }
}
