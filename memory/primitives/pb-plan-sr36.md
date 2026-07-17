# SR-36 implementation plan — SF-8 + SF-9 (both HIGH)

Task `scutemob-92`. Source findings: `memory/card-authoring/sr34-engine-findings-2026-07-17.md`
§SF-8 / §SF-9. Both pre-existing, both filed (not fixed) by `scutemob-90`.

## §0 — Rosters (built empirically from `all_cards()`, 2026-07-17, this task)

Built by a probe over `all_cards()` + `enrich_spec_from_def`, **never** a source regex
(CLAUDE.md: the `abilities:\s*vec!\[\s*\]` regex also matches `mana_abilities: vec![]`;
that trap has fired three times on this campaign).

### SF-8 roster — 9 ability rows, 6 cards (SMALLER than filed)

| Card | ab | marker | cost | note |
|---|---|---|---|---|
| Gaea's Cradle | 0 | Complete | `Tap` | bare tap — buggy today |
| Elvish Archdruid | 1 | Complete | `Tap` | buggy today |
| Priest of Titania | 0 | Complete | `Tap` | buggy today |
| Marwyn, the Nurturer | 1 | Complete | `Tap` | buggy today |
| Circle of Dreams Druid | 0 | Complete | `Tap` | buggy today |
| Howlsquad Heavy | 1 | KnownWrong | `Tap` | buggy today; marker is for the speed-gate, unrelated |
| Cabal Coffers | 0 | Partial | `Sequence([Mana{2}, Tap])` | correct via stack today (Finding-A exclusion) |
| Cabal Stronghold | 1 | Partial | `Sequence([Mana{3}, Tap])` | correct via stack today |
| Crypt of Agadeem | 2 | Partial | `Sequence([Mana{2}, Tap])` | correct via stack today |

**Falsified**: the finding's speculative list — Everflowing Chalice, Elvish Guidance,
Brightstone Ritual, Battle Hymn, Black Market — do **not** use `Effect::AddManaScaled`.
The finding explicitly flagged them "re-check each against the registry — not re-verified
in this task". They are not in scope. Do not touch them.

### SF-9 roster — 28 ability rows (MUCH wider than filed; 14 rows on `Complete` defs)

The finding named two victims and said "a full corpus scan is still owed". It is owed no
longer. The headline is **not** Staff of Compleation:

**The entire fetchland cycle is `Complete` and pays no life today** — cost
`Sequence([Tap, PayLife(1), SacrificeSelf])`, effect `Sequence([SearchLibrary, Shuffle])`,
which `try_as_tap_mana_ability` correctly declines, so it routes to
`handle_activate_ability` where `flatten_cost_into` drops the life:

Arid Mesa, Bloodstained Mire, Flooded Strand, Marsh Flats, Misty Rainforest,
Polluted Delta, Prismatic Vista, Scalding Tarn, Verdant Catacombs, Windswept Heath,
Wooded Foothills. **All 11 `Complete`.** Eleven of the most-played lands in the format,
deck-legal today, each a free fetch.

Other `Complete` victims: Doom Whisperer (ab[2], 2 life, Surveil — free repeatable
surveil), Razaketh the Foulblooded (ab[2], 2 life, Sequence — free repeatable tutor),
Warren Soultrader (ab[0], 1 life, CreateToken).

Non-`Complete` victims (still must end truthful): Aetherflux Reservoir (Partial, ab[0],
50 life, DealDamage, **has a target**), Gnarlroot Trapper (Partial, ab[0],
AddManaRestricted), Staff of Compleation (KnownWrong, ab[0]/[2]/[3]), Voldaren Estate
(KnownWrong, ab[1], AddManaAnyColorRestricted).

Already correct (lowered into a `ManaAbility` with `life_cost` by SR-34, do not
double-charge): Fiery Islet ab[0]/[1], Nurturing Peatland ab[0]/[1], Silent Clearing
ab[0]/[1], Mana Confluence ab[0], Staff of Compleation ab[1].

**The two paths are disjoint by construction** — `mana_ability_lowering` returning `Some`
is exactly the condition for the ability to be excluded from `activated_abilities`. Verify
this in the implementation rather than assuming it: a double-charge would be a new HIGH.

## §1 — SF-8 fix

`handle_tap_for_mana` (`rules/mana.rs`) needs an `EffectAmount` resolution context inside
the stackless `TapForMana` path. It has one available: `resolve_amount(state, amount, ctx)`
(`effects/mod.rs:6638`, `pub(crate)`, takes `&GameState`) and
`EffectContext::new(controller, source, vec![])`. `rules/mana.rs` is in the engine crate,
so `pub(crate)` is reachable. `fire_mana_triggered_abilities` in this same file already
constructs an `EffectContext` this way — follow that precedent.

1. **`ManaAbility` gains `scaled_amount: Option<EffectAmount>`** (`card-types/src/state/game_object.rs`),
   `#[serde(default)]`, mirroring how SR-34 added `mana_cost` / `life_cost`. Doc it against
   CR 605.1a. `EffectAmount` already lives in `card-types` (`cards/card_definition.rs`), so
   no new crate dependency and no SR-6 arrow violation — **verify this**, it is the one
   thing that could sink this design.
2. **`try_as_tap_mana_ability`'s `AddManaScaled` arm** (`replay_harness.rs:3808`) keeps
   `produces = {color: 1}` — the colour channel SR-33's
   `every_complete_land_registers_each_printed_tap_mana_color` gate reads — and additionally
   sets `scaled_amount: Some(count.clone())`. Delete the "actual production is dynamic"
   comment, which is now true instead of aspirational.
   - **`player: PlayerTarget` must be checked.** The arm currently ignores it. A mana
     ability that adds mana to someone *other* than the controller cannot be lowered (the
     stackless path always pays the activating player). Refuse to lower unless it is
     `PlayerTarget::Controller` (or whatever the corpus uses — check; all 6 use
     `Controller`). Returning `None` leaves it on the stack, which is correct-but-slow, not
     wrong.
3. **`handle_tap_for_mana` step 8** evaluates it. Where `produces` is read
   (`for (color, base_amount) in &ability.produces`), if `scaled_amount` is `Some(amt)`,
   the per-colour amount is `resolve_amount(state, amt, &ctx).max(0) as u32` instead of
   `base_amount`. Build `ctx` = `EffectContext::new(player, source, vec![])`.
   - **Step 7b's `base_preview` must use the resolved amount too**, not the literal 1 —
     it feeds `apply_mana_production_replacements`' colour-filter check, and the multiplier
     result multiplies the real amount. Nyxbloom Ancient on Gaea's Cradle with 3 creatures
     must be 9 green, not 3.
   - **Ordering hazard**: resolve the amount BEFORE step 6 taps the source if and only if
     the count could include the source itself. Gaea's Cradle counts creatures; the Cradle
     is a land, so tapping is invisible to it. But Circle of Dreams Druid / Priest of
     Titania / Elvish Archdruid / Marwyn **are creatures counting creatures** — and tapping
     a creature does not remove it from the battlefield, so `PermanentCount` is unaffected.
     Confirm by reading `resolve_amount`'s `PermanentCount` arm that it does not filter on
     untapped. If it does, resolve before the tap. **Do not assume — read it.**
   - `.max(0)` matters: `resolve_amount` returns `i32`.
4. **Delete the Finding-A exclusion** (`replay_harness.rs:3755-3758`, the
   `is_bare_tap` check) and its doc paragraph in `mana_ability_lowering`'s comment. This is
   what widens Cabal Coffers / Cabal Stronghold / Crypt of Agadeem into real mana abilities.
   Their `{2},{T}` / `{3},{T}` mana component is already handled by SR-34's `mana_cost`.
   - After deleting, **verify by activation** that each of the three produces the right
     scaled amount AND charges its generic cost. If any cannot be made correct, mark it
     truthfully rather than shipping it Complete-but-broken.
   - Their markers are `Partial` today. If the widening makes them correct, they may be
     upgradeable — but check the *whole* def (Crypt of Agadeem has 3 abilities). A marker
     upgrade needs every clause to work, per the `megrim.rs` calibration case.

## §2 — SF-9 fix

Mirror SR-34's `ManaAbility::life_cost` exactly; the precedent is 20 lines of `rules/mana.rs`.

1. **`ActivationCost` gains `life_cost: u32`** (`card-types/src/state/game_object.rs`),
   `#[serde(default)]`. CR 118.3 / 119.4.
2. **`flatten_cost_into`** (`replay_harness.rs:3915`): `Cost::PayLife(n) => ac.life_cost += n`.
   **`+=`, not `=`** — a `Sequence` can hold more than one and the walk is recursive.
   Delete the "no ActivationCost representation yet" comment.
3. **`handle_activate_ability`** (`rules/abilities.rs`): legality check then payment,
   mirroring `handle_tap_for_mana` steps 5b / 6b verbatim, including the **CR 119.4b
   short-circuit** (`if ability_cost.life_cost > 0` — a player at any life total may always
   pay 0; an unguarded `>=` would break that). Use `GameStateError::InsufficientLife`
   (already exists — SR-34 added it) and `GameEvent::LifeLost`.
   - Put the **legality check before** the mana-cost block at line 525, and the **payment
     after** it (line ~573, before the discard/sacrifice steps). Rationale: keeps the
     "validate everything payable, then pay" shape mana.rs documents. An `Err` discards the
     whole state anyway (`process_command` takes `GameState` by value and only returns it on
     `Ok`), so this is about legibility, not correctness — say so in the comment rather than
     overclaiming a transactional guarantee.
   - CR 601.2h: tap/mana/life/sacrifice may be paid in any order; none of their legality
     depends on another's result. Cite it, as mana.rs does.

## §3 — Card-def reconciliation

- The 11 fetchlands, Doom Whisperer, Razaketh, Warren Soultrader: the fix makes them
  correct. They stay `Complete`. **Prove by activation, not by inspection** — assert the
  life total moves.
- Aetherflux Reservoir (`PayLife(50)`, has a target): the fix charges it. Its `Partial`
  marker is about something else — read the note, leave it or correct it, do not upgrade
  on the strength of this fix alone.
- Staff of Compleation stays `KnownWrong` (its colour bug survives), but its note must stop
  claiming a free proliferate/draw once they are no longer free. **Rewrite the note to the
  real remaining blocker.**
- Voldaren Estate / Gnarlroot Trapper: these are *mana* abilities that fall into the
  non-mana path because `AddManaAnyColorRestricted` / `AddManaRestricted` have no
  `try_as_tap_mana_ability` arm. After SF-9 they at least *pay*. Do **not** add arms for
  them — `any_color` produces colorless (SF-11/SF-12), so lowering them would trade a
  cost bug for a colour bug. Ensure their markers name the real blocker.
- **Every card on both rosters must end either correct-and-proven or truthfully marked.**

## §4 — Version bumps (machine-forced — do not hand-pick)

`ManaAbility` and `ActivationCost` are inside `Characteristics`, which is inside the SR-8
protocol fingerprint's transitive closure. Adding a field to either is a **wire change**.

- `PROTOCOL_VERSION` bump + `PROTOCOL_SCHEMA_FINGERPRINT` re-pin + history row.
- `HASH_SCHEMA_VERSION` bump (currently 41) + its sentinel tests.
- Do not guess the numbers. Run the gates; they will tell you. `tests/protocol_schema.rs`
  recomputes the fingerprint from source and fails with the correct digest.

## §5 — Tests (`tests/primitives/primitive_sr36_scaled_mana_and_life_costs.rs`)

**Assert amounts and life totals via activation. Never shape.** SF-8 existed *because* two
tests asserted registration shape and never activated — the finding says so in its own
words ("a data-model test can pin a defect as a requirement"). Do not add a tenth
consecutive vacuous checker.

- Gaea's Cradle: 0 creatures → 0 green; 3 creatures → 3 green. **The 0 case is the one that
  distinguishes the fix from the bug** in the wrong direction — pre-fix it produced 1.
- Elvish Archdruid (counts Elves) with a non-Elf on board → counts only Elves.
- Cabal Coffers: N Swamps → N black, AND `{2}` actually leaves the pool.
- A fetchland: life 40 → 39 on activation.
- Doom Whisperer: life 40 → 38.
- Staff of Compleation ab[2] (proliferate): life 40 → 37. ab[3] (draw): 40 → 36.
- CR 119.4b: a `life_cost: 0` ability activates at negative life. And insufficient life →
  `InsufficientLife`, state untouched.
- **Non-vacuity**: for each new test, revert the fix locally and confirm it fails. Record
  which assertion caught it. A test that passes both ways is not a test.

## §6 — Seam tests to update (they name themselves)

- `tests/casting/mana_filter.rs::test_add_mana_scaled_registered_as_mana_ability` — its
  SF-8 doc-comment note is now stale. It says "if they activated and asserted the mana, they
  would fail". They should now activate and assert, and pass.
- `::test_add_mana_scaled_orphan_fix_all_cards` — same; also its trailing note explaining
  why Coffers/Stronghold/Crypt are excluded is now false. They belong in the list.
- `tests/primitives/primitive_sr34_composite_mana_costs.rs::composite_cost_add_mana_scaled_stays_on_the_stack`
  — this test **pins the exclusion I am deleting** and says explicitly to delete it when
  SF-8 lands. Delete or invert it; do not leave it asserting the old behaviour.
- `tests/core/effect_choose_gate.rs::printed_tap_mana_colors` — its doc comment documents
  the amount blind spot. Re-read it; the blind spot may now be covered elsewhere.

## §7 — Gates

`cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35: `cargo fmt` checks zero card
defs); `cargo clippy --all-targets -- -D warnings`; `cargo build --workspace` (SR-3 seal —
`test`/`clippy` unify `test-util` and cannot see the break); `cargo test --all`
(baseline 3300). `python3 tools/authoring-report.py` to reconcile coverage.

## §8 — Out of scope (file, do not fix)

SF-10 (`ManaAbility` has no `activation_condition`), SF-11 (`any_color` → colorless),
SF-12 (the colour gate is blind to any-color lands), EF-13 (105 `partial` defs that are
`Inert` by taxonomy). If SF-8/SF-9 work turns up new engine findings, file them in
`memory/card-authoring/sr36-engine-findings-2026-07-17.md` for the next SR.
