// Flare of Malice — {2}{B}{B}, Instant
// You may sacrifice a nontoken black creature rather than pay this spell's mana cost.
// Each opponent sacrifices a creature or planeswalker with the greatest mana value
// among creatures and planeswalkers they control.
//
// PB-DX27 (2026-08-13): this header repeated the same fictional second sentence as
// `oracle_text` did. Both are now the printed text.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flare-of-malice"),
        name: "Flare of Malice".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        // PB-DX27 (2026-08-13), OOS-CARDS2-10: the second sentence was a DIFFERENT CARD'S
        // text ("Target opponent sacrifices a nonland permanent and loses 2 life"), and the
        // def's abilities were authored FROM it. This is the exact input class that caused
        // the `braided_net` three-invented-abilities incident. The abilities stay blocked
        // and `known_wrong` (see the note); only the fiction is removed.
        // Replaced with the MCP-verified printed text.
        oracle_text: "You may sacrifice a nontoken black creature rather than pay this spell's \
                      mana cost.\nEach opponent sacrifices a creature or planeswalker with the \
                      greatest mana value among creatures and planeswalkers they control."
            .to_string(),
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
        completeness: Completeness::known_wrong(
            "def is authored against text this card does not have. Real oracle is {2}{B}{B} 'Each \
             opponent sacrifices a creature or planeswalker with the greatest mana value among \
             creatures and planeswalkers they control'. CARDS-2 (scutemob-181) repaired the \
             mana_cost — it read {3}{B} and is now the printed {2}{B}{B} — but ONLY that: the def \
             still targets ONE player, sacrifices any permanent incl. lands (filter: None), and \
             adds a nonexistent 'loses 2 life' clause. Requires full re-author. Genuine blockers \
             after that: greatest-MV-among selection rule (not a static TargetFilter), and the \
             sacrifice-a-nontoken-black-creature alt cost (Pitch payer at casting.rs:4205 \
             silently drops Cost::Sacrifice; TargetFilter has no nontoken predicate).",
        ),
        ..Default::default()
    }
}
