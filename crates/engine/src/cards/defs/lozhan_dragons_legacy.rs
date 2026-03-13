// Lozhan, Dragons' Legacy — {3}{U}{R}, Legendary Creature — Dragon Shaman 4/2
// Flying
// Whenever you cast an Adventure or Dragon spell, Lozhan deals damage equal to that spell's mana value to any target that isn't a commander.
// TODO: DSL gap — triggered ability fires on casting Adventure or Dragon spells and deals
// damage equal to the spell's mana value to a target that isn't a commander; no
// TriggerCondition::WheneverYouCastSpellWithType(Adventure|Dragon) exists, and no
// EffectAmount::CastSpellManaValue or TargetFilter::NonCommander are available.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lozhan-dragons-legacy"),
        name: "Lozhan, Dragons' Legacy".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Shaman"],
        ),
        oracle_text: "Flying\nWhenever you cast an Adventure or Dragon spell, Lozhan deals damage equal to that spell's mana value to any target that isn't a commander.".to_string(),
        power: Some(4),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: triggered — whenever you cast an Adventure or Dragon spell, deal damage equal
            // to that spell's mana value to any target that isn't a commander.
            // DSL gap: no WheneverYouCastSpellWithSubtype(Dragon)/WheneverYouCastAdventure trigger;
            // no EffectAmount::CastSpellManaValue; no TargetFilter::NonCommander.
        ],
        ..Default::default()
    }
}
