// Connive // Concoct — Split card (Ikoria: Lair of Behemoths)
// Connive: {2}{U/B}{U/B} Sorcery — Gain control of target creature with power 2 or less.
// Concoct: {3}{U}{B} Sorcery — Surveil 3, then return a creature card from your graveyard
//          to the battlefield.
//
// CR 708.3: Split cards have two halves; each half may be cast separately from hand.
// Neither half has Aftermath or Fuse — both are cast from hand as regular spells.
//
//
// The Fuse AbilityDefinition is used here to encode Concoct's data (name, cost, types)
// even though this card is NOT a Fuse card — the KeywordAbility::Fuse marker is
// intentionally omitted so the engine does not offer fuse-casting.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("connive-concoct"),
        name: "Connive // Concoct".to_string(),
        // Connive half: {2}{U/B}{U/B}
        mana_cost: Some(ManaCost {
            generic: 2,
            hybrid: vec![
                HybridMana::ColorColor(ManaColor::Blue, ManaColor::Black),
                HybridMana::ColorColor(ManaColor::Blue, ManaColor::Black),
            ],
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Connive — Gain control of target creature with power 2 or less.\nConcoct — Surveil 3, then return a creature card from your graveyard to the battlefield.".to_string(),
        abilities: vec![
            // Connive (left half): Gain control of target creature with power 2 or less.
            // CR 613.1b: Indefinite control change (no duration — permanent).
            AbilityDefinition::Spell {
                effect: Effect::GainControl {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    duration: EffectDuration::Indefinite,
                },
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    max_power: Some(2),
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },

            // Concoct (right half): {3}{U}{B}. Surveil 3, then return a creature card from
            // your graveyard to the battlefield. (PB-10 Finding 7 fix: implemented with PB-10
            // primitives TargetCardInYourGraveyard + MoveZone.)
            // Encoded as Fuse for data representation; KeywordAbility::Fuse NOT added.
            AbilityDefinition::Fuse {
                name: "Concoct".to_string(),
                cost: ManaCost { generic: 3, blue: 1, black: 1, ..Default::default() },
                card_type: CardType::Sorcery,
                // CR 701.25: Surveil 3, then return a creature card from GY to battlefield.
                effect: Effect::Sequence(vec![
                    Effect::Surveil {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(3),
                    },
                    Effect::MoveZone {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        to: ZoneTarget::Battlefield { tapped: false },
                        // Concoct returns to battlefield "under your control" (implicit — your GY).
                        controller_override: None,
                    },
                ]),
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                })],
            },
        ],
        ..Default::default()
    }
}
