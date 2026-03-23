// Serpent's Soul-Jar — {2}{B}, Artifact
// Whenever an Elf you control dies, exile it.
// {T}, Pay 2 life: Until end of turn, you may cast a creature spell from among cards
// exiled with this artifact.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("serpents-soul-jar"),
        name: "Serpent's Soul-Jar".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Whenever an Elf you control dies, exile it.\nWhenever you cast a spell from exile this way, you may pay {B}. When you do, each opponent loses 2 life and you gain 2 life.".to_string(),
        abilities: vec![
            // TODO: DSL gap — death trigger with controller + subtype filter + exile
            // replacement + cast-from-exile permission. Multiple DSL gaps.
        ],
        ..Default::default()
    }
}
