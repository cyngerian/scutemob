// Lathril, Blade of the Elves — {2}{B}{G}, Legendary Creature — Elf Noble 2/3
// Menace. Combat damage trigger creates Elf Warrior tokens (number = damage dealt).
// Activated: {T}, tap 10 untapped Elves: each opponent loses 10, you gain 10.
//
// TODO: "Whenever Lathril deals combat damage to a player, create that many tokens" —
//   per-creature combat damage trigger with variable amount not in DSL.
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
            // TODO: combat damage trigger (per-creature damage amount variable)
            // TODO: {T} + tap ten Elves activated ability (cost requires N other permanents)
        ],
        ..Default::default()
    }
}
