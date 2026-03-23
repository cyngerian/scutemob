// Incendiary Command — {3}{R}{R}, Sorcery
// Choose two —
// • Incendiary Command deals 4 damage to target player or planeswalker.
// • Incendiary Command deals 2 damage to each creature.
// • Destroy target nonbasic land.
// • Each player discards all the cards in their hand, then draws that many cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("incendiary-command"),
        name: "Incendiary Command".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose two —\n• Incendiary Command deals 4 damage to target player or planeswalker.\n• Incendiary Command deals 2 damage to each creature.\n• Destroy target nonbasic land.\n• Each player discards all the cards in their hand, then draws that many cards.".to_string(),
        abilities: vec![
            // TODO: Modal spell (choose two of four) — requires modal spell support with
            // per-mode targets. Mode 1 needs TargetPlayerOrPlaneswalker, mode 2 needs
            // DealDamage to each creature, mode 3 needs DestroyTarget(nonbasic land),
            // mode 4 needs wheel effect (discard hand + draw that many).
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
