// Krenko, Mob Boss — {2}{R}{R}, Legendary Creature — Goblin Warrior 3/3
// {T}: Create X 1/1 red Goblin creature tokens, where X is the number of Goblins you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("krenko-mob-boss"),
        name: "Krenko, Mob Boss".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Goblin", "Warrior"]),
        oracle_text: "{T}: Create X 1/1 red Goblin creature tokens, where X is the number of Goblins you control.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // TODO: {T}: Create X tokens where X = count of Goblins you control.
            // EffectAmount::CountCreaturesYouControlWithSubtype not in DSL.
        ],
        ..Default::default()
    }
}
