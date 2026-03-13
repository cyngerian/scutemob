// Neriv, Heart of the Storm — {1}{R}{W}{B}, Legendary Creature — Spirit Dragon 4/5
// Flying
// If a creature you control that entered this turn would deal damage, it deals twice that much damage instead.
// TODO: DSL gap — replacement effect on damage amount conditioned on whether the source
// creature entered this turn; no ReplacementTrigger::WouldDealDamage with entering-this-turn filter.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("neriv-heart-of-the-storm"),
        name: "Neriv, Heart of the Storm".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, white: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Spirit", "Dragon"],
        ),
        oracle_text: "Flying\nIf a creature you control that entered this turn would deal damage, it deals twice that much damage instead.".to_string(),
        power: Some(4),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: replacement effect — creatures you control that entered this turn deal double damage
        ],
        ..Default::default()
    }
}
