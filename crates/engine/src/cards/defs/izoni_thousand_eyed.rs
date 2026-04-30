// Izoni, Thousand-Eyed — {2}{B}{B}{G}{G}, Legendary Creature — Elf Shaman 2/3
// Undergrowth — When Izoni enters, create a 1/1 black and green Insect creature token
// for each creature card in your graveyard.
// {B}{G}, Sacrifice another creature: You gain 1 life and draw a card.
//
// TODO (PB-TS OOS seed): The second activated ability "{B}{G}, Sacrifice another creature:
//   You gain 1 life and draw a card" requires ActivationCost::SacrificeAnotherCreature —
//   a "sacrifice another creature" cost variant distinct from sacrifice_self/sacrifice_filter.
//   Appended to memory/primitives/pb-retriage-CC.md as an OOS seed by PB-TS runner 2026-04-30.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("izoni-thousand-eyed"),
        name: "Izoni, Thousand-Eyed".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, green: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elf", "Shaman"]),
        oracle_text: "Undergrowth — When Izoni, Thousand-Eyed enters, create a 1/1 black and green Insect creature token for each creature card in your graveyard.\n{B}{G}, Sacrifice another creature: You gain 1 life and draw a card.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            // Undergrowth — When Izoni, Thousand-Eyed enters, create a 1/1 black and green
            // Insect creature token for each creature card in your graveyard.
            // CR 111.1 / CR 608.2h: CardCount resolved at ETB trigger resolution time.
            // zone: Graveyard { owner: Controller } counts creature cards in the controller's
            // graveyard (including any that died previously this turn).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Insect".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Insect".to_string())].into_iter().collect(),
                        colors: [Color::Black, Color::Green].into_iter().collect(),
                        supertypes: im::OrdSet::new(),
                        power: 1,
                        toughness: 1,
                        keywords: im::OrdSet::new(),
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
            // TODO (OOS — pb-retriage-CC.md seed added 2026-04-30):
            //   {B}{G}, Sacrifice another creature: You gain 1 life and draw a card.
            //   Blocked on ActivationCost variant for "sacrifice another creature"
            //   (sacrifice-other, distinct from sacrifice_self which sacrifices the source).
        ],
        ..Default::default()
    }
}
