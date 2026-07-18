// Goldhound — {R}, Artifact Creature — Treasure Dog 1/1
// First strike
// Menace (This creature can't be blocked except by two or more creatures.)
// {T}, Sacrifice this creature: Add one mana of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goldhound"),
        name: "Goldhound".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: types_sub(
            &[CardType::Artifact, CardType::Creature],
            &["Treasure", "Dog"],
        ),
        oracle_text: "First strike\nMenace (This creature can't be blocked except by two or more \
                      creatures.)\n{T}, Sacrifice this creature: Add one mana of any color."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            // {T}, Sacrifice this creature: Add one mana of any color.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![Cost::Tap, Cost::SacrificeSelf]),
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::known_wrong(
            "CR 106.1b: '{T}, Sacrifice this creature: Add one mana of any color' adds one \
             COLORLESS mana (probed: +1 colorless). SR-34 DID lower this to a real ManaAbility — \
             sacrifice_self already existed on ManaAbility and only the matches!(cost, Cost::Tap) \
             gate was blocking it, so the mechanism is now correct (stack empty, self sacrificed) \
             — but the mana it produces is not one of the legal options 'any color' offers. \
             Blocked on interactive/deterministic color choice for any_color mana.",
        ),
        ..Default::default()
    }
}
