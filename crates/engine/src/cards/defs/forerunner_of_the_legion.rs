// Forerunner of the Legion — {2}{W}, Creature — Vampire Knight 2/2
// When this creature enters, you may search your library for a Vampire card, reveal it,
// then shuffle and put that card on top.
// Whenever another Vampire you control enters, target creature gets +1/+1 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("forerunner-of-the-legion"),
        name: "Forerunner of the Legion".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Knight"]),
        oracle_text: "When this creature enters, you may search your library for a Vampire card, reveal it, then shuffle and put that card on top.\nWhenever another Vampire you control enters, target creature gets +1/+1 until end of turn.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // ETB: search library for a Vampire, put on top.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::SearchLibrary {
                    filter: TargetFilter {
                        has_subtype: Some(SubType("Vampire".to_string())),
                        ..Default::default()
                    },
                    destination: ZoneTarget::Library {
                        owner: PlayerTarget::Controller,
                        position: LibraryPosition::Top,
                    },
                    reveal: true,
                    player: PlayerTarget::Controller,
                    also_search_graveyard: false,
                    shuffle_before_placing: true,
                },
                intervening_if: None,
                targets: vec![],
            },
            // Whenever another Vampire you control enters, target creature gets +1/+1 until EOT.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        has_subtype: Some(SubType("Vampire".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(1),
                        filter: EffectFilter::DeclaredTarget { index: 0 },
                        duration: EffectDuration::UntilEndOfTurn,
                    }),
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreature],
            },
        ],
        ..Default::default()
    }
}
