// Innkeeper's Talent — {1}{G}, Enchantment — Class
// At the beginning of combat on your turn, put a +1/+1 counter on target creature you control.
// {G}: Level 2 — Permanents you control with counters on them have ward {1}.
// {3}{G}: Level 3 — If you would put one or more counters on a permanent or player, put
// twice that many of each of those kinds of counters on that permanent or player instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("innkeepers-talent"),
        name: "Innkeeper's Talent".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Class"]),
        oracle_text: "At the beginning of combat on your turn, put a +1/+1 counter on target creature you control.\n{G}: Level 2\nPermanents you control with counters on them have ward {1}.\n{3}{G}: Level 3\nIf you would put one or more counters on a permanent or player, put twice that many of each of those kinds of counters on that permanent or player instead.".to_string(),
        abilities: vec![
            // TODO: DSL gap — Class levels with begin-combat trigger (Level 1),
            // conditional ward grant (Level 2), counter doubling replacement (Level 3).
            // Class mechanics are partially supported (PB-15) but these specific level
            // abilities need more DSL primitives.
        ],
        ..Default::default()
    }
}
