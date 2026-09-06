// Pongify — {U} Instant; destroy target creature. It can't be regenerated.
// Its controller creates a 3/3 green Ape creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("pongify"),
        name: "Pongify".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target creature. It can't be regenerated. Its controller creates a \
                      3/3 green Ape creature token."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: true,
                },
                Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Ape".to_string(),
                        power: 3,
                        toughness: 3,
                        colors: [Color::Green].into_iter().collect(),
                        supertypes: OrdSet::new(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Ape".to_string())].into_iter().collect(),
                        keywords: OrdSet::new(),
                        count: EffectAmount::Fixed(1),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        // CR 608.2h: "its controller" is the destroyed permanent's
                        // controller, not the caster (resolves via LKI after CR 400.7).
                        recipient: PlayerTarget::ControllerOf(Box::new(
                            EffectTarget::DeclaredTarget { index: 0 },
                        )),
                        ..Default::default()
                    },
                },
            ]),
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
