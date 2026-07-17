// Disciple of Freyalise // Garden of Freyalise — When this creature enters, you may sacrifice another creature. If you
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("disciple-of-freyalise"),
        name: "Disciple of Freyalise // Garden of Freyalise".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 3, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "When this creature enters, you may sacrifice another creature. If you do, you gain X life and draw X cards, where X is that creature's power.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![],
        completeness: Completeness::inert("Authorable now: WhenEntersBattlefield + MayPayThenEffect{cost: Cost::Sacrifice(creature, exclude self), then: Sequence[GainLife{PowerOfSacrificedCreature}, DrawCards{PowerOfSacrificedCreature}]}. Also needs the MDFC back face (Garden of Freyalise, Land)."),
        ..Default::default()
    }
}
