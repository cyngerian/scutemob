// Hidden Strings — {1}{U}, Sorcery
// You may tap or untap target permanent, then you may tap or untap another
// target permanent.
// Cipher
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hidden-strings"),
        name: "Hidden Strings".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "You may tap or untap target permanent, then you may tap or untap another target permanent.\nCipher (Then you may exile this spell card encoded on a creature you control. Whenever that creature deals combat damage to a player, its controller may cast a copy of the encoded card without paying its mana cost.)".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                // TODO: "tap or untap" requires player choice between tap/untap.
                // Using TapPermanent as approximation (more common use case).
                effect: Effect::Sequence(vec![
                    Effect::TapPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::TapPermanent {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                    },
                ]),
                targets: vec![
                    TargetRequirement::TargetPermanent,
                    TargetRequirement::TargetPermanent,
                ],
                modes: None,
                cant_be_countered: false,
            },
            AbilityDefinition::Cipher,
        ],
        completeness: Completeness::known_wrong("modeled as an unconditional double tap. Oracle is 'you MAY tap OR untap' — the untap mode and the optionality are both dropped, and 'another target permanent' distinctness is not enforced. Effect::Choose is non-interactive (effects/mod.rs:3190 always executes the first option), so the mode choice has no correct expression today."),
        ..Default::default()
    }
}
