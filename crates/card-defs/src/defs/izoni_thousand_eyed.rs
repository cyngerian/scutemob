// Izoni, Thousand-Eyed — {2}{B}{B}{G}{G}, Legendary Creature — Elf Shaman 2/3
// Undergrowth — When Izoni enters, create a 1/1 black and green Insect creature token
// for each creature card in your graveyard.
// {B}{G}, Sacrifice another creature: You gain 1 life and draw a card.
//
// PB-EF1 (scutemob-99, closes OOS-TS-2): the "sacrifice another creature" cost is now
// expressible — Cost::Sacrifice(TargetFilter.exclude_self) lowers onto
// ActivationCost.sacrifice_exclude_self and is enforced in handle_activate_ability
// (CR 109.1). Both abilities implemented; Complete.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("izoni-thousand-eyed"),
        name: "Izoni, Thousand-Eyed".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 2,
            green: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elf", "Shaman"],
        ),
        oracle_text: "Undergrowth — When Izoni, Thousand-Eyed enters, create a 1/1 black and \
                      green Insect creature token for each creature card in your \
                      graveyard.\n{B}{G}, Sacrifice another creature: You gain 1 life and draw a \
                      card."
            .to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            // Undergrowth — When Izoni, Thousand-Eyed enters, create a 1/1 black and green
            // Insect creature token for each creature card in your graveyard.
            // CR 111.1 / CR 608.2h: CardCount resolved at ETB trigger resolution time.
            // zone: Graveyard { owner: Controller } counts creature cards in the controller's
            // graveyard (including any that died previously this turn).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Insect".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Insect".to_string())].into_iter().collect(),
                        colors: [Color::Black, Color::Green].into_iter().collect(),
                        supertypes: imbl::OrdSet::new(),
                        power: 1,
                        toughness: 1,
                        keywords: imbl::OrdSet::new(),
                        count: EffectAmount::CardCount {
                            zone: ZoneTarget::Graveyard {
                                owner: PlayerTarget::Controller,
                            },
                            player: PlayerTarget::Controller,
                            filter: Some(TargetFilter {
                                has_card_type: Some(CardType::Creature),
                                ..Default::default()
                            }),
                        },
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // {B}{G}, Sacrifice another creature: You gain 1 life and draw a card.
            // PB-EF1 (CR 109.1): "Sacrifice ANOTHER creature" — the sacrifice cost carries
            // TargetFilter.exclude_self, lowered onto ActivationCost.sacrifice_exclude_self
            // and enforced in handle_activate_ability, so Izoni cannot pay by sacrificing
            // itself.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        black: 1,
                        green: 1,
                        ..Default::default()
                    }),
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::You,
                        exclude_self: true,
                        ..Default::default()
                    }),
                ]),
                effect: Effect::Sequence(vec![
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        ..Default::default()
    }
}
