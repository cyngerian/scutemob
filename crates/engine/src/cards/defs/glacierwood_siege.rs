// Glacierwood Siege — {3}{G}, Enchantment
// As this enters, choose Temur or Sultai.
// • Temur — Whenever you cast an instant or sorcery spell, mill four cards.
// • Sultai — You may play lands from your graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("glacierwood-siege"),
        name: "Glacierwood Siege".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "As Glacierwood Siege enters, choose Temur or Sultai.\n• Temur — Whenever you cast an instant or sorcery spell, mill four cards.\n• Sultai — You may play lands from your graveyard.".to_string(),
        abilities: vec![
            // TODO: "Choose Temur or Sultai" on ETB — siege mode choice.
            // Temur: cast-trigger mill 4 (approximable with WheneverYouCastSpell trigger).
            // Sultai: play lands from graveyard (BLOCKED — no PlayerPermission for this).
        ],
        ..Default::default()
    }
}
