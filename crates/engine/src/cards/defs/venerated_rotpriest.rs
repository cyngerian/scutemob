// Venerated Rotpriest — {G}, Creature — Phyrexian Druid 1/2
// Toxic 1
// Whenever a creature you control becomes the target of a spell, target opponent gets
//   a poison counter.
//
// TODO: "Whenever a creature you control becomes the target of a spell" —
//   TriggerCondition::WhenBecomesTargetByOpponent only fires when an OPPONENT's spell targets
//   the creature. Oracle says "a spell" with no controller restriction (includes your own spells).
//   DSL gap: need TriggerCondition::WhenCreatureYouControlBecomesTargetOfSpell.
//   The opponent-targets-it subset is expressible but the full oracle text is not.
//   Omitting trigger per W5 policy to avoid wrong game state.
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
            // TODO: Trigger fires when ANY spell targets a creature you control, but
            //   TriggerCondition::WhenBecomesTargetByOpponent is limited to opponent's spells only.
            //   Effect should give a poison counter to target opponent (no Effect::GivePlayerPoisonCounters).
            //   Both the trigger condition and the effect are DSL gaps. Omitted per W5 policy.
        ],
        ..Default::default()
    }
}
