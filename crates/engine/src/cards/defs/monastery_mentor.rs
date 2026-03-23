// Monastery Mentor — {2}{W}, Creature — Human Monk 2/2
// Prowess (Whenever you cast a noncreature spell, this creature gets +1/+1 until end
// of turn.)
// Whenever you cast a noncreature spell, create a 1/1 white Monk creature token with
// prowess.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("monastery-mentor"),
        name: "Monastery Mentor".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Monk"]),
        oracle_text: "Prowess (Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn.)\nWhenever you cast a noncreature spell, create a 1/1 white Monk creature token with prowess.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Prowess),
            // TODO: "Whenever you cast a noncreature spell" — WheneverYouCastSpell lacks a
            //   noncreature filter. Using the unfiltered trigger is wrong (fires on creature
            //   spells too). W5 policy: no approximation.
        ],
        ..Default::default()
    }
}
