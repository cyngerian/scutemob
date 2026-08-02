// 19b. Lonely Sandbar — Land; enters tapped; cycling {U}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lonely-sandbar"),
        name: "Lonely Sandbar".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\n{T}: Add {U}.\nCycling {U} ({U}, Discard this \
                      card: Draw a card.)"
            .to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement effect — this permanent enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // {T}: Add {U} (no Island subtype on the printed card; ability is explicit).
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // CR 702.29: Cycling {U} — pay {U} and discard this card to draw a card.
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost {
                    blue: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}
