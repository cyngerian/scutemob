// Growing Rites of Itlimoc // Itlimoc, Cradle of the Sun — {2}{G} DFC Legendary
// Enchantment // Legendary Land (Transform)
// Front: When Growing Rites of Itlimoc enters, look at the top four cards of your library.
//        You may reveal a creature card from among them and put it into your hand. Put the
//        rest on the bottom of your library in any order.
//        At the beginning of your end step, if you control four or more creatures,
//        transform Growing Rites of Itlimoc.
// Back:  Itlimoc, Cradle of the Sun — {T}: Add {G}. {T}: Add {G} for each creature you
//        control.
//
// DSL gap: the ETB "look at top four cards, may reveal a creature and put it into your
// hand, rest to bottom in any order" effect is not modeled -- no primitive expresses
// selective-draw-from-a-look (only Scry/Surveil exist, which reorder rather than
// selectively draw a matching card to hand). The end-step transform-if-4-creatures
// clause IS wired via TransformSelf (PB-EF5). PB-OS6 evaluated closing this gap
// (sub-primitive (d)) and deferred it to PB-OS8's general
// Effect::LookAtTopThenPlace family (count 4, filter creature, destination hand,
// rest_to bottom, optional) -- see pb-plan-OS6.md "Scope decisions (d)".
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("growing-rites-of-itlimoc-itlimoc-cradle-of-the-sun"),
        name: "Growing Rites of Itlimoc".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Enchantment]),
        oracle_text: "When Growing Rites of Itlimoc enters, look at the top four cards of your \
                      library. You may reveal a creature card from among them and put it into \
                      your hand. Put the rest on the bottom of your library in any order.\nAt the \
                      beginning of your end step, if you control four or more creatures, \
                      transform Growing Rites of Itlimoc."
            .to_string(),
        power: None,
        toughness: None,
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Transform),
            // CR 701.27a/f: "At the beginning of your end step, if you control four or
            // more creatures, transform Growing Rites of Itlimoc." (PB-EF5)
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
                effect: Effect::TransformSelf,
                intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
                    count: 4,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    },
                }),
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // TODO: ETB "look at top 4, may reveal a creature card and put it into your
            //   hand, rest to bottom in any order" -- no primitive for a selective look-
            //   and-take exists (only Scry/Surveil, which reorder rather than draw a
            //   matching card). Closing primitive: PB-OS8 / Effect::LookAtTopThenPlace.
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Itlimoc, Cradle of the Sun".to_string(),
            mana_cost: None,
            types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
            oracle_text: "{T}: Add {G}.\n{T}: Add {G} for each creature you control.".to_string(),
            power: None,
            toughness: None,
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
                    modes: None,
                },
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddManaScaled {
                        player: PlayerTarget::Controller,
                        color: ManaColor::Green,
                        count: EffectAmount::PermanentCount {
                            filter: TargetFilter {
                                has_card_type: Some(CardType::Creature),
                                ..Default::default()
                            },
                            controller: PlayerTarget::Controller,
                        },
                    },
                    timing_restriction: None,
                    targets: vec![],
                    activation_condition: None,
                    activation_zone: None,
                    once_per_turn: false,
                    modes: None,
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
        completeness: Completeness::partial(
            "ETB 'look at top four cards, may reveal a creature card and put it into your hand, \
             rest to bottom in any order' not modeled -- no primitive expresses a selective \
             look-and-take (only Scry/Surveil exist, which reorder rather than selectively draw a \
             matching card to hand). Closing primitive: PB-OS8 / Effect::LookAtTopThenPlace \
             (count 4, filter creature, destination hand, rest_to bottom, optional) -- deferred \
             out of PB-OS6, see pb-plan-OS6.md 'Scope decisions (d)'. The end-step \
             transform-if-4-creatures clause IS implemented via TransformSelf (PB-EF5); the back \
             face's two mana abilities are fully implemented.",
        ),
    }
}
