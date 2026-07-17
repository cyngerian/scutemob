// Bloodsoaked Champion — {B}, Creature — Human Warrior 2/1
// This creature can't block.
// Raid — {1}{B}: Return this card from your graveyard to the battlefield.
// Activate only if you attacked this turn.
//
// CR 508.1 (Raid) / CR 602.2: PB-AC6 added Condition::YouAttackedThisTurn, used here
// as the activation_condition on a graveyard-zone activated ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloodsoaked-champion"),
        name: "Bloodsoaked Champion".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Warrior"]),
        oracle_text: "This creature can't block.\nRaid — {1}{B}: Return this card from your \
                      graveyard to the battlefield. Activate only if you attacked this turn."
            .to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CantBlock),
            // Raid: {1}{B} from graveyard, activate only if you attacked this turn.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 1,
                    black: 1,
                    ..Default::default()
                }),
                effect: Effect::MoveZone {
                    target: EffectTarget::Source,
                    to: ZoneTarget::Battlefield { tapped: false },
                    controller_override: None,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: Some(Condition::YouAttackedThisTurn),
                activation_zone: Some(ActivationZone::Graveyard),
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
