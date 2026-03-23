// Elderfang Venom
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elderfang-venom"),
        name: "Elderfang Venom".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Attacking Elves you control have deathtouch.
Whenever an Elf you control dies, each opponent loses 1 life and you gain 1 life.".to_string(),
        abilities: vec![
            // TODO: DSL gap — "Attacking Elves you control have deathtouch."
            // EffectFilter::AttackingCreaturesYouControlWithSubtype does not exist.
            // TODO: DSL gap — "Whenever an Elf you control dies" trigger with
            // controller + subtype filter on WheneverCreatureDies.
        ],
        ..Default::default()
    }
}
