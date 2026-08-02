// Galadhrim Brigade — {2}{G}, Creature — Elf Soldier 2/2
// Squad {1}{G}
// Other Elves you control get +1/+1.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("galadhrim-brigade"),
        name: "Galadhrim Brigade".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Elf", "Soldier"]),
        oracle_text: "Squad {1}{G} (As an additional cost to cast this spell, you may pay {1}{G} \
                      any number of times. When this creature enters, create that many tokens \
                      that are copies of it.)\nOther Elves you control get +1/+1."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 702.157a: Squad — presence marker; the actual cost is carried by
            // AbilityDefinition::Squad below.
            AbilityDefinition::Keyword(KeywordAbility::Squad),
            // CR 702.157a: Squad {1}{G} — additional cost paid N times; the ETB trigger
            // creates N token copies.
            //
            // UI-2 (`scutemob-178`): this def shipped `Complete` and deck-legal with the
            // KEYWORD marker alone and no cost, so `casting.rs::get_squad_cost` returned
            // `None` and EVERY `squad_count > 0` cast of it was refused with "spell has
            // squad keyword but no squad cost defined". The keyword was authored and the
            // cost was not — the same shape CARDS-2 found repeatedly, and this is the very
            // card the first human playtest hit (`memory/playtest-triage-2026-08-02.md`
            // F9). `ultramarines_honour_guard.rs` is the reference: both variants, always.
            AbilityDefinition::Squad {
                cost: ManaCost {
                    generic: 1,
                    green: 1,
                    ..Default::default()
                },
            },
            // "Other Elves you control get +1/+1" — lord static ability
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: crate::state::EffectLayer::PtModify,
                    modification: crate::state::LayerModification::ModifyBoth(1),
                    filter: crate::state::EffectFilter::OtherCreaturesYouControlWithSubtype(
                        SubType("Elf".to_string()),
                    ),
                    duration: crate::state::EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
