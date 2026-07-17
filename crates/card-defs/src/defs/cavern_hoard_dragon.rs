// Cavern-Hoard Dragon — {7}{R}{R}, Creature — Dragon 6/6
// Flying, trample, haste
// This spell costs {X} less to cast, where X is the greatest number of artifacts an
// opponent controls.
// Whenever this creature deals combat damage to a player, you create a Treasure token for
// each artifact that player controls.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cavern-hoard-dragon"),
        name: "Cavern-Hoard Dragon".to_string(),
        mana_cost: Some(ManaCost {
            generic: 7,
            red: 2,
            ..Default::default()
        }),
        types: creature_types(&["Dragon"]),
        oracle_text: "This spell costs {X} less to cast, where X is the greatest number of \
                      artifacts an opponent controls.\nFlying, trample, haste\nWhenever this \
                      creature deals combat damage to a player, you create a Treasure token for \
                      each artifact that player controls."
            .to_string(),
        power: Some(6),
        toughness: Some(6),
        // CR 601.2f: Costs {X} less where X is the greatest number of artifacts an
        // opponent controls (max over all opponents, not sum).
        self_cost_reduction: Some(SelfCostReduction::MaxOpponentPermanents {
            filter: TargetFilter {
                has_card_type: Some(CardType::Artifact),
                ..Default::default()
            },
            per: 1,
        }),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // CR 510.2 / 608.2h: "Create a Treasure token for each artifact that player
            // controls." PlayerTarget::DamagedPlayer resolves to the player the trigger
            // fired for (ctx.damaged_player).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        count: EffectAmount::PermanentCount {
                            filter: TargetFilter {
                                has_card_type: Some(CardType::Artifact),
                                controller: TargetController::DamagedPlayer,
                                ..Default::default()
                            },
                            controller: PlayerTarget::DamagedPlayer,
                        },
                        ..treasure_token_spec(1)
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
