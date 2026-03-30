// Dour Port-Mage — {1}{U}, Creature — Frog Wizard 1/3
// Whenever one or more other creatures you control leave the battlefield without
// dying, draw a card.
// {1}{U}, {T}: Return another target creature you control to its owner's hand.
//
// TODO: "Leave without dying" trigger not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dour-port-mage"),
        name: "Dour Port-Mage".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Frog", "Wizard"]),
        oracle_text: "Whenever one or more other creatures you control leave the battlefield without dying, draw a card.\n{1}{U}, {T}: Return another target creature you control to its owner's hand.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            // TODO: "Leave without dying" trigger not in DSL.
            // {1}{U}, {T}: Bounce another creature you control.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, blue: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Hand {
                        owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })),
                    },
                    controller_override: None,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreature],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
