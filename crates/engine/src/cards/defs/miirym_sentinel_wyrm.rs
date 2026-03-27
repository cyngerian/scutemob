// Miirym, Sentinel Wyrm — {3}{G}{U}{R}, Legendary Creature — Dragon Spirit 6/6
// Flying, ward {2}
// Whenever another nontoken Dragon you control enters, create a token that's a copy of
// it, except the token isn't legendary.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("miirym-sentinel-wyrm"),
        name: "Miirym, Sentinel Wyrm".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, blue: 1, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Spirit"],
        ),
        oracle_text: "Flying, ward {2}\nWhenever another nontoken Dragon you control enters, create a token that's a copy of it, except the token isn't legendary.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Ward(2)),
            // Whenever another nontoken Dragon you control enters, create a token that's
            // a copy of it, except the token isn't legendary.
            // TODO: TargetFilter lacks "nontoken" restriction; currently fires on token Dragons too.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Dragon".to_string())),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::CreateTokenCopy {
                    source: EffectTarget::TriggeringCreature,
                    enters_tapped_and_attacking: false,
                    except_not_legendary: true,
                    gains_haste: false,
                    delayed_action: None,
                },
                // TODO: Condition should exclude self (another creature), but no SourceIsNotSelf condition exists.
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
