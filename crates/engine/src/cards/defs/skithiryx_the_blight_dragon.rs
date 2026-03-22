// Skithiryx, the Blight Dragon — {3}{B}{B}, Legendary Creature — Phyrexian Dragon Skeleton 4/4
// Flying, Infect, {B}: haste until EOT, {B}{B}: Regenerate
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skithiryx-the-blight-dragon"),
        name: "Skithiryx, the Blight Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Phyrexian", "Dragon", "Skeleton"],
        ),
        oracle_text: "Flying\nInfect (This creature deals damage to creatures in the form of -1/-1 counters and to players in the form of poison counters.)\n{B}: Skithiryx gains haste until end of turn.\n{B}{B}: Regenerate Skithiryx.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Infect),
            // {B}: Skithiryx gains haste until end of turn.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { black: 1, ..Default::default() }),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                        filter: EffectFilter::Source,
                        duration: EffectDuration::UntilEndOfTurn,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // {B}{B}: Regenerate Skithiryx (CR 701.19a).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { black: 2, ..Default::default() }),
                effect: Effect::Regenerate {
                    target: EffectTarget::Source,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
