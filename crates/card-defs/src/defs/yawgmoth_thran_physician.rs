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
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Cleric"],
        ),
        oracle_text: "Protection from Humans\nPay 1 life, Sacrifice another creature: Put a -1/-1 counter on up to one target creature and draw a card.\n{B}{B}, Discard a card: Proliferate.".to_string(),
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
                    Cost::Mana(ManaCost { black: 2, ..Default::default() }),
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
        completeness: Completeness::partial("'Sacrifice another creature' is NOT a gap — Cost::Sacrifice(TargetFilter) exists (card_definition.rs:1226, enforced abilities.rs:733). The real blocker is Cost::PayLife: replay_harness.rs:3774 has no ActivationCost representation for it and silently drops it, so 'Pay 1 life, Sacrifice another creature: ...' would resolve as a free sacrifice. Un-author until PayLife is representable in ActivationCost. Engine gap #2: Cost::Sacrifice does not exclude the source despite being documented as 'sacrifice another'."),
        ..Default::default()
    }
}
