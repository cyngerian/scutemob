// Florian, Voldaren Scion — {1}{B}{R}, Legendary Creature — Vampire Noble 3/3
// First strike; at beginning of each postcombat main phase, look at top X cards where
// X = total life opponents lost this turn; exile one, play it this turn.
// TODO: Postcombat main phase trigger with variable X (life lost by opponents this turn)
// and "exile one, play it this turn" effect. DSL gaps: no TriggerCondition for
// AtBeginningOfPostcombatMainPhase; no EffectAmount tracking life-lost-this-turn across
// all opponents; no look-at-top-X-and-exile-one pattern. Deferred.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("florian-voldaren-scion"),
        name: "Florian, Voldaren Scion".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Vampire", "Noble"],
        ),
        oracle_text: "First strike\nAt the beginning of each of your postcombat main phases, look at the top X cards of your library, where X is the total amount of life your opponents lost this turn. Exile one of those cards and put the rest on the bottom of your library in a random order. You may play the exiled card this turn.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            // TODO: Postcombat main phase trigger with X = opponents' life lost this turn.
            // DSL gaps: no AtBeginningOfPostcombatMainPhase trigger condition;
            // no EffectAmount variant for opponents' life lost; no look-top-X-exile-one effect.
        ],
        ..Default::default()
    }
}
