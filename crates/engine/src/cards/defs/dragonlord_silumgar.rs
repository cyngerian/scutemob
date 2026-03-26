// Dragonlord Silumgar — {4}{U}{B}, Legendary Creature — Elder Dragon 3/5
// Flying, deathtouch
// When Dragonlord Silumgar enters, gain control of target creature or planeswalker
// for as long as you control Dragonlord Silumgar.
//
// Note: "for as long as you control Dragonlord Silumgar" approximated as
// WhileSourceOnBattlefield (correct when Silumgar itself isn't stolen; full
// "while YOU control" semantics need a separate EffectDuration variant).
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
            // CR 613.1b: ETB — gain control of target creature or planeswalker
            // for as long as you control Dragonlord Silumgar (WhileSourceOnBattlefield approx).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::GainControl {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPermanent],
            },
        ],
        ..Default::default()
    }
}
