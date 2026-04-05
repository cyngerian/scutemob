// Neriv, Heart of the Storm — {1}{R}{W}{B}, Legendary Creature — Spirit Dragon 4/5
// Flying
// If a creature you control that entered this turn would deal damage, it deals
// twice that much damage instead.
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
            // CR 614.1: "If a creature you control that entered this turn would deal damage,
            // it deals twice that much damage instead."
            // Static replacement effect registered when Neriv enters the battlefield.
            // PlayerId(0) is bound to the controller at registration time.
            // The filter checks: source is a creature controlled by PlayerId, AND
            // source.entered_turn == current turn number.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::DamageWouldBeDealt {
                    target_filter: DamageTargetFilter::FromControllerCreaturesEnteredThisTurn(
                        PlayerId(0),
                    ),
                },
                modification: ReplacementModification::DoubleDamage,
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}
