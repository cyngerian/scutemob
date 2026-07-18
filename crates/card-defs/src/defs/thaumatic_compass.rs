// Thaumatic Compass // Spires of Orazca — DFC Artifact // Land (Transform)
// Front: {2}, {T}: Search your library for a basic land card, reveal it, put it into your hand, then shuffle.
//         At the beginning of your end step, if you control seven or more lands, transform Thaumatic Compass.
// Back:  Spires of Orazca — Land
//         {T}: Add {C}.
//         {T}: Untap target attacking creature an opponent controls and remove it from combat.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thaumatic-compass-spires-of-orazca"),
        name: "Thaumatic Compass".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{2}, {T}: Search your library for a basic land card, reveal it, put it into \
                      your hand, then shuffle.\nAt the beginning of your end step, if you control \
                      seven or more lands, transform Thaumatic Compass."
            .to_string(),
        power: None,
        toughness: None,
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 2,
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: basic_land_filter(),
                    reveal: true,
                    destination: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            AbilityDefinition::Keyword(KeywordAbility::Transform),
            // CR 701.27a/f: "At the beginning of your end step, if you control seven or
            // more lands, transform Thaumatic Compass." (PB-EF5)
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
                effect: Effect::TransformSelf,
                intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
                    count: 7,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                }),
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Spires of Orazca".to_string(),
            mana_cost: None,
            types: types(&[CardType::Land]),
            oracle_text: "{T}: Add {C}.\n{T}: Untap target attacking creature an opponent \
                          controls and remove it from combat."
                .to_string(),
            power: None,
            toughness: None,
            abilities: vec![
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 0, 0, 0, 0, 1),
                    },
                    timing_restriction: None,
                    targets: vec![],
                    activation_condition: None,
                    activation_zone: None,
                    once_per_turn: false,
                },
                // "{T}: Untap target attacking creature an opponent controls and remove it
                // from combat." The untap + attacking-opponent-creature target are expressible;
                // the "remove it from combat" clause has NO effect primitive (only Regenerate
                // references removal-from-combat internally) — so this model OMITS that clause
                // and the def stays `partial`. See OOS-EF5-4(g). CR 508 / 701.21.
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::UntapPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    timing_restriction: None,
                    targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                        controller: TargetController::Opponent,
                        is_attacking: true,
                        ..Default::default()
                    })],
                    activation_condition: None,
                    activation_zone: None,
                    once_per_turn: false,
                },
            ],
            color_indicator: None,
        }),
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        // Front (search + TransformSelf end-step trigger) is fully modeled. The back face,
        // Spires of Orazca, is NOT: "{T}: Untap target attacking creature an opponent controls
        // and remove it from combat" needs a remove-from-combat effect primitive that does not
        // exist (only Regenerate references combat removal internally). The modeled untap omits
        // that clause, so the def is truthfully `partial`, not Complete — see OOS-EF5-4(g).
        completeness: Completeness::partial(
            "Spires of Orazca back face: '{T}: Untap target attacking creature an opponent \
             controls and remove it from combat' lacks a remove-from-combat effect primitive; \
             modeled untap omits the combat-removal clause (OOS-EF5-4g). Front TransformSelf \
             (PB-EF5) is complete.",
        ),
    }
}
