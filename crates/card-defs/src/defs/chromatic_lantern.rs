// Chromatic Lantern — {3} Artifact
// Lands you control have "{T}: Add one mana of any color."
// {T}: Add one mana of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("chromatic-lantern"),
        name: "Chromatic Lantern".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Lands you control have \"{T}: Add one mana of any color.\"\n{T}: Add one \
                      mana of any color."
            .to_string(),
        abilities: vec![
            // Self tap-for-any-color activated mana ability.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // CR 613.1f: Layer 6 static ability — grants mana ability to each land
            // you control. Additive per 2018-10-05 ruling: lands keep all existing
            // abilities and also gain this tap-for-any-color ability.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddManaAbility(ManaAbility {
                        produces: Default::default(),
                        requires_tap: true,
                        sacrifice_self: false,
                        any_color: true,
                        damage_to_controller: 0,
                        ..Default::default()
                    }),
                    filter: EffectFilter::LandsYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        // PB-EF12 (EF-W-PB2-3): un-marked. `any_color: true` mana abilities (both the
        // self ability and the granted land ability) now resolve to a real chosen
        // colour (CR 111.10a/605.3b) via `Command::TapForMana.chosen_color`, not
        // ManaColor::Colorless (was SF-11 / SR-37).
        ..Default::default()
    }
}
