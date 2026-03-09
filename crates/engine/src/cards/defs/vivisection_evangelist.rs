// Vivisection Evangelist — {3WB}, Creature — Phyrexian Cleric 4/4
// Vigilance; Corrupted ETB: if an opponent has 3+ poison counters,
// destroy target creature or planeswalker an opponent controls.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vivisection-evangelist"),
        name: "Vivisection Evangelist".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Cleric"]),
        oracle_text: "Vigilance\nCorrupted — When this creature enters, if an opponent has three or more poison counters, destroy target creature or planeswalker an opponent controls.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DestroyPermanent {
                    // TODO: TargetRequirement has no TargetCreatureOrPlaneswalker variant;
                    // using TargetPermanentWithFilter(controller=Opponent) as the closest
                    // approximation. Slightly broader than oracle (can target any opponent
                    // permanent, not just creatures/planeswalkers).
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                intervening_if: Some(Condition::OpponentHasPoisonCounters(3)),
            },
        ],
        // TODO: targets field on Triggered is not currently supported in AbilityDefinition::Triggered.
        // The target (creature or planeswalker an opponent controls) cannot be declared at
        // trigger resolution time without a targets vec on Triggered. Mark as a known gap.
        power: Some(4),
        toughness: Some(4),
        back_face: None,
    }
}
