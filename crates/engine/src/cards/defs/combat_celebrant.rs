// Combat Celebrant -- {2}{R}, Creature -- Human Warrior 4/1
// If this creature hasn't been exerted this turn, you may exert it as it attacks.
// When you do, untap all other creatures you control and after this phase, there
// is an additional combat phase. (An exerted creature won't untap during your
// next untap step.)
//
// NOTE: Exert mechanic is not yet implemented (future primitive). This card def
// uses a simplified trigger: whenever this creature attacks (first combat phase
// only), untap all other creatures you control and create an additional combat
// phase. The "hasn't been exerted this turn" check and "won't untap next turn"
// enforcement are TODO pending an Exert primitive.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("combat-celebrant"),
        name: "Combat Celebrant".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: full_types(&[], &[CardType::Creature], &["Human", "Warrior"]),
        oracle_text: "If this creature hasn't been exerted this turn, you may exert it as it attacks. When you do, untap all other creatures you control and after this phase, there is an additional combat phase. (An exerted creature won't untap during your next untap step.)".to_string(),
        power: Some(4),
        toughness: Some(1),
        abilities: vec![
            // TODO: Full Exert mechanic not yet implemented.
            // Simplified as: whenever this creature attacks (first combat phase),
            // untap all other creatures you control and add an additional combat phase.
            // Missing: exert "won't untap" tracking, once-per-turn exert limit.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                intervening_if: Some(Condition::IsFirstCombatPhase),
                effect: Effect::Sequence(vec![
                    // Untap all other creatures you control (excludes Combat Celebrant itself).
                    Effect::ForEach {
                        over: ForEachTarget::EachOtherCreatureYouControl,
                        effect: Box::new(Effect::UntapPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                        }),
                    },
                    // After this phase, there is an additional combat phase.
                    Effect::AdditionalCombatPhase { followed_by_main: false },
                ]),
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
