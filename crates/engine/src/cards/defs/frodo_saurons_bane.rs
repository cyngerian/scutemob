// Frodo, Sauron's Bane — {W}, Legendary Creature — Halfling Citizen 1/2
//
// Actual oracle text (Scryfall 2023):
//   {W/B}{W/B}: If Frodo is a Citizen, it becomes a Halfling Scout with base
//     power and toughness 2/3 and lifelink.
//   {B}{B}{B}: If Frodo is a Scout, it becomes a Halfling Rogue with
//     "Whenever this creature deals combat damage to a player, that player
//     loses the game if the Ring has tempted you four or more times this game.
//     Otherwise, the Ring tempts you."
//
// CR 701.54a: "The Ring tempts you" is a keyword action.
// CR 603.2: "Whenever this deals combat damage" is a triggered ability.
//
// DSL gap: Both activated abilities change Frodo's subtype, base P/T, and grant keyword
// abilities and triggered abilities. This requires characteristic-override continuous
// effects scoped to the activated ability's source — not yet expressible in the card DSL.
// Both abilities are deferred to M10+ DSL work.
//
// TODO: {W/B}{W/B} ability — if Frodo is a Citizen, change subtype to Halfling Scout,
// set base P/T to 2/3, and grant lifelink as a continuous effect while this ability
// is in effect. Requires DSL support for conditional subtype-check + characteristic
// override activated abilities.
//
// TODO: {B}{B}{B} ability — if Frodo is a Scout, change subtype to Halfling Rogue,
// and grant "whenever this deals combat damage, that player loses the game if the
// Ring has tempted you 4+ times this game; otherwise the Ring tempts you."
// Requires conditional subtype check, triggered ability grant, and Ring-count condition.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("frodo-saurons-bane"),
        name: "Frodo, Sauron's Bane".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Halfling", "Citizen"],
        ),
        oracle_text: "{W/B}{W/B}: If Frodo, Sauron's Bane is a Citizen, it becomes a Halfling Scout with base power and toughness 2/3 and lifelink.\n{B}{B}{B}: If Frodo, Sauron's Bane is a Scout, it becomes a Halfling Rogue with \"Whenever this creature deals combat damage to a player, that player loses the game if the Ring has tempted you four or more times this game. Otherwise, the Ring tempts you.\""
            .to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            // TODO: {W/B}{W/B} — if Frodo is a Citizen, change subtype to Halfling Scout,
            // set base P/T to 2/3, grant lifelink. Requires conditional subtype-check +
            // characteristic-override activated ability DSL (M10+).

            // TODO: {B}{B}{B} — if Frodo is a Scout, change subtype to Halfling Rogue,
            // grant "whenever this deals combat damage to a player, that player loses the
            // game if Ring has tempted you 4+ times; otherwise the Ring tempts you."
            // Requires conditional subtype-check, triggered ability grant, and Ring-count
            // condition (M10+).
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
    }
}
