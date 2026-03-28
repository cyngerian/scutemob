// Mirage Phalanx — {4}{R}{R}, Creature — Human Soldier 4/4
// Soulbond. While paired: each of those creatures has combat trigger creating copy token
// with haste, exiled at end of combat.
//
// TODO: "Create a token that's a copy of this creature, except it has haste and loses
//   soulbond" — copy-token creation with modifications not in DSL (no CreateTokenCopy
//   variant in Effect that operates on Source/paired target).
// TODO: "Exile it at end of combat" — delayed triggered exile after copy creation
//   not expressible in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mirage-phalanx"),
        name: "Mirage Phalanx".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 2, ..Default::default() }),
        types: creature_types(&["Human", "Soldier"]),
        oracle_text: "Soulbond (You may pair this creature with another unpaired creature when either enters. They remain paired for as long as you control both of them.)\nAs long as Mirage Phalanx is paired with another creature, each of those creatures has \"At the beginning of combat on your turn, create a token that's a copy of this creature, except it has haste and loses soulbond. Exile it at end of combat.\"".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Soulbond),
            // At the beginning of combat on your turn, create a token that's a copy of
            // this creature, except it has haste (and loses soulbond — not expressible).
            // Exile it at end of combat.
            // TODO: The soulbond grant aspect (each of those creatures has this trigger
            //   while paired) is not expressible in the DSL; only the direct ability is.
            // TODO: "loses soulbond" on copy token not expressible.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfCombat,
                effect: Effect::CreateTokenCopy {
                    source: EffectTarget::Source,
                    enters_tapped_and_attacking: false,
                    except_not_legendary: false,
                    gains_haste: true,
                    delayed_action: Some((
                        crate::state::stubs::DelayedTriggerTiming::AtEndOfCombat,
                        crate::state::stubs::DelayedTriggerAction::ExileObject,
                    )),
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
