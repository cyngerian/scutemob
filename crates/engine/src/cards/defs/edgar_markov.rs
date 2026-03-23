// Edgar Markov — {3}{R}{W}{B}, Legendary Creature — Vampire Knight 4/4
// Eminence — Whenever you cast another Vampire spell, if Edgar is in the command zone or on
//            the battlefield, create a 1/1 black Vampire creature token.
// First strike, haste
// Whenever Edgar attacks, put a +1/+1 counter on each Vampire you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("edgar-markov"),
        name: "Edgar Markov".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, white: 1, black: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Vampire", "Knight"]),
        oracle_text: "Eminence — Whenever you cast another Vampire spell, if Edgar is in the command zone or on the battlefield, create a 1/1 black Vampire creature token.\nFirst strike, haste\nWhenever Edgar attacks, put a +1/+1 counter on each Vampire you control.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: Eminence — triggers from command zone. WheneverYouCastSpell with subtype
            // filter (Vampire only) + command-zone condition not expressible in DSL.
            // TODO: "Whenever Edgar attacks" — AddCounters to each Vampire you control.
            // EffectTarget::AllCreaturesYouControlWithSubtype not in DSL.
        ],
        ..Default::default()
    }
}
