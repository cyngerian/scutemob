// Encroaching Dragonstorm — {3}{G}, Enchantment
// When this enchantment enters, search your library for up to two basic land cards,
// put them onto the battlefield tapped, then shuffle.
// When a Dragon you control enters, return this enchantment to its owner's hand.
//
// Second trigger authored below (CR 603.2 / CR 400): `Effect::ReturnToHand` claimed missing
// is the wrong primitive — `Effect::MoveZone { target, to: ZoneTarget::Hand { owner }, .. }`
// is the DSL's "return to hand" shape (card_definition.rs:1647-1656), precedent
// `hullbreaker_horror.rs:63-71`. `owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::Source))`
// is used explicitly (not `controller_override: None`'s documented owner default, which only
// governs the CONTROLLER of a battlefield-destination object, not which player's hand a
// Hand-destination move lands in — CR 108.3/CR 400.7 correctness should not depend on control
// changes leaving the enchantment mis-homed).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("encroaching-dragonstorm"),
        name: "Encroaching Dragonstorm".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "When this enchantment enters, search your library for up to two basic land \
                      cards, put them onto the battlefield tapped, then shuffle.\nWhen a Dragon \
                      you control enters, return this enchantment to its owner's hand."
            .to_string(),
        abilities: vec![
            // When this enchantment enters, search for up to two basic lands tapped, then shuffle.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: true },
                        shuffle_before_placing: false,
                        also_search_graveyard: false,
                    },
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: true },
                        shuffle_before_placing: false,
                        also_search_graveyard: false,
                    },
                    Effect::Shuffle {
                        player: PlayerTarget::Controller,
                    },
                ]),
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // CR 603.2 / CR 400: "When a Dragon you control enters, return this enchantment to
            // its owner's hand." `exclude_self: false` is correct — Encroaching Dragonstorm is
            // an Enchantment, not a Dragon, so it can never be its own trigger source here.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Dragon".to_string())),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::MoveZone {
                    target: EffectTarget::Source,
                    to: ZoneTarget::Hand {
                        owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::Source)),
                    },
                    controller_override: None,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::partial(
            "Second trigger (Dragon ETB -> return to owner's hand, CR 603.2) is authored. Still \
             blocked on the ETB clause: 'up to two basic land cards' is modeled as two separate \
             unconditional SearchLibrary calls (lines above), which is not one search and cannot \
             express 'up to' — Effect::SearchLibrary has no count field.",
        ),
        ..Default::default()
    }
}
