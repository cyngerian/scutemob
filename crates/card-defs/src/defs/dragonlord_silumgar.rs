// Dragonlord Silumgar — {4}{U}{B}, Legendary Creature — Elder Dragon 3/5
// Flying, deathtouch
// When Dragonlord Silumgar enters, gain control of target creature or planeswalker
// for as long as you control Dragonlord Silumgar.
//
// "for as long as you control Dragonlord Silumgar" is EffectDuration::WhileYouControlSource
// (PB-EF9): the borrowed permanent reverts to its owner both when Silumgar leaves the
// battlefield AND when control of Silumgar itself changes away, and never resumes if
// control of Silumgar later returns (CR 611.2c).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragonlord-silumgar"),
        name: "Dragonlord Silumgar".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            blue: 1,
            black: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elder", "Dragon"],
        ),
        oracle_text: "Flying, deathtouch\nWhen Dragonlord Silumgar enters, gain control of target \
                      creature or planeswalker for as long as you control Dragonlord Silumgar."
            .to_string(),
        power: Some(3),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            // CR 613.1b / 611.2b/c: ETB — gain control of target creature or planeswalker
            // for as long as you control Dragonlord Silumgar. PlayerId(0) is the DSL
            // placeholder; Effect::GainControl resolves it to the controller at ETB
            // resolution time (PB-EF9).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::GainControl {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    duration: EffectDuration::WhileYouControlSource(PlayerId(0)),
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Creature, CardType::Planeswalker],
                    ..Default::default()
                })],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
