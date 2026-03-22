// Ogre Battledriver — {2}{R}{R}, Creature — Ogre Warrior 3/3
// Whenever another creature you control enters, that creature gets +2/+0 and gains haste
// until end of turn.
//
// TODO: DSL gap — ETBTriggerFilter (creature_only, controller_you, exclude_self) exists for
// filtering which ETBs trigger this, but EffectTarget::TriggeringCreature is not in DSL.
// The effect needs to pump/grant haste to the specific entering creature, not all creatures.
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
            // TODO: WheneverCreatureEntersBattlefield trigger — ETBTriggerFilter available,
            // but EffectTarget::TriggeringCreature not in DSL (+2/+0 + haste until EOT)
        ],
        ..Default::default()
    }
}
