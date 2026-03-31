// Dread Return — {2}{B}{B} Sorcery
// Return target creature card from your graveyard to the battlefield.
// Flashback—Sacrifice three creatures.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dread-return"),
        name: "Dread Return".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Return target creature card from your graveyard to the battlefield.\nFlashback—Sacrifice three creatures. (You may cast this card from your graveyard for its flashback cost. Then exile it.)".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Battlefield { tapped: false },
                    controller_override: Some(PlayerTarget::Controller),
                },
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },
            // Flashback keyword marker.
            AbilityDefinition::Keyword(KeywordAbility::Flashback),
            // TODO: Flashback cost is "Sacrifice three creatures" — not a mana cost. The
            // AltCastAbility DSL only accepts ManaCost for flashback. A sacrifice-N-creatures
            // flashback cost requires a new Cost variant in AltCastAbility. The flashback
            // ability is omitted — card can only be cast from hand (no flashback from graveyard).
        ],
        ..Default::default()
    }
}
