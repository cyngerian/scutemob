// Lathril, Blade of the Elves — {2}{B}{G}, Legendary Creature — Elf Noble 2/3
// Menace. Combat damage trigger creates Elf Warrior tokens (number = damage dealt).
// Activated: {T}, tap 10 untapped Elves: each opponent loses 10, you gain 10.
//
// TODO: "{T}, Tap ten untapped Elves you control" — cost requiring tap of N other
//   specific-type permanents not expressible in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lathril-blade-of-the-elves"),
        name: "Lathril, Blade of the Elves".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elf", "Noble"]),
        oracle_text: "Menace (This creature can't be blocked except by two or more creatures.)\nWhenever Lathril deals combat damage to a player, create that many 1/1 green Elf Warrior creature tokens.\n{T}, Tap ten untapped Elves you control: Each opponent loses 10 life and you gain 10 life.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            // CR 510.3a: "Whenever Lathril deals combat damage to a player, create that many
            // 1/1 green Elf Warrior creature tokens." — self combat damage trigger with Repeat.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::Repeat {
                    effect: Box::new(Effect::CreateToken {
                        spec: TokenSpec {
                            name: "Elf Warrior".to_string(),
                            power: 1,
                            toughness: 1,
                            colors: [Color::Green].into_iter().collect(),
                            supertypes: OrdSet::new(),
                            card_types: [CardType::Creature].into_iter().collect(),
                            subtypes: [SubType("Elf".to_string()), SubType("Warrior".to_string())]
                                .into_iter()
                                .collect(),
                            keywords: OrdSet::new(),
                            count: 1,
                            tapped: false,
                            enters_attacking: false,
                            mana_color: None,
                            mana_abilities: vec![],
                            activated_abilities: vec![],
                        },
                    }),
                    count: EffectAmount::CombatDamageDealt,
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "{T}, Tap ten untapped Elves you control" — cost requiring tap of N other
            //   specific-type permanents not expressible in DSL.
        ],
        ..Default::default()
    }
}
