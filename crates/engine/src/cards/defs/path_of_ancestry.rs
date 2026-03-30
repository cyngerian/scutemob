// Path of Ancestry — Land; enters tapped.
// "{T}: Add one mana of any color in your commander's color identity.
// When that mana is spent to cast a spell that shares a creature type with
// your commander, scry 1."
// TODO: DSL gap — creature-type comparison + conditional scry on mana spend
// not expressible. Modeled as ETB tapped + any-color mana (like Command Tower
// but tapped).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("path-of-ancestry"),
        name: "Path of Ancestry".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "Path of Ancestry enters the battlefield tapped.\n{T}: Add one mana of any color in your commander's color identity. When that mana is spent to cast a spell that shares a creature type with your commander, scry 1.".to_string(),
        abilities: vec![
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                // TODO: conditional scry on creature-type match
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
