// Mental Misstep — {U/P}, Instant
// ({U/P} can be paid with either {U} or 2 life.)
// Counter target spell with mana value 1.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mental-misstep"),
        name: "Mental Misstep".to_string(),
        mana_cost: Some(ManaCost {
            phyrexian: vec![PhyrexianMana::Single(ManaColor::Blue)],
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "({U/P} can be paid with either {U} or 2 life.)\nCounter target spell with mana value 1.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::CounterSpell {
                target: EffectTarget::DeclaredTarget { index: 0 },
            },
            targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter {
                max_cmc: Some(1),
                min_cmc: Some(1),
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
