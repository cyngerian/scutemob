// Venerated Rotpriest — {G}, Creature — Phyrexian Druid 1/2
// Toxic 1
// Whenever a creature you control becomes the target of a spell, target opponent gets
//   a poison counter.
//
// PB-AC6 added TriggerCondition::WhenBecomesTarget { scope, by_opponent, include_abilities }
// which CAN now express the trigger condition itself: scope: Some(creature filter),
// by_opponent: false (oracle says "a spell", not "an opponent's spell" — any controller),
// include_abilities: false (oracle says "a spell", not "a spell or ability").
//
// ENGINE-BLOCKED: the trigger's EFFECT is still not expressible. Oracle text is
// "target opponent gets a poison counter" — this requires (a) the triggered ability to
// itself have a target (an opponent), and (b) an Effect variant that gives a poison
// counter directly to a target player. No such Effect variant exists in the DSL today
// (poison counters are currently only produced by Infect combat damage and by
// Effect::Proliferate — see ichor_rats.rs for the same gap on a different card). Per W5
// policy, a triggered ability with a correct condition but no way to apply its effect is
// omitted rather than approximated (e.g. faking it with Proliferate would be wrong game
// state). Toxic 1 (the creature's own combat-damage-to-poison keyword) is unaffected and
// fully implemented below.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("venerated-rotpriest"),
        name: "Venerated Rotpriest".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Druid"]),
        oracle_text: "Toxic 1 (Players dealt combat damage by this creature also get a poison counter.)\nWhenever a creature you control becomes the target of a spell, target opponent gets a poison counter.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Toxic(1)),
            // ENGINE-BLOCKED: see file header. Trigger condition is now expressible via
            // TriggerCondition::WhenBecomesTarget, but the "target opponent gets a poison
            // counter" effect has no DSL representation (no player-targeting poison-counter
            // Effect variant exists). Omitted rather than approximated.
        ],
        ..Default::default()
    }
}
