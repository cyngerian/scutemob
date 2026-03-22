// Nullmage Shepherd — {3}{G} Creature — Elf Shaman 2/4
// Tap four untapped creatures you control: Destroy target artifact or enchantment.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nullmage-shepherd"),
        name: "Nullmage Shepherd".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Shaman"]),
        oracle_text:
            "Tap four untapped creatures you control: Destroy target artifact or enchantment."
                .to_string(),
        power: Some(2),
        toughness: Some(4),
        // TODO: The activated ability cost requires tapping N other permanents you control
        // (Cost::TapCreatures(4) or similar). No such Cost variant exists in the DSL.
        // When a "tap N untapped creatures you control" cost is added, implement as:
        //   AbilityDefinition::Activated {
        //       cost: Cost::TapCreatures(4),
        //       effect: Effect::DestroyPermanent { target: DeclaredTarget { index: 0 } },
        //       targets: vec![TargetPermanentWithFilter(artifact-or-enchantment)],
        //       ...
        //   }
        abilities: vec![],
        ..Default::default()
    }
}
