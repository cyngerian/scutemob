// Fecundity — {2}{G}, Enchantment
// Whenever a creature dies, that creature's controller may draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fecundity"),
        name: "Fecundity".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature dies, that creature's controller may draw a card."
            .to_string(),
        abilities: vec![
            // TODO: "that creature's controller" — needs ControllerOf(dying creature)
            //   as PlayerTarget. WheneverCreatureDies gives controller draw as approximation.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: None,
                    exclude_self: false,
                    nontoken_only: false,
                    filter: None,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::partial(
            "draws for Fecundity's controller on every creature death; oracle says 'that \
             creature's controller may draw'. Rewire to \
             PlayerTarget::ControllerOf(EffectTarget::TriggeringCreature) (wired: \
             abilities.rs:4446 -> 7758 -> resolution.rs:2109 -> effects/mod.rs:6333). Residual \
             after rewire: 'may' is inexpressible, and ControllerOf reads the graveyard object \
             whose controller was reset to owner (CR 603.10a).",
        ),
        ..Default::default()
    }
}
