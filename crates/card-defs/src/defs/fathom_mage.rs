// Fathom Mage — {2}{G}{U}, Creature — Human Wizard 1/1
// Evolve
// Whenever a +1/+1 counter is put on this creature, you may draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fathom-mage"),
        name: "Fathom Mage".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Evolve\nWhenever a +1/+1 counter is put on Fathom Mage, you may draw a card."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Evolve),
            // CR 122.6 / 122.7: "Whenever a +1/+1 counter is put on this creature, you may
            // draw a card." Rulings (2013-01-24): triggers once per counter if multiple are
            // placed simultaneously, and for ANY +1/+1 counter (not just from Evolve).
            // TODO(optional-draw): oracle says "you MAY draw" — the DSL has no
            // optional-effect wrapper, so this is authored as a mandatory draw. This mirrors
            // the existing project convention for "you may draw a card" triggers (see
            // coastal_piracy.rs, aesi_tyrant_of_gyre_strait.rs). Fidelity nit only (matters
            // solely on an empty-library "would lose the game" edge case), not wrong game
            // state in the general case.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenCounterPlaced {
                    counter: Some(CounterType::PlusOnePlusOne),
                    filter: None,
                    on_self: true,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
                once_per_turn: false,
            },
        ],
        completeness: Completeness::partial(
            "(optional-draw): oracle says 'you MAY draw' — the DSL has no optional-effect \
             wrapper, so this is authored as a...",
        ),
        ..Default::default()
    }
}
