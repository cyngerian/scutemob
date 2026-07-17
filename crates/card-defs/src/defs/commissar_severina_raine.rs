// Commissar Severina Raine — {1}{W}{B}, Legendary Creature — Human Soldier 2/2
// Whenever Commissar Severina Raine attacks, each opponent loses X life, where X
// is the number of other attacking creatures.
// {2}, Sacrifice another creature: You gain 2 life and draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("commissar-severina-raine"),
        name: "Commissar Severina Raine".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            black: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "Leading from the Front — Whenever Commissar Severina Raine attacks, each \
                      opponent loses X life, where X is the number of other attacking \
                      creatures.\nSummary Execution — {2}, Sacrifice another creature: You gain 2 \
                      life and draw a card."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "Each opponent loses X where X = other attacking creatures" —
            //   EffectAmount lacks AttackingCreatureCount. Attack trigger exists.
            // TODO: "Sacrifice another creature" — Cost::SacrificeOther not in DSL.
        ],
        completeness: Completeness::partial(
            "Attack trigger is authorable now — EffectAmount::AttackingCreatureCount { \
             controller: EachOpponent-scoped, filter: exclude_self } shipped in PB-AC3 \
             (card_definition.rs:2697) and its doc names this card. Still blocked: '{2}, \
             Sacrifice another creature' — Cost::Sacrifice(TargetFilter) drops `exclude_self` \
             when lowered to SacrificeFilter (replay_harness.rs:3743), so 'another' cannot be \
             enforced.",
        ),
        ..Default::default()
    }
}
