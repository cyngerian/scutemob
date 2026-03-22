// Animate Dead — {1}{B}, Enchantment — Aura
// Enchant creature card in a graveyard
// When this enters, if it's on the battlefield, it loses "enchant creature card in a
// graveyard" and gains "enchant creature put onto the battlefield with this Aura."
// Return enchanted creature card to the battlefield under your control and attach this to it.
// When this leaves the battlefield, that creature's controller sacrifices it.
// Enchanted creature gets -1/-0.
//
// TODO: Complex reanimation Aura — enchants graveyard card (no EnchantTarget variant for
//   graveyard), changes enchant target on ETB, returns creature, self-attaches, LTB sacrifice
//   trigger, and -1/-0 static. Multiple DSL gaps make this inexpressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("animate-dead"),
        name: "Animate Dead".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature card in a graveyard\nWhen Animate Dead enters, if it's on the battlefield, it loses \"enchant creature card in a graveyard\" and gains \"enchant creature put onto the battlefield with Animate Dead.\" Return enchanted creature card to the battlefield under your control and attach Animate Dead to it. When Animate Dead leaves the battlefield, that creature's controller sacrifices it.\nEnchanted creature gets -1/-0.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
