// Nether Traitor — {B}{B}, Creature — Spirit 1/1; Haste, Shadow.
// Whenever another creature is put into your graveyard from the battlefield, you may pay
// {B}. If you do, return this card from your graveyard to the battlefield.
//
// CR 113.6m: this ability functions ONLY from the graveyard — its effect moves the card out of
// the graveyard, and its trigger condition does not put it there. On the battlefield, another
// creature dying does nothing. That is what `trigger_zone: Some(TriggerZone::Graveyard)` below
// records, and the def has been right about it since it was written.
//
// PB-DX24 (`scutemob-202`, 2026-08-05) is when the ENGINE started reading it. Before that the
// lowering (`testing::replay_harness::build_face_ability_vectors`) dropped `trigger_zone` in
// 33 of its 34 trigger arms, so this ability was installed on the battlefield object and was
// never dispatched from the graveyard at all — the card functioned from exactly the wrong zone.
// Seeded as `OOS-DX1-3`, closed by PB-DX24.
//
// CR 603.10a (Gatherer, Nether Traitor): if Nether Traitor and another creature are put into
// your graveyard at the same time, this ability does NOT trigger — a leaves-the-battlefield
// ability looks back in time, and immediately prior to the event this card was on the
// battlefield, where (CR 113.6m) the ability did not function. Enforced by
// `rules::abilities::check_triggers`'s `arrived_in_graveyard_this_batch` set.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nether-traitor"),
        name: "Nether Traitor".to_string(),
        mana_cost: Some(ManaCost {
            black: 2,
            ..Default::default()
        }),
        types: creature_types(&["Spirit"]),
        oracle_text: "Haste\nShadow (This creature can block or be blocked by only creatures with \
                      shadow.)\nWhenever another creature is put into your graveyard from the \
                      battlefield, you may pay {B}. If you do, return this card from your \
                      graveyard to the battlefield."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            AbilityDefinition::Keyword(KeywordAbility::Shadow),
            // CR 603.3 (TriggerZone::Graveyard) / CR 118.12: "Whenever another creature is put
            // into your graveyard from the battlefield, you may pay {B}. If you do, return this
            // card from your graveyard to the battlefield."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                // PB-DX28 (closes OOS-DX4-1): Oracle "put into YOUR graveyard" is an
                // ownership condition (CR 404.3 — the graveyard's owner is the card's
                // owner), previously approximated here as `controller: Some(You)`
                // because the DSL had no owner-scoped death trigger. The two diverge
                // under any gain-control effect: a creature you OWN but an opponent
                // controls dying to YOUR graveyard should fire (the approximation
                // didn't), and one you control but do not own dying should NOT fire
                // (the approximation did). `TriggerCondition::WheneverCreatureDies.owner`
                // now exists (lowered into `DeathTriggerFilter::owner_you`/
                // `owner_opponent`, enforced at both the battlefield and
                // graveyard-zone dispatch sites in `rules::abilities`), so this is
                // authored as the printed clause directly: `controller: None`,
                // `owner: Some(TargetOwner::You)`.
                //
                // Note: this def's OWN prior note cited `athreos` and `fecundity` as
                // further instances of this corpus convention. `athreos` is one;
                // `fecundity` is NOT — its printed clause is "that creature's
                // CONTROLLER may draw a card" (a controller gap, not an ownership
                // approximation), as `fecundity.rs`'s own `partial` note already says.
                // That citation was wrong and is corrected here, not repeated.
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: None,
                    owner: Some(TargetOwner::You),
                    exclude_self: true,
                    nontoken_only: false,
                    filter: None,
                },
                effect: Effect::MayPayThenEffect {
                    cost: Cost::Mana(ManaCost {
                        black: 1,
                        ..Default::default()
                    }),
                    payer: PlayerTarget::Controller,
                    then: Box::new(Effect::MoveZone {
                        target: EffectTarget::Source,
                        to: ZoneTarget::Battlefield { tapped: false },
                        controller_override: None,
                    }),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: Some(TriggerZone::Graveyard),
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
