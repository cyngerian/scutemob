// Hands of Binding — {1}{U}, Sorcery
// Tap target creature an opponent controls. That creature doesn't untap during its
//   controller's next untap step.
// Cipher
//
// Freeze rider implemented via Effect::PreventNextUntap + GameObject.skip_untap_steps (PB-LS6).
// Note: TargetRequirement::TargetCreature does not restrict to "an opponent controls" —
// that is a pre-existing card-def gap unrelated to this batch; scope deferred.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hands-of-binding"),
        name: "Hands of Binding".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Tap target creature an opponent controls. That creature doesn't untap during its controller's next untap step.\nCipher (Then you may exile this spell card encoded on a creature you control. Whenever that creature deals combat damage to a player, its controller may cast a copy of the encoded card without paying its mana cost.)".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                // CR 702.27a: Tap target creature an opponent controls.
                // CR 502.3: that creature doesn't untap during its controller's next untap step.
                effect: Effect::Sequence(vec![
                    Effect::TapPermanent { target: EffectTarget::DeclaredTarget { index: 0 } },
                    Effect::PreventNextUntap { target: EffectTarget::DeclaredTarget { index: 0 } },
                ]),
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            },
            AbilityDefinition::Cipher,
        ],
        ..Default::default()
    }
}
