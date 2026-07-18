// Strix Serenade — {U}, Instant
// Counter target artifact, creature, or planeswalker spell. Its controller
// creates a 2/2 blue Bird creature token with flying.
//
// TODO: DSL gap — CounterSpell targets any spell; TargetRequirement lacks a
// multi-type filter (artifact OR creature OR planeswalker). The "its
// controller creates a token" also targets the controller of the countered
// spell (not the caster), which is not expressible in the DSL.
// Abilities are omitted to avoid incorrect behavior.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("strix-serenade"),
        name: "Strix Serenade".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target artifact, creature, or planeswalker spell. Its controller \
                      creates a 2/2 blue Bird creature token with flying."
            .to_string(),
        abilities: vec![
            // TODO: DSL gap — multi-type target filter (artifact/creature/planeswalker)
            // and "its controller" token creation not expressible.
        ],
        completeness: Completeness::inert(
            "Sole blocker: token recipient. 'Its controller creates a 2/2 blue Bird token' — \
             Effect::CreateToken/TokenSpec has no player field, so tokens always go to \
             ctx.controller; in multiplayer the token would go to the caster instead of the \
             countered spell's controller. The multi-type target filter EXISTS \
             (TargetSpellWithFilter + TargetFilter::has_card_types). Same blocker as \
             stroke_of_midnight, pongify, rapid_hybridization, beast_within.",
        ),
        ..Default::default()
    }
}
