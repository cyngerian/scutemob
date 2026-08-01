// Dwynen's Elite — {1}{G}, Creature — Elf Warrior 2/2
// When this creature enters, if you control another Elf, create a 1/1 green Elf Warrior
// creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dwynen-s-elite"),
        name: "Dwynen's Elite".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Elf", "Warrior"]),
        oracle_text: "When this creature enters, if you control another Elf, create a 1/1 green \
                      Elf Warrior creature token."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 603.4 (ruling 2024-11-08): "if you control another Elf" checked BOTH at
            // queue time (rules/replacement.rs's self-ETB CardDef sweep, PB-DP6) and
            // re-checked at resolution (InterveningIf::CardDef, PB-DX1).
            //
            // PB-DX3b: this ability did not exist in the def at all — `abilities` was
            // empty — so it is authored here, not merely gated (same shape as
            // `inventors_fair` in PB-DX3). The blocked note claimed
            // YouControlNOrMoreWithFilter's evaluator silently ignores
            // TargetFilter.exclude_self; that was stale — PB-EF1 wired it
            // (effects/mod.rs's `!filter.exclude_self || obj.id != source` check, marker
            // EF-5). `exclude_self: true` on the filter below is the whole point of the
            // ability (CR 109.1 "another") and is pinned by T8: Dwynen's Elite alone
            // creates NO token.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Elf Warrior".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Elf".to_string()), SubType("Warrior".to_string())]
                            .into_iter()
                            .collect(),
                        colors: [Color::Green].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: EffectAmount::Fixed(1),
                        supertypes: imbl::OrdSet::new(),
                        keywords: imbl::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
                    count: 1,
                    filter: TargetFilter {
                        has_subtype: Some(SubType("Elf".to_string())),
                        exclude_self: true,
                        ..Default::default()
                    },
                }),
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
