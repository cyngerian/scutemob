// Ghave, Guru of Spores — {2}{W}{B}{G}, Legendary Creature — Fungus Shaman 0/0
// Ghave enters with five +1/+1 counters on it.
// {1}, Remove a +1/+1 counter from a creature you control: Create a 1/1 green Saproling
//      creature token.
// {1}, Sacrifice a creature: Put a +1/+1 counter on target creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ghave-guru-of-spores"),
        name: "Ghave, Guru of Spores".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, black: 1, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Fungus", "Shaman"]),
        oracle_text: "Ghave enters with five +1/+1 counters on it.\n{1}, Remove a +1/+1 counter from a creature you control: Create a 1/1 green Saproling creature token.\n{1}, Sacrifice a creature: Put a +1/+1 counter on target creature.".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            // TODO: ETB with five +1/+1 counters — WhenEntersBattlefield + AddCounters on self
            // with count 5. EffectTarget::Source in ETB context not wired.
            // TODO: "{1}, Remove a +1/+1 counter from a creature you control" — Cost::RemoveCounter
            // not in DSL.
            // TODO: "{1}, Sacrifice a creature: Put a +1/+1 counter on target creature" —
            // Cost::Sequence([Cost::Mana, Cost::Sacrifice]) with AddCounters not tested
            // with DeclaredTarget.
        ],
        ..Default::default()
    }
}
