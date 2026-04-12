// Throne of Eldraine — {5}, Legendary Artifact
// As Throne of Eldraine enters, choose a color.
// {T}: Add four mana of the chosen color. Spend this mana only to cast monocolored
// spells of that color.
// {3}, {T}: Draw two cards. Spend only mana of the chosen color to activate this ability.
//
// Status (post-PB-Q): partially unblocked.
//
// Now expressible:
//   - "As ~ enters, choose a color" — ReplacementTrigger::WouldEnterBattlefield +
//     ReplacementModification::ChooseColor (PB-Q / PB-X). See `caged_sun.rs` for the pattern.
//   - "{T}: Add four mana of the chosen color" — Effect::AddManaOfChosenColor { amount: 4 }.
//
// Still blocked (cannot author without producing wrong game state):
//   - **PB-Q5** (reserved): the produced-mana spending restriction "Spend this mana
//     only to cast monocolored spells of that color." Restriction must travel with the
//     produced mana into the player's pool (ManaRestriction extension). Authoring the
//     mana ability without the restriction would let players cast multicolored / off-color
//     spells with this mana — wrong game state.
//   - **PB-Q2** (reserved): the activation-time restriction "Spend only mana of the
//     chosen color to activate this ability" on the {3}{T}: Draw two cards ability.
//     Requires per-ability cost-payment filter that reads the source's chosen_color.
//     Authoring the activated ability without the restriction would let players draw
//     two cards using any mana — wrong game state.
//
// Both restrictions block authoring per the W6 "no wrong game state" policy. Once
// PB-Q2 + PB-Q5 land, all three abilities can be authored together.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("throne-of-eldraine"),
        name: "Throne of Eldraine".to_string(),
        mana_cost: Some(ManaCost { generic: 5, ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "As Throne of Eldraine enters, choose a color.\n{T}: Add four mana of the chosen color. Spend this mana only to cast monocolored spells of that color.\n{3}, {T}: Draw two cards. Spend only mana of the chosen color to activate this ability.".to_string(),
        // No abilities authored — both mana ability and draw ability require
        // spending restrictions (PB-Q2 / PB-Q5) that don't yet exist in the DSL.
        // The ChooseColor replacement is intentionally NOT authored alone, since
        // a Throne with a chosen color but no functioning {T} ability is also
        // wrong game state (its other abilities would be missing).
        abilities: vec![],
        ..Default::default()
    }
}
