// Charismatic Conqueror — {1}{W}, Creature — Vampire Soldier 2/2
// Vigilance. Trigger: whenever opponent's artifact or creature enters untapped, they
// may tap it; if they don't, create a 1/1 white Vampire token with lifelink.
//
// TODO: "Whenever an artifact or creature an opponent controls enters untapped" —
//   no TriggerCondition for opponent's permanents entering untapped; would need
//   WheneverCreatureEntersBattlefield with opponent-controller filter AND a
//   tapped-status check, neither of which is available in DSL.
// TODO: "they may tap that permanent. If they don't..." — conditional opponent-choice
//   branch not expressible in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("charismatic-conqueror"),
        name: "Charismatic Conqueror".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Soldier"]),
        oracle_text: "Vigilance\nWhenever an artifact or creature an opponent controls enters untapped, they may tap that permanent. If they don't, you create a 1/1 white Vampire creature token with lifelink.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // TODO: trigger on opponent's artifact/creature entering untapped with
            //   opponent choice branch — not in DSL.
        ],
        ..Default::default()
    }
}
