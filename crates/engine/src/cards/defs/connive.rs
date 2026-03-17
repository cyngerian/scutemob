// Connive // Concoct — Split card (Ikoria: Lair of Behemoths)
// Connive: {2}{U/B}{U/B} Sorcery — Gain control of target creature with power 2 or less.
// Concoct: {3}{U}{B} Sorcery — Surveil 3, then return a creature card from your graveyard
//          to the battlefield.
//
// CR 708.3: Split cards have two halves; each half may be cast separately from hand.
// Neither half has Aftermath or Fuse — both are cast from hand as regular spells.
//
// TODO (Finding 5): GainControl is not an Effect variant in the DSL. The Connive effect
//   ("Gain control of target creature with power 2 or less") cannot be expressed. A
//   GainControl { target, duration } Effect variant must be added to card_definition.rs
//   and resolution.rs before this card is functional.
//
// TODO (Finding 4/Concoct): Concoct's "return a creature card from your graveyard to the
//   battlefield" requires ReturnFromGraveyard or MoveZone from graveyard to battlefield
//   with a valid TargetCardInYourGraveyard target. The Fuse AbilityDefinition is used
//   here to encode Concoct's data (name, cost, types) even though this card is NOT a
//   Fuse card — the KeywordAbility::Fuse marker is intentionally omitted so the engine
//   does not offer fuse-casting. Surveil 3 is implemented; the return-creature effect is
//   Effect::Nothing until a ReturnFromGraveyard primitive exists.
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
            // CR 115.1: "target creature with power 2 or less" uses TargetCreatureWithFilter.
            // TODO: Effect::Nothing placeholder — replace with GainControl once DSL supports it.
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    max_power: Some(2),
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },

            // Concoct (right half): {3}{U}{B}. Surveil 3, then return a creature card from
            // your graveyard to the battlefield.
            // Encoded as Fuse for data representation; KeywordAbility::Fuse NOT added (not a
            // Fuse card). Until a ReturnFromGraveyard primitive exists, only Surveil 3 resolves.
            AbilityDefinition::Fuse {
                name: "Concoct".to_string(),
                cost: ManaCost { generic: 3, blue: 1, black: 1, ..Default::default() },
                card_type: CardType::Sorcery,
                // TODO: Sequence([Surveil { 3 }, ReturnFromGraveyard]) once primitive exists.
                effect: Effect::Surveil {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
                },
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
