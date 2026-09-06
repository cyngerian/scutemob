// Saw in Half — {2}{B}, Instant; destroy target creature.
// If that creature dies this way, its controller creates two tokens that are copies of that
// creature, except their power and toughness are each half (rounded up).
//
// Only the destroy clause is implemented. The unmodelled clause and the bounded
// no-such-primitive claim are stated ONCE, in the `completeness` marker below — that is the
// only copy a machine reads (`validate_deck`, `tools/authoring-report.py`,
// `core::completeness_deviation_scan`), so a second prose copy here could only rot out of
// step with it. LL-1 (`scutemob-255`) removed that second copy rather than maintain two.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("saw-in-half"),
        name: "Saw in Half".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target creature. If that creature dies this way, its controller \
                      creates two tokens that are copies of that creature, except their power is \
                      half that creature's power and their toughness is half that creature's \
                      toughness. Round up each time."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: EffectTarget::DeclaredTarget { index: 0 },
                cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
            // The "creates two half-size copies" clause is not implemented. It is described
            // once, in this def's `completeness` marker below.
        }],
        completeness: Completeness::partial(
            "\"Its controller creates two tokens that are copies of that creature, except their \
             power is half that creature's power and their toughness is half that creature's \
             toughness. Round up each time\" is not modeled. Searched `Effect` in \
             crates/card-types/src/cards/card_definition.rs at 17fc2834 (106 variants): the only \
             copy-token primitives are CreateTokenCopy (source, enters_tapped_and_attacking, \
             except_not_legendary, gains_haste, delayed_action), CreateTokenAndAttachSource \
             (fixed-P/T TokenSpec) and BecomeCopyOf (copier, target, duration) — none carries a \
             power/toughness override; and `EffectAmount` at the same sha (26 variants) has \
             PowerOf/ToughnessOf but no halving arithmetic to feed one. Only the destroy clause \
             is implemented.",
        ),
        ..Default::default()
    }
}
