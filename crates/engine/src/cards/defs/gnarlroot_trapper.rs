// Gnarlroot Trapper — {B}, Creature — Elf Druid 1/1
// {T}, Pay 1 life: Add {G}. Spend this mana only to cast an Elf creature spell.
// {T}: Target attacking Elf you control gains deathtouch until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gnarlroot-trapper"),
        name: "Gnarlroot Trapper".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "{T}, Pay 1 life: Add {G}. Spend this mana only to cast an Elf creature spell.\n{T}: Target attacking Elf you control gains deathtouch until end of turn.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // {T}, Pay 1 life: Add {G}. Spend this mana only to cast an Elf creature spell.
            // Note: Pay 1 life cost is not fully expressible (Cost enum lacks PayLife variant).
            // Modeled as tap-only with the mana restriction applied.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaRestricted {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                    restriction: ManaRestriction::SubtypeOnly(SubType("Elf".to_string())),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: {T}: Target attacking Elf you control gains deathtouch until end of turn.
            // DSL gap: EffectTarget has no AttackingCreatureWithSubtype variant.
        ],
        ..Default::default()
    }
}
