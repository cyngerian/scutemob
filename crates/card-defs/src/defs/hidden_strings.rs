// Hidden Strings — {1}{U}, Sorcery
// You may tap or untap target permanent, then you may tap or untap another
// target permanent.
// Cipher
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hidden-strings"),
        name: "Hidden Strings".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "You may tap or untap target permanent, then you may tap or untap another \
                      target permanent.\nCipher (Then you may exile this spell card encoded on a \
                      creature you control. Whenever that creature deals combat damage to a \
                      player, its controller may cast a copy of the encoded card without paying \
                      its mana cost.)"
            .to_string(),
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
                    // PB-OS10 (OOS-XS-1): CR 601.2c "another target permanent" — must differ
                    // from requirement slot 0.
                    TargetRequirement::TargetPermanentDistinctFrom(0),
                ],
                modes: None,
                cant_be_countered: false,
            },
            AbilityDefinition::Cipher,
        ],
        completeness: Completeness::known_wrong(
            "modeled as an unconditional double tap. Oracle is 'you MAY tap OR untap' — the untap \
             mode and the optionality are both dropped. PB-OS10 (2026-07-19) added \
             TargetPermanentDistinctFrom and wired it onto the second target slot, so 'another \
             target permanent' distinctness IS now enforced (the same permanent can no longer be \
             chosen for both targets). The card remains known_wrong because (i) 'tap OR untap' \
             player choice is unmodeled (always taps), and (ii) the 'you MAY' optionality is \
             dropped for both instances. No Effect::Choose is introduced for these — that would \
             be a non-interactive gated stub, barred from Complete.",
        ),
        ..Default::default()
    }
}
