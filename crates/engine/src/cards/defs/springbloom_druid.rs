// Springbloom Druid — {2}{G} Creature — Elf Druid 1/1.
// "When this creature enters, you may sacrifice a land. If you do, search your
// library for up to two basic land cards, put them onto the battlefield tapped,
// then shuffle."
//
// TODO: ETB trigger with optional sacrifice cost ("you may sacrifice a land. If
// you do, ...") is not expressible in the DSL. There is no AbilityDefinition::Triggered
// variant that can express an optional sacrifice as part of the trigger's resolution
// with a conditional follow-on effect. The pattern requires:
//   1. ETB trigger fires
//   2. Player chooses whether to sacrifice a land
//   3. If they did, search for up to two basic lands
// Neither the TriggerCondition nor Effect enums have an "optional sacrifice then
// conditional search" primitive. W5: wrong implementation omitted — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("springbloom-druid"),
        name: "Springbloom Druid".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "When this creature enters, you may sacrifice a land. If you do, search your library for up to two basic land cards, put them onto the battlefield tapped, then shuffle.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // TODO: ETB optional-sacrifice-then-search pattern not in DSL.
            // Needs: Effect::OptionalSacrifice { filter: land_filter, then: Box<Effect> }
            // or SpellAdditionalCost::OptionalSacrificeLand equivalent for triggered abilities.
        ],
        ..Default::default()
    }
}
