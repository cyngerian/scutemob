// Sanctum Seeker — {2}{B}{B}, Creature — Vampire Knight 3/4
// Whenever a Vampire you control attacks, each opponent loses 1 life and you gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sanctum-seeker"),
        name: "Sanctum Seeker".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: creature_types(&["Vampire", "Knight"]),
        oracle_text: "Whenever a Vampire you control attacks, each opponent loses 1 life and you gain 1 life.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // CR 508.1m / CR 603.2: "Whenever a Vampire you control attacks,
            // each opponent loses 1 life and you gain 1 life."
            // PB-N: Vampire subtype filter via triggering_creature_filter.
            // Each opponent loses exactly 1 life (ForEach EachOpponent), then
            // controller gains exactly 1 life (flat — not total_lost). This is
            // distinct from DrainLife, which gains total_lost across all opponents
            // (wrong in 3+ player games per effects/mod.rs:493-504).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Vampire".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::Sequence(vec![
                    Effect::ForEach {
                        over: ForEachTarget::EachOpponent,
                        effect: Box::new(Effect::LoseLife {
                            player: PlayerTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(1),
                        }),
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
