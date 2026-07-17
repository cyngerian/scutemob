// Food Chain — {2}{G}, Enchantment
// Exile a creature you control: Add X mana of any one color, where X is 1 plus
// the exiled creature's mana value. Spend this mana only to cast creature spells.
//
// TODO: Multiple DSL gaps:
// 1. Cost::ExileCreatureYouControl does not exist (only SacrificeSelf, SacrificeCreature).
// 2. "X = 1 + exiled creature's MV" — dynamic mana amount from exiled card's MV.
// 3. ManaRestriction::CreatureSpellsOnly exists but AddManaAnyColorRestricted
//    needs a dynamic amount, not fixed.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("food-chain"),
        name: "Food Chain".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Exile a creature you control: Add X mana of any one color, where X is 1 \
                      plus the exiled creature's mana value. Spend this mana only to cast \
                      creature spells."
            .to_string(),
        abilities: vec![
            // TODO: Exile-creature cost + dynamic mana = 1 + MV + creature-only restriction.
            // All three sub-components are DSL gaps. Leaving as placeholder.
            AbilityDefinition::Activated {
                // TODO: Cost should be "exile a creature you control" not sacrifice.
                cost: Cost::SacrificeSelf,
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        completeness: Completeness::known_wrong(
            "the activated ability is a wrong placeholder — Cost::SacrificeSelf sacrifices Food \
             Chain itself instead of exiling a creature you control, and it adds ONE unrestricted \
             mana of any color instead of (1 + exiled creature's MV) spendable only on creature \
             spells. Blockers: no exile-another-creature Cost variant (Cost has ExileSelf only, \
             card_definition.rs:1224); AddManaAnyColor has no amount (:1472) and \
             AddManaAnyColorRestricted takes no EffectAmount. Prefer abilities: vec![] until both \
             land.",
        ),
        ..Default::default()
    }
}
