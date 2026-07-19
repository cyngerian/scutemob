// Disciple of Freyalise // Garden of Freyalise — Modal DFC (Dominaria)
// Front: {3}{G}{G}{G} Creature — Elf Druid 3/3. When this creature enters, you may
//        sacrifice another creature. If you do, you gain X life and draw X cards,
//        where X is that creature's power.
// Back:  Garden of Freyalise — Land. As this land enters, you may pay 3 life. If you
//        don't, it enters tapped. {T}: Add {G}.
//
// Both faces are fully implemented. PB-EF1 (scutemob-99) closed the "another creature"
// blocker (TargetFilter.exclude_self honored on the optional-cost sacrifice path).
// PB-OS2 (EF-EF1-A) closed the surviving blocker: the optional-cost path now threads
// the sacrificed creature's layer-resolved LKI into ctx.sacrificed_creature_lki before
// `then` runs, so EffectAmount::PowerOfSacrificedCreature resolves correctly (CR 608.2h/i).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("disciple-of-freyalise"),
        name: "Disciple of Freyalise".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 3,
            ..Default::default()
        }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "When this creature enters, you may sacrifice another creature. If you do, \
                      you gain X life and draw X cards, where X is that creature's power."
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            // CR 118.12 / 109.1: "you may sacrifice another creature. If you do, gain
            // X life and draw X cards, X = its power." PB-OS2 (EF-EF1-A) makes the
            // optional-cost path capture the sacrificed creature's layer-resolved LKI
            // power (CR 608.2h/608.2i).
            effect: Effect::MayPayThenEffect {
                cost: Cost::Sacrifice(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    exclude_self: true, // "another creature" (CR 109.1)
                    ..Default::default()
                }),
                payer: PlayerTarget::Controller,
                then: Box::new(Effect::Sequence(vec![
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::PowerOfSacrificedCreature,
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::PowerOfSacrificedCreature,
                    },
                ])),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Garden of Freyalise".to_string(),
            mana_cost: None,
            types: types(&[CardType::Land]),
            oracle_text: "As this land enters, you may pay 3 life. If you don't, it enters \
                          tapped.\n{T}: Add {G}."
                .to_string(),
            power: None,
            toughness: None,
            abilities: vec![
                // CR 614.1c: pay-3-life-or-enters-tapped self-replacement.
                AbilityDefinition::Replacement {
                    trigger: ReplacementTrigger::WouldEnterBattlefield {
                        filter: ObjectFilter::Any,
                    },
                    modification: ReplacementModification::EntersTappedUnlessPayLife(3),
                    is_self: true,
                    unless_condition: None,
                },
                // {T}: Add {G}.
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
        completeness: Completeness::Complete,
    }
}
