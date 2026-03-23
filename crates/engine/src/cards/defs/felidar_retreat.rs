// Felidar Retreat — {3}{W}, Enchantment
// Landfall — Whenever a land you control enters, choose one —
// • Create a 2/2 white Cat Beast creature token.
// • Put a +1/+1 counter on each creature you control. Those creatures gain
//   vigilance until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("felidar-retreat"),
        name: "Felidar Retreat".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, ..Default::default() }),
        types: full_types(&[], &[CardType::Enchantment], &[]),
        oracle_text: "Landfall — Whenever a land you control enters, choose one —\n\u{2022} Create a 2/2 white Cat Beast creature token.\n\u{2022} Put a +1/+1 counter on each creature you control. Those creatures gain vigilance until end of turn.".to_string(),
        abilities: vec![
            // TODO: Landfall modal trigger — mode 1 is a token, mode 2 is mass counters +
            // vigilance grant. Modal triggered abilities with Choose not fully supported.
            // Creating token for mode 1 only as approximation.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Cat Beast".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [
                            SubType("Cat".to_string()),
                            SubType("Beast".to_string()),
                        ]
                        .into_iter()
                        .collect(),
                        colors: [Color::White].into_iter().collect(),
                        power: 2,
                        toughness: 2,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
