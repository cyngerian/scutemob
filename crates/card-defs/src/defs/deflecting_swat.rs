// Deflecting Swat — {2}{R}, Instant
// If you control a commander, you may cast this without paying its mana cost.
// You may choose new targets for target spell or ability.
//
// CR 118.9: Cast without paying mana cost if you control a commander.
// CR 115.7d: "Choose new targets" — may change any or all targets.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("deflecting-swat"),
        name: "Deflecting Swat".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "If you control a commander, you may cast this spell without paying its mana \
                      cost.\nYou may choose new targets for target spell or ability."
            .to_string(),
        abilities: vec![
            // CR 118.9 / 2020-04-17 ruling: cast without paying mana cost if you control
            // any commander on the battlefield (any player's commander qualifies).
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::CommanderFreeCast,
                cost: ManaCost::default(),
                details: None,
            },
            AbilityDefinition::Spell {
                // CR 115.7d: "You may choose new targets" — must_change: false.
                // Deterministic fallback: targets left unchanged (player "chose" not to change).
                // Interactive choice deferred to M10.
                //
                // PB-DX25b review Finding C3: the printed card says "target spell
                // OR ABILITY", but `targets` below declares `TargetSpell`
                // (spell-only) -- an oracle/def mismatch this batch's census
                // touched (F-A) but did not fix, filed as a candidate seed
                // (`OOS-DX25b-5`). Widening this to
                // `TargetSpellOrAbilityWithSingleTarget`-shaped coverage is
                // BLOCKED by the same missing id space `OOS-DX25b-1` names: an
                // activated/triggered ability's stack entry is never added to
                // `state.objects`, so it could not be announced either way.
                // With `must_change: false` this effect is ALSO a deterministic
                // no-op regardless of the requirement (`effects/mod.rs`'s
                // `!must_change` branch always `continue`s before any mutation)
                // -- do not widen the requirement here; it would change nothing
                // observable and would misrepresent this as a completeness fix.
                effect: Effect::ChangeTargets {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    must_change: false,
                },
                targets: vec![TargetRequirement::TargetSpell],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
