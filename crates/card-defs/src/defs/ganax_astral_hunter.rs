// Ganax, Astral Hunter — {4}{R}, Legendary Creature — Dragon 3/4
// Flying
// Whenever Ganax or another Dragon you control enters, create a Treasure token.
// Choose a Background
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ganax-astral-hunter"),
        name: "Ganax, Astral Hunter".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            red: 1,
            ..Default::default()
        }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Dragon"]),
        oracle_text: "Flying\nWhenever Ganax or another Dragon you control enters, create a \
                      Treasure token.\nChoose a Background"
            .to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 603.2: "Whenever Ganax or another Dragon you control enters, create a
            // Treasure token." "Ganax OR another Dragon" = any Dragon you control, including
            // itself → exclude_self: false. PB-AC0: has_subtype: Dragon is now honored on
            // the creature-ETB path via triggering_creature_filter forwarding.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Dragon".to_string())),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(1),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::ChooseABackground),
        ],
        ..Default::default()
    }
}
