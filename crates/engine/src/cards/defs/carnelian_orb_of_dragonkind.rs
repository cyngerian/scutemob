// Carnelian Orb of Dragonkind — {2}{R}, Artifact
// {T}: Add {R}. If that mana is spent on a Dragon creature spell, it gains haste
//   until end of turn.
//
// TODO: Mana-spend Dragon trigger (haste grant) — no mana-spend trigger in DSL.
// Implementing only the base {T}: Add {R} ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("carnelian-orb-of-dragonkind"),
        name: "Carnelian Orb of Dragonkind".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}: Add {R}. If that mana is spent on a Dragon creature spell, it gains haste until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 1, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // TODO: mana-spend Dragon trigger (haste grant) — DSL gap
        ],
        ..Default::default()
    }
}
