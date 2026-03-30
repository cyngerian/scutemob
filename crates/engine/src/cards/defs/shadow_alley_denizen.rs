// Shadow Alley Denizen — {B}, Creature — Vampire Rogue 1/1
// Whenever another black creature you control enters, target creature gains
// intimidate until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shadow-alley-denizen"),
        name: "Shadow Alley Denizen".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Rogue"]),
        oracle_text: "Whenever another black creature you control enters, target creature gains intimidate until end of turn. (It can't be blocked except by artifact creatures and/or creatures that share a color with it.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // "Whenever another black creature you control enters, target creature gains
            // intimidate until end of turn."
            // WheneverCreatureEntersBattlefield with color filter (black) + controller_you.
            // ETBTriggerFilter is built from this in replay_harness enrichment.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        colors: Some([Color::Black].iter().copied().collect()),
                        ..Default::default()
                    }),
                },
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::Intimidate),
                        filter: EffectFilter::DeclaredTarget { index: 0 },
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
