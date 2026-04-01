// Complete the Circuit — {5}{U}, Instant
// Convoke
// You may cast sorcery spells this turn as though they had flash.
// When you next cast an instant or sorcery spell this turn, copy that spell twice.
//   You may choose new targets for the copies.
//
// TODO: "You may cast sorcery spells this turn as though they had flash" — no
//   Effect::GrantFlashToSorceries or EffectDuration::UntilEndOfTurn on card-type-restriction
//   lifting exists in DSL.
// TODO: "When you next cast an instant or sorcery spell this turn, copy that spell twice" —
//   no delayed trigger (WhenNextCastSpell) or Effect::CopySpellTwice in DSL.
// Convoke keyword IS supported and is expressed.
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
            // TODO: "cast sorcery spells as though they had flash" — DSL gap (no grant-flash-to-type effect).
            // TODO: "When you next cast an instant/sorcery this turn, copy that spell twice" —
            //   DSL gap (no delayed-trigger WhenNextCastSpell + Effect::CopySpellTwice).
        ],
        ..Default::default()
    }
}
