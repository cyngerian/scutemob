// Case of the Locked Hothouse — {3}{G}, Enchantment — Case
// You may play an additional land on each of your turns.
// To solve — You control seven or more lands. (If unsolved, solve at the beginning of your end step.)
// Solved — You may look at the top card of your library any time, and you may play lands and cast
// creature and enchantment spells from the top of your library.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("case-of-the-locked-hothouse"),
        name: "Case of the Locked Hothouse".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: full_types(&[], &[CardType::Enchantment], &["Case"]),
        oracle_text: "You may play an additional land on each of your turns.\nTo solve \u{2014} You control seven or more lands. (If unsolved, solve at the beginning of your end step.)\nSolved \u{2014} You may look at the top card of your library any time, and you may play lands and cast creature and enchantment spells from the top of your library.".to_string(),
        abilities: vec![
            // CR 305.2: Additional land play (always active, both solved and unsolved).
            AbilityDefinition::AdditionalLandPlays { count: 1 },
            // CR 719.3a: "To solve -- You control seven or more lands."
            // At the beginning of your end step, if you control 7+ lands AND this Case is not
            // yet solved, it becomes solved (set SOLVED designation via SolveCase effect).
            // The intervening-if combines both conditions (CR 603.4).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
                effect: Effect::SolveCase,
                intervening_if: Some(Condition::And(
                    Box::new(Condition::YouControlNOrMoreWithFilter {
                        count: 7,
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Land),
                            ..Default::default()
                        },
                    }),
                    Box::new(Condition::Not(Box::new(Condition::SourceIsSolved))),
                )),
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // CR 702.169b: "Solved -- You may look at the top card of your library any time,
            // and you may play lands and cast creature and enchantment spells from the top of
            // your library."
            // TODO: Solved play-from-top ability requires PB-A (play spells/lands from top of
            // library). The SourceIsSolved condition is correct; the play-from-top engine
            // primitive is HIGH complexity and deferred. This ability is intentionally omitted
            // until PB-A is implemented.
        ],
        ..Default::default()
    }
}
