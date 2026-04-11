// Gauntlet of Power — {5}, Artifact
// As Gauntlet of Power enters, choose a color.
// Creatures of the chosen color get +1/+1.
// Whenever a basic land's ability causes a player to add mana of the chosen color,
// that player adds one additional mana of that color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gauntlet-of-power"),
        name: "Gauntlet of Power".to_string(),
        mana_cost: Some(ManaCost { generic: 5, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "As Gauntlet of Power enters, choose a color.\nCreatures of the chosen color get +1/+1.\nWhenever a basic land's ability causes a player to add mana of the chosen color, that player adds one additional mana of that color.".to_string(),
        abilities: vec![
            // CR 614.12 / CR 614.12a: "As this enters, choose a color."
            // Replacement effect — NOT a triggered ability (PB-X C1 lesson).
            // Default: White (arbitrary; deterministic fallback overrides at ETB time).
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::ChooseColor(Color::White),
                is_self: true,
                unless_condition: None,
            },
            // CR 613.1f / CR 105.1: Static +1/+1 to ALL creatures of the chosen color.
            // NOTE: Unlike Caged Sun, this is ALL creatures (any controller), not just "you control".
            // EffectFilter::AllCreaturesOfChosenColor reads chosen_color from this permanent.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::AllCreaturesOfChosenColor,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 106.6a / CR 105.1: "Whenever a basic land's ability causes a player to add
            // mana of the chosen color, that player adds one additional mana of that color."
            // Implemented as a replacement on ManaWouldBeProduced (see Caged Sun for rationale).
            // Note: Gauntlet of Power restricts to BASIC lands only (not any land like Caged Sun).
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::ManaWouldBeProduced {
                    // PlayerId(0) is a placeholder; bound to controller at ETB registration.
                    // TODO: Gauntlet of Power should fire for ANY player's basic land tap,
                    // not just the controller's. Full multi-player mana replacement dispatch
                    // deferred to PB-Q2 (requires per-player replacement registration loop).
                    // For now, only the controller's lands benefit.
                    controller: PlayerId(0),
                    // Only fires when the tapped basic land produces mana of the chosen color.
                    color_filter: Some(ChosenColorRef::SelfChosen),
                    // Applies to basic lands only (Gauntlet of Power: "a basic land's ability").
                    source_filter: Some(ReplacementManaSourceFilter::BasicLand),
                },
                modification: ReplacementModification::AddOneManaOfChosenColor,
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}
