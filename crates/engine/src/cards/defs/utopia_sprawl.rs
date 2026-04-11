// Utopia Sprawl — {G}, Enchantment — Aura
// Enchant Forest
// As Utopia Sprawl enters, choose a color.
// Whenever enchanted Forest is tapped for mana, its controller adds one mana
// of the chosen color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("utopia-sprawl"),
        name: "Utopia Sprawl".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant Forest\nAs Utopia Sprawl enters, choose a color.\nWhenever enchanted Forest is tapped for mana, its controller adds one mana of the chosen color.".to_string(),
        abilities: vec![
            // CR 303.4: Enchant land restriction (Forest specifically, but engine uses Land
            // for simplicity — Forest subtype filtering is not yet in the target filter DSL).
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Land)),
            // CR 614.12 / CR 614.12a: "As this enters, choose a color."
            // Replacement effect — NOT a triggered ability (PB-X C1 lesson).
            // The ETB replacement fires AFTER the Aura is attached to its target (per
            // resolution.rs ordering: Aura attachment at line ~1547, then self-ETB
            // replacements at line ~1571). This means chosen_color is set atomically with
            // ETB, and the triggered mana ability sees it immediately.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::ChooseColor(Color::Green),
                is_self: true,
                unless_condition: None,
            },
            // CR 605.1b / CR 106.12a: "Whenever enchanted Forest is tapped for mana,
            // its controller adds one mana of the chosen color."
            // Triggered mana ability using the WhenTappedForMana condition.
            // The trigger source filter (EnchantedLand) restricts this to the specific
            // Forest this Aura is attached to. At execution time, ctx.source is this
            // Utopia Sprawl object, so Effect::AddManaOfChosenColor reads this Aura's
            // chosen_color field.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenTappedForMana {
                    source_filter: ManaSourceFilter::EnchantedLand,
                },
                effect: Effect::AddManaOfChosenColor {
                    player: PlayerTarget::Controller,
                    amount: 1,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
