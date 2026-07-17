// Boseiju, Who Endures — Legendary Land
// {T}: Add {G}.
// Channel — {1}{G}, Discard this card: Destroy target artifact, enchantment, or nonbasic
//   land an opponent controls. That player may search for land with basic land type,
//   put onto battlefield, shuffle. This ability costs {1} less per legendary creature you control.
use crate::cards::helpers::*;
pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("boseiju-who-endures"),
        name: "Boseiju, Who Endures".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {G}.\nChannel — {1}{G}, Discard this card: Destroy target \
                      artifact, enchantment, or nonbasic land an opponent controls. That player \
                      may search their library for a land card with a basic land type, put it \
                      onto the battlefield, then shuffle. This ability costs {1} less to activate \
                      for each legendary creature you control."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            // Channel — {1}{G}, Discard: Destroy target + opponent searches.
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
                        cant_be_regenerated: false,
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
                        also_search_graveyard: false,
                    },
                    Effect::Shuffle {
                        player: PlayerTarget::ControllerOf(Box::new(
                            EffectTarget::DeclaredTarget { index: 0 },
                        )),
                    },
                ]),
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Artifact, CardType::Enchantment, CardType::Land],
                    nonbasic: true,
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        // CR 602.2b + 601.2f: Channel ability (index 0) costs {1} less per legendary creature
        // controller has. The mana tap ability goes to mana_abilities, so the channel ability
        // is activated_ability index 0.
        activated_ability_cost_reductions: vec![(
            0,
            SelfActivatedCostReduction::PerPermanent {
                per: 1,
                filter: TargetFilter {
                    legendary: true,
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                },
                controller: PlayerTarget::Controller,
            },
        )],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
