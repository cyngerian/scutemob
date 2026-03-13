// Archetype of Imagination — {4}{U}{U}, Enchantment Creature — Human Wizard 3/2
// Creatures you control have flying.
// Creatures your opponents control lose flying and can't have or gain flying.
//
// TODO: DSL gap — both abilities require controller-aware creature filters:
// 1. "Creatures you control have flying" — requires EffectFilter::CreaturesYouControl
//    (not available; only AllCreatures exists).
// 2. "Creatures your opponents control lose flying and can't have or gain flying" —
//    requires EffectFilter for opponent-controlled creatures and a RemoveKeyword +
//    prevention effect not expressible in the current DSL.
// Both abilities are omitted to avoid incorrect behavior.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("archetype-of-imagination"),
        name: "Archetype of Imagination".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 2, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment, CardType::Creature], &["Human", "Wizard"]),
        oracle_text: "Creatures you control have flying.\nCreatures your opponents control lose flying and can't have or gain flying.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![],
        ..Default::default()
    }
}
