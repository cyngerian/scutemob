// Breena, the Demagogue — {1}{W}{B}, Legendary Creature — Bird Warlock 1/3
// Flying
// Whenever a player attacks one of your opponents, if that opponent has more life than
// another of your opponents, that attacking player draws a card and you put two +1/+1
// counters on a creature you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("breena-the-demagogue"),
        name: "Breena, the Demagogue".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Bird", "Warlock"],
        ),
        oracle_text: "Flying\nWhenever a player attacks one of your opponents, if that opponent has more life than another of your opponents, that attacking player draws a card and you put two +1/+1 counters on a creature you control.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — complex multiplayer trigger: "whenever a player attacks
            // one of your opponents" with life comparison intervening-if. Multiple DSL
            // gaps: trigger condition, life comparison, attacker draws, controller places counters.
        ],
        ..Default::default()
    }
}
