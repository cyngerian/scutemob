// Oketra's Monument — {3}, Legendary Artifact
// White creature spells you cast cost {1} less to cast.
// Whenever you cast a creature spell, create a 1/1 white Warrior creature token with vigilance.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("oketras-monument"),
        name: "Oketra's Monument".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact],
            &[],
        ),
        oracle_text: "White creature spells you cast cost {1} less to cast.\nWhenever you cast a creature spell, create a 1/1 white Warrior creature token with vigilance.".to_string(),
        abilities: vec![
            // TODO: Cost reduction only applies to white creature spells. DSL CostReduction
            //   uses SpellsYouCast (all spells) with no color+type filter. Omitting to avoid
            //   wrong game state (reducing non-white creature spells incorrectly).
            // Warrior token trigger: WheneverYouCastSpell fires on any spell, not just creatures.
            // TODO: WheneverYouCastSpell has no creature-only filter. Omitting per W5 policy.
        ],
        ..Default::default()
    }
}
