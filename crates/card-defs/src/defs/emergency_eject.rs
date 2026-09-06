// Emergency Eject — {2}{W}, Instant
// Destroy target nonland permanent. Its controller creates a Lander token.
// (It's an artifact with "{2}, {T}, Sacrifice this token: Search your library for a basic
// land card, put it onto the battlefield tapped, then shuffle.")
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("emergency-eject"),
        name: "Emergency Eject".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target nonland permanent. Its controller creates a Lander token. \
                      (It's an artifact with \"{2}, {T}, Sacrifice this token: Search your \
                      library for a basic land card, put it onto the battlefield tapped, then \
                      shuffle.\")"
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Lander".to_string(),
                        power: 0,
                        toughness: 0,
                        colors: OrdSet::new(),
                        supertypes: OrdSet::new(),
                        card_types: [CardType::Artifact].into_iter().collect(),
                        subtypes: [SubType("Lander".to_string())].into_iter().collect(),
                        keywords: OrdSet::new(),
                        count: EffectAmount::Fixed(1),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![ActivatedAbility {
                            targets: vec![],
                            cost: ActivationCost {
                                requires_tap: true,
                                mana_cost: Some(ManaCost {
                                    generic: 2,
                                    ..ManaCost::default()
                                }),
                                sacrifice_self: true,
                                ..Default::default()
                            },
                            description: "{2}, {T}, Sacrifice this token: Search your library for \
                                          a basic land card, put it onto the battlefield tapped, \
                                          then shuffle."
                                .to_string(),
                            effect: Some(Effect::Sequence(vec![
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
                            ])),
                            sorcery_speed: false,
                            activation_condition: None,
                            activation_zone: None,
                            once_per_turn: false,
                            modes: None,
                        }],
                        // CR 608.2h: "its controller" is the destroyed permanent's
                        // controller, not the caster (resolves via LKI after CR 400.7).
                        recipient: PlayerTarget::ControllerOf(Box::new(
                            EffectTarget::DeclaredTarget { index: 0 },
                        )),
                        ..Default::default()
                    },
                },
            ]),
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                non_land: true,
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
