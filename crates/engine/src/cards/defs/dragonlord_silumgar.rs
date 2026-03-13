// Dragonlord Silumgar — {4}{U}{B}, Legendary Creature — Elder Dragon 3/5
// Flying, deathtouch
// When Dragonlord Silumgar enters, gain control of target creature or planeswalker
// for as long as you control Dragonlord Silumgar.
//
// TODO: DSL gap — ETB control effect omitted.
// "Gain control of target creature or planeswalker for as long as you control Dragonlord Silumgar."
// Requires a conditional continuous control effect (EffectDuration::WhileYouControlSource)
// targeting either a creature or planeswalker. No duration variant for "while you control [this]"
// and no multi-type target filter (creature OR planeswalker) in the current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragonlord-silumgar"),
        name: "Dragonlord Silumgar".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elder", "Dragon"],
        ),
        oracle_text: "Flying, deathtouch\nWhen Dragonlord Silumgar enters, gain control of target creature or planeswalker for as long as you control Dragonlord Silumgar.".to_string(),
        power: Some(3),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
        ],
        ..Default::default()
    }
}
