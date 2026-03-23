// Umezawa's Jitte — {2}, Legendary Artifact — Equipment
// Whenever equipped creature deals combat damage, put two charge counters on Umezawa's Jitte.
// Remove a charge counter from Umezawa's Jitte: Choose one —
// • Equipped creature gets +2/+2 until end of turn.
// • Target creature gets -1/-1 until end of turn.
// • You gain 2 life.
// Equip {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("umezawas-jitte"),
        name: "Umezawa's Jitte".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact],
            &["Equipment"],
        ),
        oracle_text: "Whenever equipped creature deals combat damage, put two charge counters on Umezawa's Jitte.\nRemove a charge counter from Umezawa's Jitte: Choose one —\n• Equipped creature gets +2/+2 until end of turn.\n• Target creature gets -1/-1 until end of turn.\n• You gain 2 life.\nEquip {2}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // TODO: Equip {2} cost AbilityDefinition — Equip keyword marker handles attachment,
            // but cost-bearing activated ability needs engine wiring.
            // TODO: "Whenever equipped creature deals combat damage" — put 2 charge counters.
            // TODO: "Remove a charge counter: Choose one" — modal activated with counter cost.
        ],
        ..Default::default()
    }
}
