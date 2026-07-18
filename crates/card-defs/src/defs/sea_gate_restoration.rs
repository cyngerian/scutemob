// Sea Gate Restoration // Sea Gate, Reborn — Modal DFC (Zendikar Rising)
// Front: {4}{U}{U}{U} Sorcery — Draw cards equal to the number of cards in your hand
//        plus one. You have no maximum hand size for the rest of the game.
// Back:  Sea Gate, Reborn — Land. As this land enters, you may pay 3 life. If you
//        don't, it enters tapped. {T}: Add {U}.
//
// CR 608.2h: the draw count is a fixed amount evaluated once at resolution, before any
//   draws happen — EffectAmount::Sum(HandSize, Fixed(1)) is resolved once by
//   Effect::DrawCards (resolve_amount is called before the draw loop), so this is not
//   the "count cards, then draw, recount" self-referential trap.
// CR 712.8a/712.8e: MDFC front/back face characteristics; CR 614.1c pay-life-or-tapped
//   replacement (mirrors Blood Crypt / Revitalizing Repast's back face).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sea-gate-restoration"),
        name: "Sea Gate Restoration".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            blue: 3,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Draw cards equal to the number of cards in your hand plus one. You have no \
                      maximum hand size for the rest of the game."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // CR 608.2h: count locked in at resolution (before any draws happen).
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Sum(
                        Box::new(EffectAmount::HandSize {
                            player: PlayerTarget::Controller,
                        }),
                        Box::new(EffectAmount::Fixed(1)),
                    ),
                },
                // CR 402.2: no maximum hand size for the rest of the game.
                Effect::SetNoMaximumHandSize {
                    player: PlayerTarget::Controller,
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Sea Gate, Reborn".to_string(),
            mana_cost: None,
            types: types(&[CardType::Land]),
            oracle_text: "As this land enters, you may pay 3 life. If you don't, it enters \
                          tapped.\n{T}: Add {U}."
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
                // {T}: Add {U}.
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 1, 0, 0, 0, 0),
                    },
                    timing_restriction: None,
                    targets: vec![],
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
        completeness: Completeness::Complete,
    }
}
