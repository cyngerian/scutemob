// Reckless Bushwhacker — {2}{R}, Creature — Goblin Warrior Ally 2/1; Surge {1}{R}, Haste.
// CR 702.117: Surge — alternative cost if you or a teammate cast another spell this turn.
// ETB trigger ("if surge cost was paid, +1/+0 and haste to other creatures") stubbed:
// requires Condition::WasSurged (deferred — no card currently uses this conditional).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reckless-bushwhacker"),
        name: "Reckless Bushwhacker".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior", "Ally"]),
        oracle_text: "Surge {1}{R} (You may cast this spell for its surge cost rather than its mana cost if you or a teammate has cast another spell this turn.)\nHaste\nWhen Reckless Bushwhacker enters the battlefield, if its surge cost was paid, other creatures you control get +1/+0 and gain haste until end of turn.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Surge),
            AbilityDefinition::Surge {
                cost: ManaCost { generic: 1, red: 1, ..Default::default() },
            },
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: ETB trigger — "if surge cost was paid, other creatures get +1/+0 and haste
            // until end of turn" — requires Condition::WasSurged + ForEach(Creature, pump+haste).
            // Implement when authoring a card that needs it or when Condition::WasSurged is added.
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
    }
}
