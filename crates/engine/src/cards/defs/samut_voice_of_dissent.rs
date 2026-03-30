// Samut, Voice of Dissent — {3}{R}{G}, Legendary Creature — Human Warrior 3/4
// Flash
// Double strike, vigilance, haste
// Other creatures you control have haste.
// {W}, {T}: Untap another target creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("samut-voice-of-dissent"),
        name: "Samut, Voice of Dissent".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Warrior"]),
        oracle_text: "Flash\nDouble strike, vigilance, haste\nOther creatures you control have haste.\n{W}, {T}: Untap another target creature.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Keyword(KeywordAbility::DoubleStrike),
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // "Other creatures you control have haste."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                    filter: EffectFilter::OtherCreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // {W}, {T}: Untap another target creature.
            // NOTE: "another" means any creature other than Samut herself; the DSL does not
            // enforce the self-exclusion on the target — treated as any target creature.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { white: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::UntapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreature],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
