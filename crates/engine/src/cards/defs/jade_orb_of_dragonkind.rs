// Jade Orb of Dragonkind — {2}{G}, Artifact
// {T}: Add {G}. When you spend this mana to cast a Dragon creature spell, it enters
// with an additional +1/+1 counter on it and gains hexproof until your next turn.
//
// TODO: "When you spend this mana to cast a Dragon creature spell" trigger — no
//   mana-spend trigger mechanism in DSL. Implementing only the base {T}: Add {G} ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("jade-orb-of-dragonkind"),
        name: "Jade Orb of Dragonkind".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}: Add {G}. When you spend this mana to cast a Dragon creature spell, it enters with an additional +1/+1 counter on it and gains hexproof until your next turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // TODO: mana-spend Dragon trigger (counter + hexproof) — DSL gap
        ],
        ..Default::default()
    }
}
