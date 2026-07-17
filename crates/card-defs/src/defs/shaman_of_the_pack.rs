// Shaman of the Pack — {1}{B}{G}, Creature — Elf Shaman 3/2
// When this creature enters, target opponent loses life equal to the number of Elves
// you control.
//
// TODO: "loses life equal to the number of Elves you control" —
// EffectAmount::PermanentCountWithSubtype(SubType("Elf"), TargetController::You) not in DSL.
// A fixed amount would be wrong. W5: abilities empty.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shaman-of-the-pack"),
        name: "Shaman of the Pack".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 1,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Elf", "Shaman"]),
        oracle_text: "When this creature enters, target opponent loses life equal to the number \
                      of Elves you control."
            .to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // TODO: ETB trigger — "target opponent loses life equal to the number of Elves
            // you control." Needs EffectAmount::SubtypeCount("Elf", You) — not in DSL.
            // W5: omitted to avoid wrong game state.
        ],
        completeness: Completeness::partial(
            "ETB unimplemented but EXPRESSIBLE — the note's requested primitive shipped as \
             EffectAmount::PermanentCount{filter: {has_subtype: Elf, controller: You}, \
             controller: Controller} (card_definition.rs:2502). Needs authoring: \
             WhenEntersBattlefield + LoseLife{DeclaredTarget} + targets: [TargetPlayer]. Verify \
             'target opponent' (not any player) can be expressed before flipping to Complete.",
        ),
        ..Default::default()
    }
}
