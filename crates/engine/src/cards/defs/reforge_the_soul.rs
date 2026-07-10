// Reforge the Soul — {3}{R}{R}, Sorcery
// Each player discards their hand, then draws seven cards.
// Miracle {1}{R}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reforge-the-soul"),
        name: "Reforge the Soul".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Each player discards their hand, then draws seven cards.\nMiracle {1}{R} (You may cast this card for its miracle cost when you draw it if it's the first card you drew this turn.)".to_string(),
        abilities: vec![
            // TODO: Miracle {1}{R} — KeywordAbility::Miracle not yet implemented.
            // When Miracle is added, include AltCastAbility with Miracle cost.
            AbilityDefinition::Spell {
                // PB-AC9 (CR 701.9 / 121.1): each player discards their ENTIRE hand, then
                // draws seven cards. `Effect::WheelHand` fixes the previous approximation
                // (`DiscardCards{Fixed(7)}`, which discarded exactly 7 regardless of hand
                // size instead of "their hand").
                effect: Effect::WheelHand {
                    player: PlayerTarget::EachPlayer,
                    disposal: WheelDisposal::Discard,
                    draw: WheelDraw::Fixed(7),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
