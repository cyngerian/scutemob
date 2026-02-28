// 19b. Lonely Sandbar — Land — Island; enters tapped; cycling {U}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lonely-sandbar"),
        name: "Lonely Sandbar".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Island"]),
        oracle_text: "This land enters tapped.\n{T}: Add {U}.\nCycling {U} ({U}, Discard this card: Draw a card.)".to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement effect — this permanent enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
            },
            // {T}: Add {U} (Island subtype grants this implicitly, but explicit here).
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
            },
            // CR 702.29: Cycling {U} — pay {U} and discard this card to draw a card.
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost { blue: 1, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
