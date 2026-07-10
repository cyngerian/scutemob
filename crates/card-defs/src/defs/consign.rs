// Consign // Oblivion — {1}{U} Instant // {4}{B} Sorcery (Aftermath)
// Consign: "Return target nonland permanent to its owner's hand."
// Oblivion: "Target player discards two cards." (Aftermath — cast from GY only, then exile.)
// CR 702.127: Aftermath keyword.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("consign"),
        name: "Consign // Oblivion".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Return target nonland permanent to its owner's hand.\n\
            Oblivion — Aftermath (Cast this spell only from your graveyard. Then exile it.) \
            Target player discards two cards."
            .to_string(),
        abilities: vec![
            // CR 702.127a: Aftermath keyword marker.
            AbilityDefinition::Keyword(KeywordAbility::Aftermath),
            // Consign half: bounce target nonland permanent.
            AbilityDefinition::Spell {
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Hand {
                        owner: PlayerTarget::OwnerOf(Box::new(
                            EffectTarget::DeclaredTarget { index: 0 },
                        )),
                    },
                    controller_override: None,
                },
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    non_land: true,
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },
            // Oblivion half: {4}{B} Sorcery — target player discards two cards.
            AbilityDefinition::Aftermath {
                name: "Oblivion".to_string(),
                cost: ManaCost { generic: 4, black: 1, ..Default::default() },
                card_type: CardType::Sorcery,
                effect: Effect::DiscardCards {
                    player: PlayerTarget::DeclaredTarget { index: 0 },
                    count: EffectAmount::Fixed(2),
                },
                targets: vec![TargetRequirement::TargetPlayer],
            },
        ],
        ..Default::default()
    }
}
