// Helm of the Host — {4}, Legendary Artifact — Equipment
// At the beginning of combat on your turn, create a token that's a copy of
// equipped creature, except the token isn't legendary. That token gains haste.
// Equip {5}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("helm-of-the-host"),
        name: "Helm of the Host".to_string(),
        mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Artifact], &["Equipment"]),
        oracle_text: "At the beginning of combat on your turn, create a token that's a copy of equipped creature, except the token isn't legendary. That token gains haste.\nEquip {5}".to_string(),
        abilities: vec![
            // TODO: AtBeginningOfCombat trigger creates a copy of equipped creature (not legendary +
            // haste). DSL gap: no EffectTarget::EquippedCreature for CreateTokenCopy source.
            // Equip {5}: attach this Equipment to target creature you control.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 5, ..Default::default() }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                targets: vec![TargetRequirement::TargetCreature],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
