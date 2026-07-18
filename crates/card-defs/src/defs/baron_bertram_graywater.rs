// Baron Bertram Graywater — {2}{W}{B}, Legendary Creature — Vampire Noble 3/4
// Whenever one or more tokens you control enter, create a 1/1 black Vampire Rogue creature
// token with lifelink. This ability triggers only once each turn.
// {1}{B}, Sacrifice another creature or artifact: Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("baron-bertram-graywater"),
        name: "Baron Bertram Graywater".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            black: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Vampire", "Noble"],
        ),
        oracle_text: "Whenever one or more tokens you control enter, create a 1/1 black Vampire \
                      Rogue creature token with lifelink. This ability triggers only once each \
                      turn.\n{1}{B}, Sacrifice another creature or artifact: Draw a card."
            .to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // ENGINE-BLOCKED: "Whenever one or more tokens you control enter, create a 1/1
            // black Vampire Rogue creature token with lifelink. This ability triggers only
            // once each turn." PB-AC1 shipped the `once_per_turn` limiter, but the trigger
            // needs a TOKEN-restricted ETB filter (not "any creature entering"). `TargetFilter
            // .is_token` is documented as a runtime `GameObject` field that is checked only in
            // the `combat_damage_filter` path (card_definition.rs) — it is NOT confirmed wired
            // into the creature-ETB trigger dispatch (unlike `has_subtype`/`is_nontoken`,
            // which PB-AC0 specifically wired through for Characteristics-level matching).
            // Approximating with an unfiltered `WheneverCreatureEntersBattlefield` (any
            // creature you control, not just tokens — nontoken Vampire Nobles, Soldiers, etc.
            // would wrongly trigger it) is a KI-1-class overbroad filter and would produce
            // wrong game state. Left fully blocked until `is_token` is verified wired for ETB
            // triggers (or an equivalent primitive ships).
            // {1}{B}, Sacrifice: Draw a card
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 1,
                        black: 1,
                        ..Default::default()
                    }),
                    Cost::Sacrifice(TargetFilter::default()),
                ]),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::partial(
            "needs-rewiring: TargetFilter.is_token IS wired for ETB trigger dispatch \
             (abilities.rs:6260; forwarded at replay_harness.rs:2701) — the old ENGINE-BLOCKED \
             note is stale. Author with TriggerCondition::WheneverPermanentEntersBattlefield { \
             filter: is_token + controller You } + once_per_turn: true. ALSO FIX: the {1}{B} \
             ability's Cost::Sacrifice(TargetFilter::default()) is overbroad — oracle requires \
             'another creature or artifact' (needs has_card_types [Creature, Artifact] + \
             exclude_self).",
        ),
        ..Default::default()
    }
}
