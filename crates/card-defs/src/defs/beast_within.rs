// 22. Beast Within — {2G}, Instant; destroy target permanent, its controller
// creates a 3/3 green Beast creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("beast-within"),
        name: "Beast Within".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            generic: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target permanent. Its controller creates a 3/3 green Beast creature \
                      token."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Beast".to_string(),
                        power: 3,
                        toughness: 3,
                        colors: [Color::Green].into_iter().collect(),
                        supertypes: OrdSet::new(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Beast".to_string())].into_iter().collect(),
                        keywords: OrdSet::new(),
                        count: EffectAmount::Fixed(1),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        // CR 701.7a / CR 608.2h: "ITS controller" is the destroyed
                        // permanent's controller, not the caster. The destroy has already
                        // retired the target's ObjectId (CR 400.7), so this resolves
                        // through the LKI snapshot -- see `resolve_player_target_list`'s
                        // `ControllerOf` arm.
                        recipient: PlayerTarget::ControllerOf(Box::new(
                            EffectTarget::DeclaredTarget { index: 0 },
                        )),
                        ..Default::default()
                    },
                },
            ]),
            targets: vec![TargetRequirement::TargetPermanent],
            modes: None,
            cant_be_countered: false,
        }],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
