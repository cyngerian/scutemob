// Elderfang Venom — {2}{B}{G}, Enchantment
// Attacking Elves you control have deathtouch.
// Whenever an Elf you control dies, each opponent loses 1 life and you gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elderfang-venom"),
        name: "Elderfang Venom".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Attacking Elves you control have deathtouch.\nWhenever an Elf you control dies, each opponent loses 1 life and you gain 1 life.".to_string(),
        abilities: vec![
            // CR 613.1f / CR 611.3a: "Attacking Elves you control have deathtouch."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Deathtouch),
                    filter: EffectFilter::AttackingCreaturesYouControlWithSubtype(
                        SubType("Elf".to_string()),
                    ),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: "Whenever an Elf you control dies, each opponent loses 1 life and you gain
            // 1 life." Blocked on PB-26: WheneverCreatureDies needs subtype filter (Elf only).
        ],
        ..Default::default()
    }
}
