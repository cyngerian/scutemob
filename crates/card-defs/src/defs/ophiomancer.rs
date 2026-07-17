// Ophiomancer — {2}{B}, Creature — Human Shaman 2/2
// At the beginning of each upkeep, if you control no Snakes, create a 1/1 black Snake
// creature token with deathtouch.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ophiomancer"),
        name: "Ophiomancer".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Shaman"]),
        oracle_text: "At the beginning of each upkeep, if you control no Snakes, create a 1/1 black Snake creature token with deathtouch.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "If you control no Snakes" intervening-if — Condition lacks
            //   "you control no permanents with subtype X" variant.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfEachUpkeep,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Snake".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Snake".to_string())].into_iter().collect(),
                        colors: [Color::Black].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: EffectAmount::Fixed(1),
                        supertypes: imbl::OrdSet::new(),
                        keywords: [KeywordAbility::Deathtouch].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::partial("Blocker stale: set intervening_if: Some(Condition::Not(Box::new(Condition::ControlCreatureWithSubtype(SubType(\"Snake\".into()))))) — Option<Condition> is the def-level type and check_condition handles Not + ControlCreatureWithSubtype (dispatched at resolution.rs:2073). Until rewired this def creates a Snake every upkeep regardless of board state; marker should be known_wrong, not partial."),
        ..Default::default()
    }
}
