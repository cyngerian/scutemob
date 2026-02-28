// 109. Inexorable Tide — {3}{U}{U}, Enchantment.
// CR 701.34a: "Whenever you cast a spell, proliferate."
// 111. Rift Bolt — {2}{R}, Sorcery; Suspend 1—{R}.
// CR 702.62: Suspend 1 means exile from hand with 1 time counter by paying {R};
// at upkeep, remove the counter and cast for free. Deals 3 damage to any target.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rift-bolt"),
        name: "Rift Bolt".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Rift Bolt deals 3 damage to any target.\nSuspend 1—{R} (Rather than cast this card from your hand, pay {R} and exile it with a time counter on it. At the beginning of your upkeep, remove a time counter. When the last is removed, cast it without paying its mana cost.)".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Suspend),
            AbilityDefinition::Suspend {
                cost: ManaCost { red: 1, ..Default::default() },
                time_counters: 1,
            },
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(3),
                },
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
