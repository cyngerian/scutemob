// Constant Mists — {1}{G}, Instant
// Buyback — Sacrifice a land.
// Prevent all combat damage that would be dealt this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("constant-mists"),
        name: "Constant Mists".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Buyback—Sacrifice a land. (You may sacrifice a land in addition to any \
                      other costs as you cast this spell. If you do, put this card into your hand \
                      as it resolves.)\nPrevent all combat damage that would be dealt this turn."
            .to_string(),
        abilities: vec![
            // CR 702.27a: Buyback — sacrifice a land.
            // TODO: Buyback cost is "sacrifice a land" not mana. AbilityDefinition::Buyback
            // only accepts ManaCost. When Buyback supports non-mana costs, use sacrifice-land.
            AbilityDefinition::Buyback {
                cost: ManaCost::default(),
            },
            AbilityDefinition::Spell {
                effect: Effect::PreventAllCombatDamage,
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        completeness: Completeness::known_wrong(
            "Buyback's real cost is 'sacrifice a land' but AbilityDefinition::Buyback accepts \
             only ManaCost (card_definition.rs:540), so the def ships ManaCost::default() = a \
             FREE {0} buyback (casting.rs:5060 pays it verbatim). This makes Constant Mists a \
             repeatable free fog. Prefer dropping the Buyback ability until non-mana buyback \
             costs are supported.",
        ),
        ..Default::default()
    }
}
