// Bloodline Necromancer — {4}{B}, Creature — Vampire Wizard 3/2
// Lifelink
// When this creature enters, you may return target Vampire or Wizard creature card from your graveyard to the battlefield.
// TODO: DSL gap — ETB trigger returning a target creature card with subtype filter (Vampire or Wizard)
// from graveyard to battlefield; no return_from_graveyard effect with subtype OR filter exists.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloodline-necromancer"),
        name: "Bloodline Necromancer".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Wizard"]),
        oracle_text: "Lifelink\nWhen this creature enters, you may return target Vampire or Wizard creature card from your graveyard to the battlefield.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            // TODO: ETB triggered — return target Vampire or Wizard creature card from your GY to BF.
            // DSL gap: no return_from_graveyard effect; no TargetFilter with multi-subtype OR condition.
        ],
        ..Default::default()
    }
}
