// Bootleggers' Stash — {5}{G}, Artifact
// Lands you control have "{T}: Create a Treasure token."
//
// Authored 2026-04-12 (PB-N stale-TODO sweep): unblocked by PB-S
// (LayerModification::AddActivatedAbility + EffectFilter::LandsYouControl).
// First card def to use AddActivatedAbility on a filtered grant.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bootleggers-stash"),
        name: "Bootleggers' Stash".to_string(),
        mana_cost: Some(ManaCost { generic: 5, green: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Lands you control have \"{T}: Create a Treasure token.\"".to_string(),
        abilities: vec![
            // CR 613.1f / CR 605.1b: Layer 6 static ability — grants a non-mana
            // activated ability ({T}: Create a Treasure token) to each land the
            // controller of this artifact controls. The granted ability is appended
            // to the land's activated_abilities; existing land abilities (e.g. its
            // own mana ability) are preserved per CR 613.5.
            //
            // Note: creating a Treasure token is NOT a mana ability (CR 605.1b —
            // it doesn't add mana directly), so this uses AddActivatedAbility, not
            // AddManaAbility. The token itself, once on the battlefield, has its
            // own {T}, Sacrifice: Add one mana ability.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddActivatedAbility(Box::new(
                        ActivatedAbility {
                            cost: ActivationCost {
                                requires_tap: true,
                                ..Default::default()
                            },
                            description: "{T}: Create a Treasure token.".to_string(),
                            effect: Some(Effect::CreateToken {
                                spec: treasure_token_spec(1),
                            }),
                            sorcery_speed: false,
                            targets: vec![],
                            activation_condition: None,
                            activation_zone: None,
                            once_per_turn: false,
                        },
                    )),
                    filter: EffectFilter::LandsYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
