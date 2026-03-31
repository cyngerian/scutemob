// Emergency Eject — {2}{W}, Instant
// Destroy target nonland permanent. Its controller creates a Lander token.
// (It's an artifact with "{2}, {T}, Sacrifice this token: Search your library for a basic
// land card, put it onto the battlefield tapped, then shuffle.")
//
// TODO: "Lander token" is a new named token type not present in the DSL.
// The CreateToken effect requires a TokenSpec, and no lander_token_spec() helper
// exists. The destroy effect is expressible but the Lander creation cannot be
// faithfully implemented without a Lander token spec. Per W5 policy, only the
// destroy effect is implemented; the Lander creation is left as TODO.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("emergency-eject"),
        name: "Emergency Eject".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target nonland permanent. Its controller creates a Lander token. (It's an artifact with \"{2}, {T}, Sacrifice this token: Search your library for a basic land card, put it onto the battlefield tapped, then shuffle.\")".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                // TODO: Create a Lander token for the target's controller.
                // Lander is an artifact token with an activated ability.
                // TokenSpec does not have a Lander variant; no lander_token_spec() exists.
            ]),
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                non_land: true,
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
