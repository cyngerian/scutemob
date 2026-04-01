// Flare of Malice — {3}{B}, Instant
// You may sacrifice a nontoken black creature rather than pay this spell's mana cost.
// Target opponent sacrifices a nonland permanent and loses 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flare-of-malice"),
        name: "Flare of Malice".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "You may sacrifice a nontoken black creature rather than pay this spell's mana cost.\nTarget opponent sacrifices a nonland permanent and loses 2 life.".to_string(),
        abilities: vec![
            // TODO: Sacrifice-creature alt cost not in DSL.
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    // TODO: SacrificePermanents has no nonland filter — opponent
                    // sacrifices any permanent, not specifically a nonland.
                    Effect::SacrificePermanents {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::LoseLife {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(2),
                    },
                ]),
                targets: vec![TargetRequirement::TargetPlayer],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
