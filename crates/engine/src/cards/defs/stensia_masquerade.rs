// Stensia Masquerade — {2}{R}, Enchantment
// Attacking creatures you control have first strike.
// Whenever a Vampire you control deals combat damage to a player, put a +1/+1 counter on it.
// Madness {2}{R}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("stensia-masquerade"),
        name: "Stensia Masquerade".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Attacking creatures you control have first strike.\nWhenever a Vampire you control deals combat damage to a player, put a +1/+1 counter on it.\nMadness {2}{R}".to_string(),
        abilities: vec![
            // CR 613.1f / CR 611.3a: "Attacking creatures you control have first strike."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::FirstStrike),
                    filter: EffectFilter::AttackingCreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: "Whenever a Vampire you control deals combat damage to a player, put a
            // +1/+1 counter on it." Blocked on PB-26: subtype-filtered combat damage trigger
            // not in DSL.
            // TODO: Madness {2}{R} — AltCostKind::Madness not in DSL.
        ],
        ..Default::default()
    }
}
