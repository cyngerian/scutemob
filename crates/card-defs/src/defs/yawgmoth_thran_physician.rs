// Yawgmoth, Thran Physician — {2}{B}{B}, Legendary Creature — Human Cleric 2/4
// Protection from Humans
// Pay 1 life, Sacrifice another creature: Put a -1/-1 counter on up to one target
// creature and draw a card.
// {B}{B}, Discard a card: Proliferate.
//
// TODO: "Sacrifice another creature" — Cost::SacrificeOtherCreature not in DSL.
//   Leaving the sacrifice ability with TODO.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("yawgmoth-thran-physician"),
        name: "Yawgmoth, Thran Physician".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Cleric"],
        ),
        oracle_text: "Protection from Humans\nPay 1 life, Sacrifice another creature: Put a -1/-1 \
                      counter on up to one target creature and draw a card.\n{B}{B}, Discard a \
                      card: Proliferate."
            .to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(
                ProtectionQuality::FromSubType(SubType("Human".to_string())),
            )),
            // TODO: "Pay 1 life, Sacrifice another creature" — Cost lacks
            //   SacrificeOtherCreature. Full ability not expressible.
            // {B}{B}, Discard a card: Proliferate.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        black: 2,
                        ..Default::default()
                    }),
                    Cost::DiscardCard,
                ]),
                effect: Effect::Proliferate,
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        completeness: Completeness::partial(
            "The 'Pay 1 life, Sacrifice another creature: Put a -1/-1 counter on up to one target \
             creature and draw a card' ability is unauthored. Two of its three former blockers \
             are gone: Cost::Sacrifice(TargetFilter) exists (card_definition.rs, enforced in \
             abilities.rs), and Cost::PayLife IS now representable — SR-36 (scutemob-92) added \
             ActivationCost::life_cost and a payment step to handle_activate_ability (CR \
             118.3/119.4), so it is no longer silently dropped. The surviving blocker is the \
             'another': handle_activate_ability's sacrifice_filter path validates zone, \
             controller, can't-be-sacrificed and the type filter, but never that sacrifice_target \
             != source — so 'sacrifice ANOTHER creature' would let Yawgmoth pay by sacrificing \
             itself (CR 602.2). Authoring it now would ship wrong game state, not a missing \
             clause.",
        ),
        ..Default::default()
    }
}
