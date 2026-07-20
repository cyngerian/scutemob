# PB-RS3 roster review — 6 defs carrying `AtBeginningOfCombat`

<!-- last_updated: 2026-07-20 -->

Reviewer: `card-batch-reviewer` (2026-07-20, task `scutemob-145`). Every def below is, for the
first time, actually reachable at runtime — the `begin_combat` card-def sweep landed in
`95f1c306`. This review asks whether each def is faithful **now that it will actually execute**.

Sweep itself verified first (`rules/turn_actions.rs:1687-1768`): filters
`zone == Battlefield && is_phased_in()`, skips `controller != active` (so the "on your turn"
restriction is enforced engine-side and no def needs to encode it), uses
`effective_abilities(obj.is_transformed)` (PB-OS4b face-aware contract), pushes
`PendingTriggerKind::CardDefETB`.

## Verdicts

| Card | Verdict | Note |
| --- | --- | --- |
| `helm_of_the_host` | **explicit `Complete`** — endorsed | oracle-exact; integrity repair |
| `loyal_apprentice` | **flip → `Complete`** — endorsed, conditional on F3 | index-space discriminator |
| `siege_gang_lieutenant` | **flip → `Complete`** — endorsed, conditional on F3 | intervening-if both directions |
| `goblin_rabblemaster` | **blocker misframed — probe required** | see F-Rabble (HIGH) |
| `legion_warboss` | stays `partial` | blocker real; note undercounts by one gap (F4) |
| `mirage_phalanx` | stays `known_wrong` | containment holds; wrong in BOTH directions |

## Findings

### F3 (MEDIUM, engine-wide, pre-existing) — intervening-if is not checked at queue time

`intervening_if` is checked **only at resolution** (`resolution.rs:2125-2135`), never when the
trigger would go on the stack. **CR 603.4 requires both.** This is a deliberate engine-wide
convention documented at the upkeep sweep (`turn_actions.rs:265-266`) — **not** a PB-RS3
regression, and not specific to any card.

Divergent case: you do *not* control your commander at beginning of combat but regain control
before the trigger resolves (instant-speed control change, blink, phase-in). Real MTG never
triggers; this engine creates the Thopter. Reachable in 4-player Commander, but narrow. The
reverse case (control at trigger, lose before resolution) is handled correctly.

**Disposition**: the `Complete` flips on `loyal_apprentice` / `siege_gang_lieutenant` are endorsed
**conditional on treating this as an engine-wide convention rather than a per-card defect**. If
the `Complete` bar means "correct under CR 603.4 in all cases", neither card clears it — **and
neither does any other intervening-if card already shipped `Complete`**. Fixing it engine-wide is
squarely out of PB-RS3's scope (it touches every trigger sweep, not just `begin_combat`).
**Filed as a seed — see `rider-seed-triage-2026-07-19.md`.**

### F-Rabble (HIGH) — `goblin_rabblemaster`'s stated blocker looks already closed

The def's note claims the forced-attack clause "needs a new subtype-filtered must-attack
`GameRestriction` variant." **That framing is wrong.** The engine does not implement must-attack
as a `GameRestriction` at all — it uses `KeywordAbility::MustAttackEachCombat`, read from
**layer-resolved** characteristics at `combat.rs:378-390`. Every piece needed already exists:

- `LayerModification::AddKeyword(KeywordAbility::MustAttackEachCombat)` — `continuous_effect.rs:363`
- `EffectFilter::OtherCreaturesYouControlWithSubtype(SubType)` — `continuous_effect.rs:172`,
  already live in `galadhrim_brigade.rs`, `bloodline_keeper.rs`, `camellia_the_seedmiser.rs`
- `AbilityDefinition::Static { continuous_effect: ... }` with `WhileSourceOnBattlefield`

So "Other Goblin creatures you control attack each combat if able" appears **expressible today**
as a Layer 6 static grant. KI-3 stale/misframed TODO. Treat as a strong lead requiring a probe,
not a certainty — the open question is whether `AddKeyword` on the ability-grant layer composes
with `expect_characteristics` for a **non-source** object here.

### F1 (LOW, corpus-wide) — Equip target is unfiltered

`helm_of_the_host`'s Equip uses `TargetRequirement::TargetCreature`, but CR 702.6a is "target
creature **you control**". No `TargetCreatureYouControl` variant exists anywhere in the corpus (0
occurrences). Mitigated at resolution: `AttachEquipment` (`effects/mod.rs:4490-4497`) explicitly
requires `obj.controller == ctx.controller`, so attaching to an opponent's creature silently
no-ops rather than producing wrong board state. Residue is announcement legality only. **Does not
block `Complete`**; worth a seed for a filtered equip-target variant.

### F2 (LOW, accepted fallback) — permanent vs until-EOT haste

`loyal_apprentice`'s token carries haste on `TokenSpec.keywords` (permanent) vs the oracle's
"until end of turn". Unobservable — the token loses summoning sickness by the next turn anyway.
Consistent with the rest of the corpus.

### F4 (LOW) — `legion_warboss` note undercounts its gaps

Mentor is genuinely absent (0 matches for `KeywordAbility::Mentor` across all crates). But there
is a **second** live gap now that the trigger fires: "and attacks this combat if able" is
unimplemented, so the token can decline to attack. **Do not** "fix" this by putting
`MustAttackEachCombat` in `TokenSpec.keywords` — that is *each* combat, permanently, and would
over-restrict on every later turn. The note should name both gaps.

### `mirage_phalanx` — wrong in both directions, containment adequate

- **Oracle**: while paired, *each of the two paired creatures* has the combat trigger. Unpaired →
  **no trigger at all**.
- **Def**: trigger is unconditional on Mirage Phalanx itself (`intervening_if: None`), authored
  only on Phalanx.
- **Net effect now that the sweep is live**: an **unpaired** Phalanx wrongly creates a haste copy
  of itself every combat (pure card advantage from nothing), and a **paired** Phalanx
  under-produces by never granting the trigger to its partner. Wrong in both directions
  simultaneously.

Containment via `known_wrong` + `validate_deck` (SR-2) is genuinely sufficient — the card cannot
enter a game. **One thing to verify**: that no golden script or test fixture constructs Mirage
Phalanx directly via `ObjectSpec`, bypassing `validate_deck`.
