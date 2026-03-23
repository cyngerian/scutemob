// Sengir Autocrat — {3}{B}, Creature — Human 2/2
// ETB: create three 0/1 black Serf creature tokens.
// LTB: exile all Serf tokens.
//
// TODO: "When this creature leaves the battlefield, exile all Serf tokens" —
//   WhenLeavesBattlefield trigger not in TriggerCondition DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sengir-autocrat"),
        name: "Sengir Autocrat".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Human"]),
        oracle_text: "When this creature enters, create three 0/1 black Serf creature tokens.\nWhen this creature leaves the battlefield, exile all Serf tokens.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Serf".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Serf".to_string())].into_iter().collect(),
                        colors: [Color::Black].into_iter().collect(),
                        power: 0,
                        toughness: 1,
                        count: 3,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "When this creature leaves the battlefield, exile all Serf tokens" —
            //   WhenLeavesBattlefield trigger not in TriggerCondition DSL.
        ],
        ..Default::default()
    }
}
