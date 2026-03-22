// Saw in Half — {2}{B}, Instant; destroy target creature.
// If that creature dies this way, its controller creates two tokens that are copies of that
// creature, except their power and toughness are each half (rounded up).
// TODO: DSL gap — creating two copy-tokens with halved stats requires CreateTokenCopy with
// stat modifications; the DSL does not support fractional/halved stat overrides on token copies.
// Implementing: the destroy effect only.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("saw-in-half"),
        name: "Saw in Half".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target creature. If that creature dies this way, its controller creates two tokens that are copies of that creature, except their power is half that creature's power and their toughness is half that creature's toughness. Round up each time.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
            // TODO: "If that creature dies this way, its controller creates two tokens that are
            // copies of that creature, except their power is half and their toughness is half
            // (round up)." — requires CreateTokenCopy with per-stat halving modification.
            // DSL gap: no halved-stat copy token variant exists.
        }],
        ..Default::default()
    }
}
