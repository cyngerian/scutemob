// Dragonlord Dromoka — {4}{G}{W}, Legendary Creature — Elder Dragon 5/7
// This spell can't be countered.
// Flying, lifelink
// Your opponents can't cast spells during your turn.
//
// Flying and Lifelink keywords are implemented.
//
// TODO: DSL gap — "This spell can't be countered" is a property of the spell on the stack.
// CardDefinition has no cant_be_countered field; that flag only exists on AbilityDefinition::Spell
// which is used for instants/sorceries. Creature spells that can't be countered cannot be
// expressed in the current DSL. This property is omitted.
//
// TODO: DSL gap — "Your opponents can't cast spells during your turn" requires a
// static ability that restricts opponent actions based on whose turn it is. There is
// no EffectFilter or continuous effect type that enforces casting restrictions on
// opponents during the controller's turn. This ability is omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragonlord-dromoka"),
        name: "Dragonlord Dromoka".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 1, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elder", "Dragon"],
        ),
        oracle_text: "This spell can't be countered.\nFlying, lifelink\nYour opponents can't cast spells during your turn.".to_string(),
        power: Some(5),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
        ],
        ..Default::default()
    }
}
