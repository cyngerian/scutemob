// Stroke of Midnight — {2}{W} Instant; destroy target nonland permanent.
// Its controller creates a 1/1 white Human creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("stroke-of-midnight"),
        name: "Stroke of Midnight".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target nonland permanent. Its controller creates a 1/1 white Human creature token.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // "Its controller creates a 1/1 white Human creature token."
            // TODO: CreateToken does not have a player/controller field — tokens always go to
            // ctx.controller (the spell's caster). In multiplayer, the token should go to the
            // destroyed permanent's controller (e.g., PlayerTarget::ControllerOf), but
            // CreateToken lacks this parameter. This is a known systemic DSL gap (see also
            // Pongify, Rapid Hybridization, Beast Within). Fix when CreateToken gains a player field.
            effect: Effect::Sequence(vec![
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Human".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: [Color::White].into_iter().collect(),
                        supertypes: OrdSet::new(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Human".to_string())].into_iter().collect(),
                        keywords: OrdSet::new(),
                        count: 1,
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
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
