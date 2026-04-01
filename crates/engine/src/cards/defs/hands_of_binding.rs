// Hands of Binding — {1}{U}, Sorcery
// Tap target creature an opponent controls. That creature doesn't untap during its
//   controller's next untap step.
// Cipher
//
// TODO: "doesn't untap during its controller's next untap step" — no Effect::PreventNextUntap
//   or EffectDuration::UntilNextUntapStep(target_controller) in DSL. Tap effect is expressed;
//   the "skip untap" portion is omitted until a DoesntUntapNextTurn effect primitive exists.
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
                // TODO: "doesn't untap during its controller's next untap step" — DSL gap.
                //   No Effect::PreventNextUntap or EffectDuration::UntilNextUntapStep.
                effect: Effect::TapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            },
            AbilityDefinition::Cipher,
        ],
        ..Default::default()
    }
}
