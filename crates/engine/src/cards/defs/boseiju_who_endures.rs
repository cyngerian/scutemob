// Boseiju, Who Endures — Legendary Land
// {T}: Add {G}.
// Channel — {1}{G}, Discard this card: Destroy target artifact, enchantment, or nonbasic
//   land an opponent controls. That player may search for land with basic land type,
//   put onto battlefield, shuffle. This ability costs {1} less per legendary creature you control.
// Partial: target filter uses controller:Opponent. "artifact OR enchantment OR nonbasic land"
// requires expressing multi-type OR with a nonbasic-land exclusion — DSL gap (TargetFilter
// can express opponent control via TargetController::Opponent but cannot combine multiple
// card type alternatives with a "nonbasic" land variant in a single filter).
// TODO: Target filter should restrict to "artifact, enchantment, or nonbasic land" —
//   needs has_card_types OR semantics combined with non_basic land filter. DSL gap.
// TODO: Cost reduction — {1} less per legendary creature you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("boseiju-who-endures"),
        name: "Boseiju, Who Endures".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {G}.\nChannel — {1}{G}, Discard this card: Destroy target artifact, enchantment, or nonbasic land an opponent controls. That player may search their library for a land card with a basic land type, put it onto the battlefield, then shuffle. This ability costs {1} less to activate for each legendary creature you control.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // Channel — {1}{G}, Discard: Destroy target + opponent searches.
            // Target filter restricts to opponent-controlled permanents (partial).
            // TODO: Target filter needs "artifact, enchantment, or nonbasic land" type restriction.
            // TODO: Cost reduction per legendary creature.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 1,
                        green: 1,
                        ..Default::default()
                    }),
                    Cost::DiscardSelf,
                ]),
                effect: Effect::Sequence(vec![
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::SearchLibrary {
                        player: PlayerTarget::ControllerOf(Box::new(
                            EffectTarget::DeclaredTarget { index: 0 },
                        )),
                        // CR 305.8: "land card with a basic land type" means any land with
                        // Plains/Island/Swamp/Mountain/Forest subtype — includes nonbasic lands
                        // like shock lands. NOT the same as basic: true (Basic supertype).
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Land),
                            has_subtypes: vec![
                                SubType("Plains".to_string()),
                                SubType("Island".to_string()),
                                SubType("Swamp".to_string()),
                                SubType("Mountain".to_string()),
                                SubType("Forest".to_string()),
                            ],
                            ..Default::default()
                        },
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: false },
                        shuffle_before_placing: false,
                    },
                    Effect::Shuffle {
                        player: PlayerTarget::ControllerOf(Box::new(
                            EffectTarget::DeclaredTarget { index: 0 },
                        )),
                    },
                ]),
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],
            },
        ],
        ..Default::default()
    }
}
