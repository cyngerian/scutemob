// Temporal Trespass — {8}{U}{U}{U}, Sorcery
// Delve (Each card you exile from your graveyard while casting this spell
// pays for {1}.)
// Take an extra turn after this one. Exile Temporal Trespass.
//
// CR 702.66: Delve — exile cards from graveyard to pay for generic mana.
// CR 500.7: "Take an extra turn after this one."
// self_exile_on_resolution: "Exile Temporal Trespass." — the spell exiles itself
// after resolving instead of going to the graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("temporal-trespass"),
        name: "Temporal Trespass".to_string(),
        mana_cost: Some(ManaCost { generic: 8, blue: 3, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Delve (Each card you exile from your graveyard while casting this spell pays for {1}.)\nTake an extra turn after this one. Exile Temporal Trespass.".to_string(),
        abilities: vec![
            // CR 702.66: Delve keyword marker — enables graveyard exile during casting.
            AbilityDefinition::Keyword(KeywordAbility::Delve),
            // CR 500.7: Take an extra turn after this one.
            AbilityDefinition::Spell {
                effect: Effect::ExtraTurn {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        // "Exile Temporal Trespass." — self-exile on successful resolution.
        self_exile_on_resolution: true,
        ..Default::default()
    }
}
