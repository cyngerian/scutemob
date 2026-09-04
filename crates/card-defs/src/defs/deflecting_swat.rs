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
                // `OOS-DX25b-5` CLOSED by PB-DX52 (`scutemob-229`). The printed
                // card says "target spell OR ABILITY"; this def declared the
                // spell-only `TargetSpell` and silently dropped half the line.
                //
                // The note that stood here is REPAIRED IN PLACE rather than
                // deleted, because PB-DX52 falsified it (PB-DX27's rule: a
                // blocker note is a claim). It said widening was "BLOCKED by the
                // same missing id space `OOS-DX25b-1` names" -- true then, false
                // now: `Target::StackObject` is that id space. And it said
                // widening "would change nothing observable" -- also false now.
                // With `must_change: false` the RESOLUTION is still a
                // deterministic no-op (`OOS-DX25b-4`, open, deferred to PB-DX54
                // because CR 115.7d's "you MAY choose new targets" is a player
                // decision needing an `EffectChoiceQuestion` variant), but the
                // ANNOUNCEMENT is not: widening changes the candidate SET the
                // offer layer enumerates, which is observable at
                // `queries::legal_targets_per_slot`, in the browser's target
                // picker, and in `GameEvent::TargetsAnnounced`.
                //
                // `TargetSpellOrAbility` (CR 115.1a / CR 115.7d), NOT
                // `TargetSpellOrAbilityWithSingleTarget`: this card prints no
                // "with a single target" clause, so asserting `targets.len() == 1`
                // would refuse legal targets the printed card admits. That
                // distinction is why PB-DX52 added a variant rather than reusing
                // Bolt Bend's.
                effect: Effect::ChangeTargets {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    must_change: false,
                },
                targets: vec![TargetRequirement::TargetSpellOrAbility],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
