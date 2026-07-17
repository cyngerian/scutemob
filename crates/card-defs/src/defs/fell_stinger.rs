// Fell Stinger — {2}{B}, Creature — Zombie Scorpion 3/2
// Deathtouch, Exploit; when exploits a creature: target player draws 2, loses 2
// TODO: exploit trigger draw/lose effect with target player requires targeted_trigger
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fell-stinger"),
        name: "Fell Stinger".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            ..Default::default()
        }),
        types: creature_types(&["Zombie", "Scorpion"]),
        oracle_text: "Deathtouch\nExploit (When this creature enters, you may sacrifice a creature.)\nWhen this creature exploits a creature, target player draws two cards and loses 2 life.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            AbilityDefinition::Keyword(KeywordAbility::Exploit),
            // TODO: Exploit trigger "when this creature exploits a creature" that draws 2
            // and makes a target player lose 2 life — the exploit trigger fires but requires
            // targeting a player for the draw/drain effect which is not in DSL
            // (targeted_trigger gap).
        ],
        completeness: Completeness::partial("Blocked on the secondary exploit trigger: no TriggerCondition for 'when this creature exploits a creature' (CR 702.110b), and Exploit's own trigger unconditionally declines the sacrifice at resolution.rs:3794 pending interactive choice (Command::ExploitCreature), so nothing is ever exploited. Targeted triggers are NOT the blocker — Triggered.targets exists and is in use."),
        ..Default::default()
    }
}
