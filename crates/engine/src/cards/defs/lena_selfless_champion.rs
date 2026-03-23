// Lena, Selfless Champion — {4}{W}{W}, Legendary Creature — Human Knight 3/3
// When Lena enters, create a 1/1 white Soldier creature token for each nontoken
// creature you control.
// Sacrifice Lena: Creatures you control with power less than Lena's power gain
// indestructible until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lena-selfless-champion"),
        name: "Lena, Selfless Champion".to_string(),
        mana_cost: Some(ManaCost { generic: 4, white: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Knight"],
        ),
        oracle_text: "When Lena enters, create a 1/1 white Soldier creature token for each nontoken creature you control.\nSacrifice Lena: Creatures you control with power less than Lena's power gain indestructible until end of turn.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // TODO: ETB creates variable count tokens (one per nontoken creature you control).
            //   EffectAmount::Fixed only; no CountNontokenCreaturesYouControl variant.
            //   W5 policy: omitted.
            // TODO: Sacrifice activated ability grants indestructible only to creatures with
            //   power less than Lena's power — requires dynamic power comparison in filter.
            //   DSL lacks this conditional filter. W5 policy: omitted.
        ],
        ..Default::default()
    }
}
