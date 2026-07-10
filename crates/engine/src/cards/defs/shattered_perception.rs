// Shattered Perception — {2}{R}, Sorcery
// Discard all the cards in your hand, then draw that many cards.
// Flashback {5}{R}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shattered-perception"),
        name: "Shattered Perception".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Discard all the cards in your hand, then draw that many cards.\nFlashback {5}{R} (You may cast this card from your graveyard for its flashback cost. Then exile it.)".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flashback),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Flashback,
                details: None,
                cost: ManaCost { generic: 5, red: 1, ..Default::default() },
            },
            AbilityDefinition::Spell {
                // CR 701.9 / 121.1: discard the whole hand, then draw that many cards.
                // `Effect::WheelHand` snapshots the hand size before discarding.
                effect: Effect::WheelHand {
                    player: PlayerTarget::Controller,
                    disposal: WheelDisposal::Discard,
                    draw: WheelDraw::ThatMany,
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
