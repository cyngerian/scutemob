// Call the Spirit Dragons — {W}{U}{B}{R}{G}, Enchantment
// Dragons you control have indestructible.
// At the beginning of your upkeep, for each color, put a +1/+1 counter on a Dragon you
// control of that color. If you put +1/+1 counters on five Dragons this way, you win the game.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("call-the-spirit-dragons"),
        name: "Call the Spirit Dragons".to_string(),
        mana_cost: Some(ManaCost { white: 1, blue: 1, black: 1, red: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Dragons you control have indestructible.\nAt the beginning of your upkeep, for each color, put a +1/+1 counter on a Dragon you control of that color. If you put +1/+1 counters on five Dragons this way, you win the game.".to_string(),
        abilities: vec![
            // Layer 6: Dragons you control have indestructible.
            // Using OtherCreaturesYouControlWithSubtype — source is an enchantment, not a Dragon.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Indestructible),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Dragon".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // TODO: DSL gap — upkeep trigger with per-color counter placement on Dragons
            // + win condition check. Multiple DSL gaps.
        ],
        ..Default::default()
    }
}
