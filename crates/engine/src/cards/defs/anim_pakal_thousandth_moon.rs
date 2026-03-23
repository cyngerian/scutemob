// Anim Pakal, Thousandth Moon — {1}{R}{W}, Legendary Creature — Human Soldier 1/2
// Whenever you attack with one or more non-Gnome creatures, put a +1/+1 counter on Anim
// Pakal, then create X 1/1 colorless Gnome artifact creature tokens that are tapped and
// attacking, where X is the number of +1/+1 counters on Anim Pakal.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("anim-pakal-thousandth-moon"),
        name: "Anim Pakal, Thousandth Moon".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "Whenever you attack with one or more non-Gnome creatures, put a +1/+1 counter on Anim Pakal, Thousandth Moon, then create X 1/1 colorless Gnome artifact creature tokens that are tapped and attacking, where X is the number of +1/+1 counters on Anim Pakal.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            // TODO: "Whenever you attack with non-Gnomes" trigger not in DSL.
            // TODO: Counter-based token count not in DSL.
        ],
        ..Default::default()
    }
}
