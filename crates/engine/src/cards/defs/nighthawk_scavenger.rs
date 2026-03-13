// Nighthawk Scavenger — {1}{B}{B}, Creature — Vampire Rogue 1+*/3
// Flying, deathtouch, lifelink
// TODO: DSL gap — CDA "Nighthawk Scavenger's power is equal to 1 plus the number of card types
//   among cards in your opponents' graveyards."
//   (EffectLayer::PtCda counting distinct card types across opponents' graveyards not supported)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nighthawk-scavenger"),
        name: "Nighthawk Scavenger".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 2,
            ..Default::default()
        }),
        types: creature_types(&["Vampire", "Rogue"]),
        oracle_text: "Flying, deathtouch, lifelink\nNighthawk Scavenger's power is equal to 1 plus the number of card types among cards in your opponents' graveyards.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
        ],
        ..Default::default()
    }
}
