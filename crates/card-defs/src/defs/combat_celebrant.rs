// Combat Celebrant -- {2}{R}, Creature -- Human Warrior 4/1
// If this creature hasn't been exerted this turn, you may exert it as it attacks.
// When you do, untap all other creatures you control and after this phase, there
// is an additional combat phase. (An exerted creature won't untap during your
// next untap step.)
//
// PB-AC5: Full Exert mechanic implemented (CR 701.43). KeywordAbility::Exert marks
// the "may exert as it attacks" optional cost (CR 508.1g); combat.rs's exert_choices
// validation enforces "hasn't been exerted this turn" (rejects an already-EXERTED
// attacker) and sets Designations::EXERTED (cleared + skips untap at the controller's
// next untap step, CR 701.43a/b). The linked "when you do" trigger below
// (TriggerCondition::WhenExertedAsAttacks, CR 607.2h) fires ONLY when the player
// actually chose to exert this attack -- not on every attack.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("combat-celebrant"),
        name: "Combat Celebrant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: full_types(&[], &[CardType::Creature], &["Human", "Warrior"]),
        oracle_text: "If this creature hasn't been exerted this turn, you may exert it as it \
                      attacks. When you do, untap all other creatures you control and after this \
                      phase, there is an additional combat phase. (An exerted creature won't \
                      untap during your next untap step.)"
            .to_string(),
        power: Some(4),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Exert),
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenExertedAsAttacks,
                intervening_if: None,
                effect: Effect::Sequence(vec![
                    // Untap all other creatures you control (excludes Combat Celebrant itself).
                    Effect::ForEach {
                        over: ForEachTarget::EachOtherCreatureYouControl,
                        effect: Box::new(Effect::UntapPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                        }),
                    },
                    // After this phase, there is an additional combat phase.
                    Effect::AdditionalCombatPhase {
                        followed_by_main: false,
                    },
                ]),
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
