// Miirym, Sentinel Wyrm — {3}{G}{U}{R}, Legendary Creature — Dragon Spirit 6/6
// Flying, ward {2}
// Whenever another nontoken Dragon you control enters, create a token that's a copy of
// it, except the token isn't legendary.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("miirym-sentinel-wyrm"),
        name: "Miirym, Sentinel Wyrm".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 1,
            blue: 1,
            red: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Spirit"],
        ),
        oracle_text: "Flying, ward {2}\nWhenever another nontoken Dragon you control enters, \
                      create a token that's a copy of it, except the token isn't legendary."
            .to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Ward(2)),
            // CR 603.2: "Whenever another nontoken Dragon you control enters, create a
            // token that's a copy of it, except the token isn't legendary."
            // PB-AC0: has_subtype Dragon and is_nontoken are now honored on the
            // creature-ETB path via triggering_creature_filter forwarding.
            // exclude_self: true handles "another" (Miirym's own ETB does not fire it).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Dragon".to_string())),
                        controller: TargetController::You,
                        is_nontoken: true,
                        ..Default::default()
                    }),
                    exclude_self: true,
                },
                effect: Effect::CreateTokenCopy {
                    source: EffectTarget::TriggeringCreature,
                    enters_tapped_and_attacking: false,
                    except_not_legendary: true,
                    gains_haste: false,
                    delayed_action: None,
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
