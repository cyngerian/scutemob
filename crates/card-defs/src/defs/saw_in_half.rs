// Saw in Half — {2}{B}, Instant; destroy target creature.
// If that creature dies this way, its controller creates two tokens that are copies of that
// creature, except their power and toughness are each half (rounded up).
// DSL gap: searched the `Effect` enum (crates/card-types/src/cards/card_definition.rs, 106
// variants) for a copy-token primitive that supports a per-stat modifier. The only candidates
// are `CreateTokenCopy` (5 fields: source, enters_tapped_and_attacking, except_not_legendary,
// gains_haste, delayed_action), `CreateTokenAndAttachSource { spec: TokenSpec }`, and
// `BecomeCopyOf` (copier, target, duration) — none of them carries a power/toughness override,
// so "except their power/toughness are each half, rounded up" cannot be expressed today.
// Implementing: the destroy effect only.
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
            // "If that creature dies this way, its controller creates two tokens that are
            // copies of that creature, except their power is half and their toughness is half
            // (round up)." — not implemented; see the file-level DSL-gap note above.
        }],
        completeness: Completeness::partial(
            "\"Its controller creates two tokens that are copies of that creature, except their \
             power is half that creature's power and their toughness is half that creature's \
             toughness. Round up each time\" is not modeled — no `Effect` variant (106 variants \
             searched in card_definition.rs) supports a per-stat halving modifier on a token copy \
             (CreateTokenCopy/CreateTokenAndAttachSource/BecomeCopyOf all lack one). Only the \
             destroy clause is implemented.",
        ),
        ..Default::default()
    }
}
