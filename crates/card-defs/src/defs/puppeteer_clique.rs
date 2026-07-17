// Puppeteer Clique — {3}{B}{B}, Creature — Faerie Wizard 3/2
// Flying
// When this creature enters, put target creature card from an opponent's graveyard onto
// the battlefield under your control. It gains haste. At the beginning of your next end
// step, exile it.
// Persist
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("puppeteer-clique"),
        name: "Puppeteer Clique".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: creature_types(&["Faerie", "Wizard"]),
        oracle_text: "Flying\nWhen Puppeteer Clique enters, put target creature card from an opponent's graveyard onto the battlefield under your control. It gains haste. At the beginning of your next end step, exile it.\nPersist".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // PARTIAL TODO: ETB reanimate from opponent's GY — DSL gap (no MoveFrom graveyard
            // to battlefield under your control with haste grant). The delayed exile part
            // (at beginning of your next end step) is now expressible via ExileWithDelayedReturn
            // with AtOwnersNextEndStep, but the reanimate source targeting an opponent's GY
            // is not yet in the DSL.
            AbilityDefinition::Keyword(KeywordAbility::Persist),
        ],
        completeness: Completeness::partial("Re-verify before authoring: TargetCardInGraveyard (PB-10) + Effect::MoveZone(Battlefield) + ExileWithDelayedReturn may now cover most of the ETB. Open links: (a) does MoveZone put a targeted graveyard card onto the battlefield under YOUR control; (b) can the target be scoped to an opponent's graveyard; (c) granting haste to the reanimated object (new object per CR 400.7)."),
        ..Default::default()
    }
}
