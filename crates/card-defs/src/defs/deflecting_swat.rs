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
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If you control a commander, you may cast this spell without paying its mana cost.\nYou may choose new targets for target spell or ability.".to_string(),
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
                // Deflecting Swat can target ANY spell or ability (not just single-target ones).
                // Deterministic fallback: targets left unchanged (player "chose" not to change).
                // Interactive choice deferred to M10.
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
