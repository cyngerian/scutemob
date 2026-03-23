// Goro-Goro, Disciple of Ryusei — {1}{R}, Legendary Creature — Goblin Samurai 2/2
// {R}: Creatures you control gain haste until end of turn.
// {3}{R}{R}: Create a 5/5 red Dragon Spirit creature token with flying. Activate only if
// you control an attacking modified creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goro-goro-disciple-of-ryusei"),
        name: "Goro-Goro, Disciple of Ryusei".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Goblin", "Samurai"]),
        oracle_text: "{R}: Creatures you control gain haste until end of turn.\n{3}{R}{R}: Create a 5/5 red Dragon Spirit creature token with flying. Activate only if you control an attacking modified creature.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "{R}: Creatures you control gain haste until EOT" — mass keyword grant not in DSL
            // TODO: "{3}{R}{R}: Create Dragon Spirit" — activation condition "control attacking
            // modified creature" not in DSL (modified = equipment/aura/counters)
        ],
        ..Default::default()
    }
}
