// 40. Rhystic Study — {2U}, Enchantment; whenever an opponent casts a spell,
// you may draw a card unless that player pays {1}.
// M9.4: payer is DeclaredTarget { index: 0 } — the opponent who cast the spell.
// The triggering opponent is expected to be passed as target 0 when the
// card-def trigger dispatch system is wired up (currently deferred).
// In the interim the draw always fires (payment never collected) because
// triggered abilities resolve with targets: vec![].
//
// SR-33: that admission was in this comment while the def shipped `Complete`. It is now
// `known_wrong`. The deviation scan never looked at this file because the comment
// contains none of its needles ("simplif" / "modeled as" / "approximat" / "deviation") —
// a needle-coverage gap. The structural gate in `tests/core/effect_choose_gate.rs` now
// catches this class regardless of what the comment says.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rhystic-study"),
        name: "Rhystic Study".to_string(),
        mana_cost: Some(ManaCost { blue: 1, generic: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever an opponent casts a spell, you may draw a card unless that player pays {1}.".to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WheneverOpponentCastsSpell {
                spell_type_filter: None,
                noncreature_only: false,
            },
            effect: Effect::MayPayOrElse {
                cost: Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                // DeclaredTarget { index: 0 } = the specific opponent who cast the spell.
                // This is the correct model (CR 603.1): only "that player" pays, not all
                // opponents. Resolves to an empty list at runtime until trigger context
                // wiring passes the casting opponent as target 0.
                payer: PlayerTarget::DeclaredTarget { index: 0 },
                or_else: Box::new(Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                }),
            },
            intervening_if: None,
            targets: vec![],

            modes: None,
            trigger_zone: None,
        }],
        completeness: Completeness::known_wrong(
            "SR-33: the draw always fires and the {1} is never collected — this card's whole \
             point is the tax, so it plays as an unconditional 'draw a card whenever an \
             opponent casts a spell'. Two independent blockers: `Effect::MayPayOrElse` \
             discards `cost`/`payer` and unconditionally runs `or_else` (effects/mod.rs), and \
             the trigger resolves with `targets: vec![]`, so `DeclaredTarget { index: 0 }` \
             (the casting opponent) resolves to an empty list even once choice exists. Needs \
             a general choice Command plus trigger-context target wiring.",
        ),
        ..Default::default()
    }
}
