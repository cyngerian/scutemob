// Ogre Battledriver — {2}{R}{R}, Creature — Ogre Warrior 3/3
// Whenever another creature you control enters, that creature gets +2/+0 and gains
// haste until end of turn.
//
// ENGINE-BLOCKED: "that creature gets +2/+0 and gains haste until end of turn" requires
// Effect::ApplyContinuousEffect with EffectFilter::TriggeringCreature (the entering
// creature). EffectTarget::TriggeringCreature exists for point effects (AddCounter,
// DealDamage), but ContinuousEffectDef.filter is EffectFilter — and EffectFilter has
// no TriggeringCreature variant. The trigger itself is now correct (filter + exclude_self
// both supported), but the effect cannot target the entering creature for a continuous
// P/T + keyword grant.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ogre-battledriver"),
        name: "Ogre Battledriver".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 2, ..Default::default() }),
        types: creature_types(&["Ogre", "Warrior"]),
        oracle_text: "Whenever another creature you control enters, that creature gets +2/+0 and gains haste until end of turn. (It can attack and {T} this turn.)".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // ENGINE-BLOCKED: trigger fires correctly (WheneverCreatureEntersBattlefield,
            // exclude_self: true), but Effect::ApplyContinuousEffect cannot target the
            // TriggeringCreature — EffectFilter::TriggeringCreature does not exist in the DSL.
            // Omitted per W5 policy: wrong game state (e.g. buffing self or all creatures)
            // is worse than no implementation.
        ],
        ..Default::default()
    }
}
