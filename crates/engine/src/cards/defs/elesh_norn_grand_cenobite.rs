// Elesh Norn, Grand Cenobite — {5}{W}{W}, Legendary Creature — Phyrexian Praetor 4/7
// Vigilance
// Other creatures you control get +2/+2.
// Creatures your opponents control get -2/-2.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elesh-norn-grand-cenobite"),
        name: "Elesh Norn, Grand Cenobite".to_string(),
        mana_cost: Some(ManaCost { generic: 5, white: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Phyrexian", "Praetor"],
        ),
        oracle_text: "Vigilance\nOther creatures you control get +2/+2.\nCreatures your opponents control get -2/-2.".to_string(),
        power: Some(4),
        toughness: Some(7),
        abilities: vec![
            // TODO: Three abilities need EffectFilter::CreaturesOpponentsControl (DSL gap):
            // 1. Vigilance — fine standalone, but paired with only the buff (no debuff)
            //    would produce wrong game state (W5 policy). Stripped to empty.
            // 2. "Other creatures you control get +2/+2." — correct DSL exists.
            // 3. "Creatures your opponents control get -2/-2." — EffectFilter gap.
            // All stripped per W5: buff without debuff = wrong game state.
        ],
        ..Default::default()
    }
}
