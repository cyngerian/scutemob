// Serra Ascendant — {W}, Creature — Human Monk 1/1
// Lifelink (Damage dealt by this creature also causes you to gain that much life.)
// As long as you have 30 or more life, this creature gets +5/+5 and has flying.
//
// Lifelink is implemented.
//
// TODO: "As long as you have 30 or more life, this creature gets +5/+5 and has flying" —
// conditional static ability. Condition::ControllerLifeAtLeast(30) EXISTS in the Condition enum
// and InterveningIf enum, but there is no EffectDuration variant for "while condition X holds".
// EffectDuration has: WhileSourceOnBattlefield, UntilEndOfTurn, Indefinite, WhilePaired.
// No "WhileCondition(Condition)" variant exists. DSL gap.
// Also, a +5/+5 PtModify + Flying Ability grant would require TWO ContinuousEffectDef entries
// under a single conditional static, which AbilityDefinition::Static does not support.
// Full implementation deferred until EffectDuration::WhileCondition is added to the layer system.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("serra-ascendant"),
        name: "Serra Ascendant".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Monk"]),
        oracle_text: "Lifelink (Damage dealt by this creature also causes you to gain that much life.)\nAs long as you have 30 or more life, this creature gets +5/+5 and has flying.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
        ],
        ..Default::default()
    }
}
