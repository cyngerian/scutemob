// Gnarlroot Trapper — {B}, Creature — Elf Druid 1/1
// {T}, Pay 1 life: Add {G}. Spend this mana only to cast an Elf creature spell.
// {T}: Target attacking Elf you control gains deathtouch until end of turn.
// TODO: DSL gap — mana ability with a spending restriction ("only to cast Elf creature spells")
// is not expressible. The second activated ability targets an attacking creature with a
// subtype filter; EffectTarget has no AttackingCreatureWithSubtype variant.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gnarlroot-trapper"),
        name: "Gnarlroot Trapper".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "{T}, Pay 1 life: Add {G}. Spend this mana only to cast an Elf creature spell.\n{T}: Target attacking Elf you control gains deathtouch until end of turn. (Any amount of damage it deals to a creature is enough to destroy it.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        // TODO: {T}, Pay 1 life: Add {G} (restricted mana — Elf creature spells only)
        // TODO: {T}: attacking Elf gains deathtouch until end of turn
        ..Default::default()
    }
}
