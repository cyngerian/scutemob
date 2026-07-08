# Card Review: PB-AC3 backfill + PARTIAL spot-check

**Reviewed**: 2026-07-08 (card-batch-reviewer)
**Cards**: 7 backfilled + 4 PARTIAL spot-checks
**Findings**: 4 HIGH, 1 MEDIUM (none), 3 LOW

## Backfilled CLEAN (verified oracle-accurate)
- **Keep Watch** — `DrawCards { AttackingCreatureCount { EachPlayer, None } }`. CLEAN.
- **Throne of the God-Pharaoh** — `AtBeginningOfYourEndStep` → `LoseLife { EachOpponent, TappedCreatureCount { Controller } }`. CLEAN.
- **Mirror Entity** — 7b `SetBothDynamic{XValue}` + Layer 4 `TypeChange` `AddAllCreatureTypes`. CLEAN (layer fix confirmed correct).
- **Krenko, Tin Street Kingpin** — `AddCounter` then `CreateToken(PowerOf(Source))`. Correct. LOW F1: oracle_text expands first name reference (cosmetic).
- **Ulvenwald Hydra** — `*/*` (None/None) + `CdaPowerToughness{PermanentCount{Land}}` + ETB search. Correct. LOW F2: mandatory "may search" (project convention).
- **Storm-Kiln Artist** — `CdaModifyPowerToughness{PermanentCount{Artifact}}` + Magecraft→Treasure. Correct. LOW F4: copy-half of magecraft unimplemented (documented, consistent w/ archmage_emeritus).

## HIGH findings (wrong game state)
- **F3 (Wight of the Reliquary)**: activated `{T}, Sacrifice(TargetFilter{Creature})` drops `exclude_self` → can sacrifice itself to "sacrifice ANOTHER creature." Project precedent `vampire_gourmand.rs` OMITS the ability for the identical gap. PB-AC3 CDA part is fine.
- **F5 (Mishra, Claimed by Gix)**: ships a "KNOWN-WRONG placeholder" `Fixed(1)` drain on `WheneverYouAttack`; oracle X = number of attacking creatures = now expressible via `AttackingCreatureCount`. Also `DeclaredTarget{0}` with empty targets. Should author drain correctly, TODO only Meld.
- **F6 (Ashaya, Soul of the Wild)**: declared `*/*` (None/None) with NO CDA → resolves 0/0, dies to SBA. Plus type-grant statics use `CreaturesYouControl` (includes tokens); oracle is "Nontoken creatures". CDA now expressible via `CdaPowerToughness{PermanentCount{Land}}`.
- **F7 (Multani, Yavimaya's Avatar)**: base 0/0 + Reach/Trample only, NO pump → dies to SBA. Pump now expressible via `CdaModifyPowerToughness{Sum(PermanentCount{Land}, CardCount{Graveyard,Land})}`. Only GY-return activated ability genuinely blocked.

## Galadhrim Ambush — CLEAN (correctly fully-blocked `vec![]`)

## Verdict: NOT READY TO CLOSE until 4 HIGH addressed

## RESOLUTION (2026-07-08) — all 4 HIGH fixed (commit d771b795)
- **F5 Mishra**: replaced KNOWN-WRONG `Fixed(1)` drain with `AttackingCreatureCount{EachPlayer,None}` on both lose-life (EachOpponent) and gain-life (Controller); Meld stays TODO.
- **F7 Multani**: added `CdaModifyPowerToughness{ Sum(PermanentCount{Land}, CardCount{Graveyard,Land}) }` on both axes (was a dying base 0/0); GY-return activated ability stays TODO.
- **F6 Ashaya**: added `CdaPowerToughness{PermanentCount{Land}}` (was a dying */* with no CDA); REMOVED the two overbroad `AddCardTypes(Land)`/`AddSubtypes(Forest)` statics that wrongly animated tokens (oracle "Nontoken") — TODO nontoken-scoped `EffectFilter`.
- **F3 Wight**: omitted the `{T},Sacrifice(TargetFilter{Creature})` ability (could self-sacrifice to "another creature"; no `Cost::SacrificeAnother`) per `vampire_gourmand.rs` precedent; Vigilance + CDA modify retained.
- +2 regression tests (`test_ashaya_pt_equals_lands_you_control`, `test_multani_pt_sums_lands_and_graveyard_lands`) confirm the previously-dying bodies compute correct P/T.
- LOW findings (Krenko oracle text, Ulvenwald may-search, Storm-Kiln copy-magecraft, authoring-report.py Krenko regex) left as pre-existing/out-of-scope.
- **Verdict: RESOLVED — ready to close.**
