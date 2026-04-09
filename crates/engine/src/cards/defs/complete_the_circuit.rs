// Complete the Circuit — {5}{U}, Instant
// Convoke
// You may cast sorcery spells this turn as though they had flash.
// When you next cast an instant or sorcery spell this turn, copy that spell twice.
//   You may choose new targets for the copies.
// PARTIAL (PB-J): Effect::CopySpellOnStack is now available in the DSL.
//   The remaining gap is the "When you NEXT cast an instant or sorcery spell THIS TURN"
//   delayed trigger — this requires a DelayedTrigger variant keyed on "next spell cast
//   matching a filter during current turn", which is a separate engine primitive not yet
//   built. The GrantFlash effect is correct and fully implemented. The copy-twice
//   trigger is deferred until the delayed-spell-cast-trigger primitive is added.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("complete-the-circuit"),
        name: "Complete the Circuit".to_string(),
        mana_cost: Some(ManaCost { generic: 5, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Convoke (Your creatures can help cast this spell. Each creature you tap while casting this spell pays for {1} or one mana of that creature's color.)\nYou may cast sorcery spells this turn as though they had flash.\nWhen you next cast an instant or sorcery spell this turn, copy that spell twice. You may choose new targets for the copies.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Convoke),
            // CR 601.3b: Grant flash for sorceries until end of turn.
            AbilityDefinition::Spell {
                effect: Effect::GrantFlash {
                    filter: FlashGrantFilter::Sorceries,
                    duration: EffectDuration::UntilEndOfTurn,
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
