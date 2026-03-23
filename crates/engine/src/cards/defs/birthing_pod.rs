// Birthing Pod — {3}{G/P} Artifact
// {1}{G/P}, {T}, Sacrifice a creature: Search your library for a creature card with mana value equal to
//   1 plus the sacrificed creature's mana value, put that card onto the battlefield, then shuffle.
//   Activate only as a sorcery.
//
// DSL gap: the search filter "mana value equal to 1 plus sacrificed creature's mana value" is dynamic
//   and requires runtime context about the sacrificed creature (no TargetFilter for dynamic MV).
//   Phyrexian mana cost also not representable in ManaCost (no {G/P} field).
// W5 policy: complex sacrifice-conditional search cannot be expressed faithfully — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("birthing-pod"),
        name: "Birthing Pod".to_string(),
        mana_cost: Some(ManaCost { generic: 3, phyrexian: vec![PhyrexianMana::Single(ManaColor::Green)], ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "({G/P} can be paid with either {G} or 2 life.)\n{1}{G/P}, {T}, Sacrifice a creature: Search your library for a creature card with mana value equal to 1 plus the sacrificed creature's mana value, put that card onto the battlefield, then shuffle. Activate only as a sorcery.".to_string(),
        abilities: vec![
            // TODO: {1}{G/P}, {T}, Sacrifice a creature: search for creature with MV = sacrificed MV + 1
            //   (needs dynamic MV filter on SearchLibrary; Phyrexian mana cost gap; sacrifice-as-cost with reference)
        ],
        ..Default::default()
    }
}
