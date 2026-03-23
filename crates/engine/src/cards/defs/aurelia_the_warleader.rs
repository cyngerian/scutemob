// Aurelia, the Warleader — {2}{R}{R}{W}{W}, Legendary Creature — Angel 3/4
// Flying, vigilance, haste
// Whenever Aurelia attacks for the first time each turn, untap all creatures you control.
// After this phase, there is an additional combat phase.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("aurelia-the-warleader"),
        name: "Aurelia, the Warleader".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 2, white: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Angel"]),
        oracle_text: "Flying, vigilance, haste\nWhenever Aurelia attacks for the first time each turn, untap all creatures you control. After this phase, there is an additional combat phase.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // "Whenever Aurelia attacks for the first time each turn" maps to WhenAttacks
            // with Condition::IsFirstCombatPhase (same pattern as Karlach).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                intervening_if: Some(Condition::IsFirstCombatPhase),
                effect: Effect::Sequence(vec![
                    // Untap all creatures you control.
                    Effect::ForEach {
                        over: ForEachTarget::EachCreatureYouControl,
                        effect: Box::new(Effect::UntapPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                        }),
                    },
                    // After this phase, there is an additional combat phase.
                    Effect::AdditionalCombatPhase { followed_by_main: false },
                ]),
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
