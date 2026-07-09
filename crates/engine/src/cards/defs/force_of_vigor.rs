// Force of Vigor — {2}{G}{G}, Instant
// If it's not your turn, you may exile a green card from your hand rather than
// pay this spell's mana cost.
// Destroy up to two target artifacts and/or enchantments.
// PB-AC5: Pitch alt cost implemented via AltCostKind::Pitch.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("force-of-vigor"),
        name: "Force of Vigor".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If it's not your turn, you may exile a green card from your hand rather than pay this spell's mana cost.\nDestroy up to two target artifacts and/or enchantments.".to_string(),
        abilities: vec![
            // CR 118.9: Pitch — exile a green card from hand instead of the mana cost,
            // only legal when it's not the caster's turn.
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Pitch,
                cost: ManaCost::default(),
                details: Some(AltCastDetails::Pitch {
                    costs: vec![Cost::ExileFromHand { color: Color::Green }],
                    opponents_turn_only: true,
                }),
            },
            AbilityDefinition::Spell {
                // CR 601.2c / 115.1b: "Destroy up to two target artifacts and/or enchantments."
                effect: Effect::Sequence(vec![
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: false,
                    },
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                        cant_be_regenerated: false,
                    },
                ]),
                targets: vec![TargetRequirement::UpToN {
                    count: 2,
                    inner: Box::new(TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                        has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                        ..Default::default()
                    })),
                }],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
