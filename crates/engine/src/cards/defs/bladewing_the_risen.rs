// Bladewing the Risen — {3}{B}{B}{R}{R}, Legendary Creature — Zombie Dragon 4/4
// Flying
// TODO: DSL gap — ETB ability requires returning target Dragon card from graveyard to battlefield
//   (targeted graveyard retrieval not supported)
// TODO: DSL gap — activated ability {B}{R} gives Dragon creatures +1/+1 until end of turn
//   (creature-type-filtered continuous effect with temporary duration not supported)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bladewing-the-risen"),
        name: "Bladewing the Risen".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            black: 2,
            red: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Zombie", "Dragon"],
        ),
        oracle_text: "Flying\nWhen Bladewing enters, you may return target Dragon permanent card from your graveyard to the battlefield.\n{B}{R}: Dragon creatures get +1/+1 until end of turn.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
        ],
        ..Default::default()
    }
}
