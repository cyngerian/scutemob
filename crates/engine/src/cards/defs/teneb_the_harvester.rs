// Teneb, the Harvester — {3}{W}{B}{G}, Legendary Creature — Dragon 6/6
// Flying
// Whenever Teneb deals combat damage to a player, you may pay {2}{B}. If you do,
// put target creature card from a graveyard onto the battlefield under your control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("teneb-the-harvester"),
        name: "Teneb, the Harvester".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, black: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon"],
        ),
        oracle_text: "Flying\nWhenever Teneb deals combat damage to a player, you may pay {2}{B}. If you do, put target creature card from a graveyard onto the battlefield under your control.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 603.1: Triggered — combat damage to player → optional mana payment → return.
            // DSL GAP (PB-10 Finding 5): "you may pay {2}{B}. If you do, ..." requires an optional
            // mana payment on triggered abilities (Cost on triggers or Effect::PayManaOrElse), which
            // does not exist in the DSL yet. The trigger fires unconditionally, returning a creature
            // from any graveyard to the battlefield without the {2}{B} payment. This produces wrong
            // game state — Teneb should only reanimate when the controller pays the additional cost.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Battlefield { tapped: false },
                    // "under your control" — override controller to the Teneb owner.
                    controller_override: Some(PlayerTarget::Controller),
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCardInGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                })],
            },
        ],
        ..Default::default()
    }
}
