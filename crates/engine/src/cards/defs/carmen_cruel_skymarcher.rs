// Carmen, Cruel Skymarcher — {3}{W}{B}, Legendary Creature — Vampire Soldier 2/2
// Flying
// Whenever a player sacrifices a permanent, put a +1/+1 counter on Carmen and you gain 1 life.
// Whenever Carmen attacks, return up to one target permanent card with mana value less than
// or equal to Carmen's power from your graveyard to the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("carmen-cruel-skymarcher"),
        name: "Carmen, Cruel Skymarcher".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Vampire", "Soldier"],
        ),
        oracle_text: "Flying\nWhenever a player sacrifices a permanent, put a +1/+1 counter on Carmen, Cruel Skymarcher and you gain 1 life.\nWhenever Carmen attacks, return up to one target permanent card with mana value less than or equal to Carmen's power from your graveyard to the battlefield.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — "Whenever a player sacrifices a permanent" trigger condition.
            // TODO: DSL gap — attack trigger returning permanent from GY with dynamic MV filter.
        ],
        ..Default::default()
    }
}
