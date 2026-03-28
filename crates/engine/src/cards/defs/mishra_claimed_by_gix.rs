// Mishra, Claimed by Gix
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mishra-claimed-by-gix"),
        name: "Mishra, Claimed by Gix".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, red: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Artificer", "Phyrexian"]),
        oracle_text: "Whenever you attack, each opponent loses X life and you gain X life, where X is the number of attacking creatures. If Mishra, Claimed by Gix and a creature named Phyrexian Dragon Engine are attacking, and you both own and control them, exile them, then meld them into Mishra, Lost to Phyrexia. It enters tapped and attacking.".to_string(),
        abilities: vec![
            // Whenever you attack, each opponent loses 1 life and you gain 1 life.
            // TODO: "X = number of attacking creatures" — EffectAmount::AttackingCreatureCount not in DSL.
            // Using Fixed(1) as partial approximation.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouAttack,
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
            // TODO: Meld trigger (Phyrexian Dragon Engine + Mishra meld) — Meld not yet in DSL.
        ],
        power: Some(3),
        toughness: Some(5),
        ..Default::default()
    }
}
