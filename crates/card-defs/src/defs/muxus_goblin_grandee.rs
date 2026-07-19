// Muxus, Goblin Grandee — {4}{R}{R}, Legendary Creature — Goblin Noble 4/4
// When Muxus enters, reveal the top six cards of your library. Put all Goblin creature
// cards with mana value 5 or less from among them onto the battlefield and the rest on
// the bottom of your library in a random order.
// Whenever Muxus attacks, it gets +1/+1 until end of turn for each other Goblin you
// control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("muxus-goblin-grandee"),
        name: "Muxus, Goblin Grandee".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            red: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Goblin", "Noble"],
        ),
        oracle_text: "When Muxus, Goblin Grandee enters, reveal the top six cards of your \
                      library. Put all Goblin creature cards with mana value 5 or less from among \
                      them onto the battlefield and the rest on the bottom of your library in a \
                      random order.\nWhenever Muxus, Goblin Grandee attacks, it gets +1/+1 until \
                      end of turn for each other Goblin you control."
            .to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            // PB-OS5: attack-half only. "Whenever Muxus attacks, it gets +1/+1 until end of
            // turn for each other Goblin you control." Count = other Goblins YOU CONTROL, NOT
            // just attacking ones (Muxus ruling 2020-06-23: determined at resolution) —
            // PermanentCount (you-control scope), not AttackingCreatureCount. exclude_self
            // excludes Muxus itself via ctx.source (WhenAttacks -> source is this creature).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBothDynamic {
                            amount: Box::new(EffectAmount::PermanentCount {
                                filter: TargetFilter {
                                    controller: TargetController::You,
                                    has_subtype: Some(SubType("Goblin".to_string())),
                                    exclude_self: true,
                                    ..Default::default()
                                },
                                controller: PlayerTarget::Controller,
                            }),
                            negate: false,
                        },
                        filter: EffectFilter::Source,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO(OOS-OS8-2): ETB half — "reveal the top six cards of your library. Put all
            // Goblin creature cards with mana value 5 or less from among them onto the
            // battlefield and the rest on the bottom of your library in a random order." This
            // is a put-MULTIPLE-by-filter dig (ALL matching Goblins, not at-most-one), which is
            // exactly the ALREADY-SHIPPED Effect::RevealAndRoute (PB-22 S3) — NOT
            // Effect::LookAtTopThenPlace (PB-OS8's new put-AT-MOST-ONE primitive). Authorable
            // via RevealAndRoute { count: 6, filter: { has_subtype: Goblin, has_card_type:
            // Creature, max_cmc: Some(5) }, matched_dest: Battlefield{tapped:false},
            // unmatched_dest: Library{Bottom} } — not authored in this PB (out of declared
            // scope; needs its own review pass).
        ],
        completeness: Completeness::partial(
            "PB-OS5: attack half authored (PermanentCount you-control + ModifyBothDynamic). ETB \
             reveal-top-six/put-Goblins-onto-battlefield is a put-MULTIPLE-by-filter dig — this \
             is Effect::RevealAndRoute (already shipped, PB-22 S3), NOT \
             Effect::LookAtTopThenPlace (PB-OS8, put-AT-MOST-ONE). Re-pointed from the earlier \
             misattributed OOS-EF10/PB-OS8 marker; filed as OOS-OS8-2 (authorable in a small \
             follow-up using RevealAndRoute).",
        ),
        ..Default::default()
    }
}
