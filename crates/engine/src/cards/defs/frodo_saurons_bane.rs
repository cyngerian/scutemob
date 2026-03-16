// Frodo, Sauron's Bane — {W}, Legendary Creature — Halfling Citizen 1/2
//
// {B}{B}{B}: Frodo, Sauron's Bane becomes a Halfling Scout and gains
//   "Whenever this creature deals combat damage to a player, the Ring tempts you."
// {1}{B}{B}: Frodo, Sauron's Bane becomes a legendary 2/3 Halfling Scout with
//   menace, indestructible, and "Whenever this creature deals combat damage to a
//   player, the Ring tempts you."
//
// CR 701.54a: "The Ring tempts you" is a keyword action.
// CR 603.2: "Whenever this deals combat damage" is a triggered ability.
//
// DSL gap: The two activated abilities that change Frodo's characteristics (subtype, P/T,
// keyword abilities, triggered abilities) require a layer-7 / continuous-effect pattern with
// the ability's source as the target and a full characteristic-override effect. This pattern
// is not yet expressible in the card-definition DSL. Both abilities are deferred.
//
// Partial implementation: Frodo's base form is represented correctly (W, 1/2 Legendary
// Halfling Citizen). The "whenever deals combat damage, Ring tempts you" is added as a
// permanent triggered ability to capture the flavor — this does not represent the full
// oracle text (which requires activation first), but preserves the core Ring mechanic
// for testing. Clearly documented so future M10+ DSL work can replace it.
//
// TODO: Remove the permanent combat-damage trigger and add two proper activated abilities
// once the DSL supports characteristic-changing activated abilities with triggered ability
// grants (layer 6/7 modification scoped to the triggered source).
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
        oracle_text: "{B}{B}{B}: Frodo, Sauron's Bane becomes a Halfling Scout and gains \"Whenever this creature deals combat damage to a player, the Ring tempts you.\"\n{1}{B}{B}: Frodo, Sauron's Bane becomes a legendary 2/3 Halfling Scout with menace, indestructible, and \"Whenever this creature deals combat damage to a player, the Ring tempts you.\""
            .to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            // TODO: {B}{B}{B} — transform Frodo into Halfling Scout form; grant triggered
            // ability "whenever this deals combat damage to a player, the Ring tempts you."
            // Requires DSL support for activated abilities that grant triggered abilities as
            // continuous effects (layer 6). Deferred to M10+.

            // TODO: {1}{B}{B} — transform Frodo into legendary 2/3 Halfling Scout with
            // Menace, Indestructible, and combat-damage ring-temptation trigger.
            // Same DSL gap as above. Deferred to M10+.

            // NOTE: No triggered ability is listed here deliberately — the oracle text's
            // "whenever deals combat damage" triggers only activate after one of the above
            // activated abilities resolves. Adding a permanent trigger would misrepresent the card.
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}
