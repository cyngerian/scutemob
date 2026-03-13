// Boromir, Warden of the Tower — {2}{W}, Legendary Creature — Human Soldier 3/3
// "Vigilance
// Whenever an opponent casts a spell, if no mana was spent to cast it, counter that spell.
// Sacrifice Boromir: Creatures you control gain indestructible until end of turn.
// The Ring tempts you."
//
// Vigilance is implemented.
//
// TODO: DSL gap — "Whenever an opponent casts a spell, if no mana was spent to cast it,
// counter that spell" requires WheneverOpponentCastsSpell trigger + an intervening-if
// condition checking that zero mana was spent. No such trigger condition or mana-spent
// check exists in the DSL.
//
// TODO: DSL gap — "Sacrifice Boromir: ... The Ring tempts you" is an activated sacrifice
// ability with a two-part effect (grant indestructible to all creatures you control +
// ring tempt). No activated ability with SacrificeSelf cost is currently expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("boromir-warden-of-the-tower"),
        name: "Boromir, Warden of the Tower".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "Vigilance\nWhenever an opponent casts a spell, if no mana was spent to cast it, counter that spell.\nSacrifice Boromir: Creatures you control gain indestructible until end of turn. The Ring tempts you.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
        ],
        ..Default::default()
    }
}
