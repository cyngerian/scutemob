// 57. Faithless Looting — {R}, Sorcery; draw two cards, then discard two cards.
// Flashback {2}{R}.
//
// CR 702.34a: Sorcery with flashback — can be cast from graveyard at sorcery speed
// by paying {2}{R}. Exiled on any stack departure when cast via flashback.
//
// Note: Faithless Looting is banned in Modern but legal in Commander. It is a
// Commander staple and an ideal test card for sorcery-speed flashback.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("faithless-looting"),
        name: "Faithless Looting".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Draw two cards, then discard two cards.\nFlashback {2}{R} (You may cast this card from your graveyard for its flashback cost. Then exile it.)".to_string(),
        abilities: vec![
            // CR 702.34a: Flashback marker — enables casting from graveyard in casting.rs.
            AbilityDefinition::Keyword(KeywordAbility::Flashback),
            // CR 702.34a: The flashback cost itself ({2}{R}).
            AbilityDefinition::Flashback {
                cost: ManaCost { generic: 2, red: 1, ..Default::default() },
            },
            // The spell effect: draw 2 cards, then discard 2 cards.
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(2),
                    },
                    Effect::DiscardCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(2),
                    },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
