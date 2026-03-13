// Skithiryx, the Blight Dragon — {3}{B}{B}, Legendary Creature — Phyrexian Dragon Skeleton 4/4
// "Flying
// Infect (This creature deals damage to creatures in the form of -1/-1 counters and to players
// in the form of poison counters.)
// {B}: Skithiryx gains haste until end of turn.
// {B}{B}: Regenerate Skithiryx."
//
// Flying and Infect are implemented.
//
// TODO: DSL gap — "{B}: Skithiryx gains haste until end of turn" is an activated ability that
// applies ApplyContinuousEffect targeting self — no confirmed "grant keyword to self" activated
// ability pattern in current DSL.
//
// TODO: DSL gap — "{B}{B}: Regenerate Skithiryx" is a Regenerate activated ability.
// KeywordAbility::Regenerate is a static keyword; an activated self-regeneration ability
// is not directly expressible in the current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skithiryx-the-blight-dragon"),
        name: "Skithiryx, the Blight Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Phyrexian", "Dragon", "Skeleton"],
        ),
        oracle_text: "Flying\nInfect (This creature deals damage to creatures in the form of -1/-1 counters and to players in the form of poison counters.)\n{B}: Skithiryx gains haste until end of turn.\n{B}{B}: Regenerate Skithiryx.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Infect),
        ],
        ..Default::default()
    }
}
