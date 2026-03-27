// Thousand-Faced Shadow — {U}, Creature — Human Ninja 1/1
// Ninjutsu {2}{U}{U}
// Flying
// When this creature enters from your hand, if it's attacking, create a token that's
// a copy of another target attacking creature. The token enters tapped and attacking.
// TODO: "enters from your hand, if it's attacking" — intervening-if condition on ETB
//   trigger not fully expressible. Currently triggers on any ETB (not just from hand).
//   Ninjutsu puts it onto the battlefield from hand tapped and attacking, which is the
//   primary use case. The intervening-if should also check "if it's attacking."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thousand-faced-shadow"),
        name: "Thousand-Faced Shadow".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Ninja"]),
        oracle_text: "Ninjutsu {2}{U}{U} ({2}{U}{U}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)\nFlying\nWhen this creature enters from your hand, if it's attacking, create a token that's a copy of another target attacking creature. The token enters tapped and attacking.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost {
                    generic: 2,
                    blue: 2,
                    ..Default::default()
                },
            },
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // ETB trigger: create a token copy of another target attacking creature,
            // tapped and attacking. CR 707.2 + CR 508.4.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateTokenCopy {
                    source: EffectTarget::DeclaredTarget { index: 0 },
                    enters_tapped_and_attacking: true,
                    except_not_legendary: false,
                    gains_haste: false,
                    delayed_action: None,
                },
                // TODO: TargetFilter lacks is_attacking and exclude_source constraints.
                // Oracle says "another target attacking creature" but filter allows
                // self-targeting and non-attacking creatures.
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                })],
                intervening_if: None,
            },
        ],
        power: Some(1),
        toughness: Some(1),
        ..Default::default()
    }
}
