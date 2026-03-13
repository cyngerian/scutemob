// Darksteel Mutation — {1}{W}, Enchantment — Aura
// Enchant creature
// Enchanted creature is an Insect artifact creature with base power and toughness 0/1
// and has indestructible, and it loses all other abilities, card types, and creature types.
//
// TODO: DSL gap — this Aura applies multiple simultaneous layer effects to the enchanted creature:
// 1. Layer 4: set card types to {Artifact, Creature} and subtype to {Insect} (losing others).
// 2. Layer 7b: set base P/T to 0/1.
// 3. Layer 6: add Indestructible.
// 4. Layer 6: remove all other abilities (RemoveAllAbilities + partial).
// The combination of SetTypeLine + SetPowerToughness + AddKeyword + RemoveAllAbilities
// on AttachedCreature in a single Aura is not expressible as a single ContinuousEffectDef.
// Multiple Static abilities on an Aura may be possible but the interaction order is complex.
// All abilities are omitted to avoid incorrect behavior.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("darksteel-mutation"),
        name: "Darksteel Mutation".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nEnchanted creature is an Insect artifact creature with base power and toughness 0/1 and has indestructible, and it loses all other abilities, card types, and creature types.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
