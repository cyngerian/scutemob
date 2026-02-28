// 56. Think Twice — {1}{U}, Instant; draw a card. Flashback {2}{U}.
//
// CR 702.34a: "Flashback [cost]" means the card may be cast from its owner's
// graveyard by paying [cost] rather than its mana cost. If the flashback cost was
// paid, exile this card instead of putting it anywhere else when it leaves the stack.
//
// Two abilities encode flashback:
// 1. AbilityDefinition::Keyword(KeywordAbility::Flashback) — marker for quick
// presence-checking in casting.rs (zone validation, cost lookup).
// 2. AbilityDefinition::Flashback { cost } — stores the alternative cost {2}{U}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("think-twice"),
        name: "Think Twice".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Draw a card.\nFlashback {2}{U} (You may cast this card from your graveyard for its flashback cost. Then exile it.)".to_string(),
        abilities: vec![
            // CR 702.34a: Flashback marker — enables casting from graveyard in casting.rs.
            AbilityDefinition::Keyword(KeywordAbility::Flashback),
            // CR 702.34a: The flashback cost itself ({2}{U}).
            AbilityDefinition::Flashback {
                cost: ManaCost { generic: 2, blue: 1, ..Default::default() },
            },
            // The spell effect: draw a card for the controller.
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
