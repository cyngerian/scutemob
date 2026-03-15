// Elder Deep-Fiend — {8}, Creature — Eldrazi Octopus 5/6; Emerge {5}{U}{U}, Flash,
// cast trigger: tap up to four target permanents.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elder-deep-fiend"),
        name: "Elder Deep-Fiend".to_string(),
        mana_cost: Some(ManaCost { generic: 8, ..Default::default() }),
        types: creature_types(&["Eldrazi", "Octopus"]),
        oracle_text: "Emerge {5}{U}{U} (You may cast this spell by sacrificing a creature and paying the emerge cost reduced by that creature's mana value.)\nFlash\nWhen you cast this spell, tap up to four target permanents.".to_string(),
        power: Some(5),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Emerge),
            AbilityDefinition::Emerge {
                cost: ManaCost { generic: 5, blue: 2, ..Default::default() },
            },
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            // TODO: "When you cast this spell, tap up to four target permanents."
            // Requires TriggerCondition::WhenCast (self-cast trigger) which does not
            // exist in the current DSL. Also requires TargetRequirement::UpToNTargetPermanents
            // or a multi-target tap effect. Add when TriggerCondition::WhenCast and
            // multi-target tap support are implemented.
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}
