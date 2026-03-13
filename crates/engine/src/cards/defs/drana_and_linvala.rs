// Drana and Linvala — {1}{W}{W}{B}, Legendary Creature — Vampire Angel 3/4
// Flying, vigilance; static: opponents' creature activated abilities can't be activated;
// static: this creature has all activated abilities of opponents' creatures.
// TODO: Both static abilities (ability suppression + ability copying from opponents' creatures)
// are not expressible in the DSL. No LayerModification for ability suppression on opponents'
// permanents or for copying activated abilities from other objects. Deferred.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("drana-and-linvala"),
        name: "Drana and Linvala".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 2, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Vampire", "Angel"],
        ),
        oracle_text: "Flying, vigilance\nActivated abilities of creatures your opponents control can't be activated.\nDrana and Linvala has all activated abilities of all creatures your opponents control. You may spend mana as though it were mana of any color to activate those abilities.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // TODO: Static — activated abilities of opponents' creatures can't be activated.
            // DSL gap: no LayerModification for ability suppression scoped to opponents' permanents.
            // TODO: Static — this has all activated abilities of opponents' creatures.
            // DSL gap: no ability-copying LayerModification in DSL. Deferred.
        ],
        ..Default::default()
    }
}
