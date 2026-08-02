// Necron Deathmark — {3}{B}{B}, Artifact Creature — Necron 5/3
// Flash
// Synaptic Disintegrator — When this creature enters, destroy up to one target creature
// and target player mills three cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("necron-deathmark"),
        name: "Necron Deathmark".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            black: 2,
            ..Default::default()
        }),
        types: full_types(&[], &[CardType::Artifact, CardType::Creature], &["Necron"]),
        oracle_text: "Flash\nSynaptic Disintegrator — When this creature enters, destroy up to \
                      one target creature and target player mills three cards."
            .to_string(),
        power: Some(5),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    // Destroy up to one target creature (any controller). Index 0 is an
                    // UpToN slot -- if not declared, DestroyPermanent resolves against an
                    // empty target list and is a no-op (CR 601.2c/608.2b).
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: false,
                    },
                    // Target player mills three cards.
                    Effect::MillCards {
                        player: PlayerTarget::DeclaredTarget { index: 1 },
                        count: EffectAmount::Fixed(3),
                    },
                ]),
                intervening_if: None,
                targets: vec![
                    TargetRequirement::UpToN {
                        count: 1,
                        inner: Box::new(TargetRequirement::TargetCreature),
                    },
                    TargetRequirement::TargetPlayer,
                ],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
