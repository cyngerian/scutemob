// Ramos, Dragon Engine — {6}, Legendary Artifact Creature — Dragon 4/4
// Flying
// Whenever you cast a spell, put a +1/+1 counter on Ramos for each of that spell's colors.
// Remove five +1/+1 counters from Ramos: Add {W}{W}{U}{U}{B}{B}{R}{R}{G}{G}. Activate
// only once each turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ramos-dragon-engine"),
        name: "Ramos, Dragon Engine".to_string(),
        mana_cost: Some(ManaCost { generic: 6, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact, CardType::Creature],
            &["Dragon"],
        ),
        oracle_text: "Flying\nWhenever you cast a spell, put a +1/+1 counter on Ramos for each of that spell's colors.\nRemove five +1/+1 counters from Ramos: Add {W}{W}{U}{U}{B}{B}{R}{R}{G}{G}. Activate only once each turn.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — "Whenever you cast a spell" trigger with dynamic counter
            // count based on spell's colors. Needs WheneverYouCastASpell trigger +
            // EffectAmount::SpellColorCount.
            // TODO: DSL gap — "Remove five +1/+1 counters" cost + once-per-turn restriction.
            // Cost::RemoveCounter and TimingRestriction::OncePerTurn not in DSL.
        ],
        ..Default::default()
    }
}
