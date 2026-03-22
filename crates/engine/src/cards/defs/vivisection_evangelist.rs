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
            // CR 603.1: Corrupted — When this creature enters, if an opponent has 3+
            // poison counters, destroy target creature or planeswalker an opponent controls.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                intervening_if: Some(Condition::OpponentHasPoisonCounters(3)),
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Creature, CardType::Planeswalker],
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],
            },
        ],
        power: Some(4),
        toughness: Some(4),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
    }
}
