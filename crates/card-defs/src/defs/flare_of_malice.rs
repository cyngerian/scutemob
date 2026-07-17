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
                    // TODO: Flare of Malice requires greatest-MV-among selection rule —
                    // not expressible as a static TargetFilter. OUT-OF-SCOPE for PB-SFT.
                    Effect::SacrificePermanents {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        count: EffectAmount::Fixed(1),
                        filter: None,
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
        completeness: Completeness::known_wrong("def is authored against text this card does not have. Real oracle is {2}{B}{B} 'Each opponent sacrifices a creature or planeswalker with the greatest mana value among creatures and planeswalkers they control' — def has mana_cost {3}{B}, targets ONE player, sacrifices any permanent incl. lands (filter: None), and adds a nonexistent 'loses 2 life' clause. Requires full re-author. Genuine blockers after that: greatest-MV-among selection rule (not a static TargetFilter), and the sacrifice-a-nontoken-black-creature alt cost (Pitch payer at casting.rs:4205 silently drops Cost::Sacrifice; TargetFilter has no nontoken predicate)."),
        ..Default::default()
    }
}
