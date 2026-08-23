# PB-DX15a — execution notes

**Task**: `scutemob-216` · v4 queue rank 3 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 3)
**Seeds**: `OOS-DP9-8` (CR 608.2e/701.22c/701.23i APNAP) + `OOS-DP9-11` (CR 400.7 same-zone renumber)
**Riders named by the memo**: `OOS-DX24-1`, `OOS-DX24-7`
**Explicitly NOT taken**: `OOS-DP9-16` (parked — unreachable by construction; there is no PB-DX15b)

---

## §0 — Wire prediction, written BEFORE any code changed

> This section was written and committed before a single non-test source line moved.
> Commit order is the evidence: this file's first commit precedes every code commit
> on the branch. The v4 memo's row 3 cell says **none (HIGH)** for both halves.

**Prediction: `PROTOCOL_SCHEMA_FINGERPRINT` UNMOVED and `HASH_SCHEMA_VERSION` UNMOVED,
for both halves.** Derivation, per half, rather than inherited from the memo:

**APNAP half (`OOS-DP9-8`).** The change reorders the *iteration* of an existing
`Vec<PlayerId>` produced by `effects::resolve_player_target_list` and its siblings. It
adds no type, no enum variant and no struct field. `PlayerTarget` is unchanged;
`EffectChoiceQuestion` / `EffectChoiceAnswer` are unchanged; no `Command` or `GameEvent`
gains or loses a shape. `rules::abilities::apnap_order` already exists and is already
called from `rules/engine.rs:1617` and `rules/abilities.rs:8374`, so nothing new becomes
reachable from a closure root. **HASH**: `hash.rs` hashes declared *shapes*; a different
ordering of the same player ids changes the runtime `public_state_hash` **value** on
affected games but not the *schema*, and `HASH_SCHEMA_VERSION` gates the schema.

**Same-zone half (`OOS-DP9-11`).** Replacing a same-zone `move_object_to_zone` /
`move_object_to_bottom_of_zone` with a `Zone::reposition_within`-style permutation
mutates `GameState.zones` (an existing field of an existing type) and *declines* to
mutate `objects` / `timestamp_counter`. `Zone` is unchanged. No new type.

**Stop condition (binding).** If either gate moves, that is a signal to **stop and
re-scope**, not to edit the pin. Both gates are executed after the implement phase and
the measured value is recorded in §7 below, taken from the gate's own output.

**What WILL move, and is budgeted rather than discovered:**
- Golden scripts whose per-step assertions read `ObjectId`s across a same-zone reorder,
  or whose event order is per-player (APNAP reorders the questions/events).
- SR-9b per-step fingerprints, for both reasons: fewer `ObjectId`s minted (the same-zone
  half) and a different event order (the APNAP half).
- Any seeded fixture whose shuffle/coin-flip outcome depends on `timestamp_counter`,
  because the same-zone half stops consuming values from it.

Every moved pin is listed **by name with its CR reason** in §6. A pin repaired by
weakening an assertion is a defect, not a repair.

---

## §1 — Census (both populations are FLOORS — dispatch hygiene 6)

Both populations were re-derived at HEAD by an inverse method. Both are **printed by a
test** (`crates/engine/tests/core/pb_dx15a_same_zone_identity_roster.rs` and
`crates/engine/tests/core/pb_dx15a_apnap_choice_roster.rs`) rather than transcribed into
this file, so a reader can re-derive rather than trust. The numbers below are the
figures those tests print.

### §1.1 — Same-zone renumber (`OOS-DP9-11`): the filed 5 is a FLOOR, short by 12

The v4 memo and the registry row both name **five** deck-legal `Complete` defs
(`birthing_ritual`, `chaos_warp`, `goblin_ringleader`, `growing_rites_of_itlimoc`,
`sylvan_messenger`). All five reproduce at HEAD — none is stale — **and they are one of
FOUR engine mechanisms that reach a same-zone move.** Measured population: **17**
deck-legal `Complete` defs, plus 5 more that would reach the moment they are promoted.

| family | engine site | deck-legal `Complete` members | in the memo? |
|---|---|---|---|
| A — `RevealAndRoute` / `LookAtTopThenPlace` routing to a library | `effects/mod.rs` `:6303` (bottom), `:6305` (**top**), `:6456` | **5** — the memo's list, exactly | yes |
| B — `SearchLibrary` with `destination: ZoneTarget::Library` | `effects/mod.rs:4153` | **8** — `elvish_harbinger`, `enlightened_tutor`, `forerunner_of_the_legion`, `imperial_seal`, `insatiable_avarice`, `mystical_tutor`, `vampiric_tutor`, `worldly_tutor` | **no** |
| C — `KeywordAbility::Hideaway` | `resolution.rs:6350` | **1** — `windbrisk_heights` | **no** |
| D — `KeywordAbility::PartnerWith` | `resolution.rs:6429` | **3** — `brallin_skyshark_rider`, `pir_imaginative_rascal`, `toothy_imaginary_friend` | **no** |

Three corrections to the filed row, each of which matters to someone reading it:

1. **`chaos_warp` reaches the `LibraryPosition::Top` branch (`:6305`), not the bottom
   helper.** The row is filed against `move_object_to_bottom_of_zone`; Chaos Warp's
   revealed non-permanent card stays on **top** and is renumbered by
   `move_object_to_zone`. A sweep scoped to the bottom helper alone would have missed
   the memo's own third-named def.
2. **Family D's blast radius is not "1-3 cards", it is the whole library.**
   `resolution.rs:6429` permutes `zone.object_ids()` for the *entire* target library by
   moving every id to the bottom in turn — so a 99-card Commander library used to mint
   **99** fresh `ObjectId`s and burn **99** `timestamp_counter` values on **every**
   Partner-With ETB, invalidating every pre-existing library id in the game. It runs
   *"whether found or not"* (`:6404-6412`), i.e. unconditionally.
3. **The two censuses do not nest.** An oracle-text axis over all 1,803 defs (the
   printed shapes: "put the rest on the bottom of your library", "look at the top N")
   finds family A and **cannot see** families B/C/D — a tutor's printed text never says
   "put the rest on the bottom", and a keyword confers through
   `AbilityDefinition::Keyword`, not through an `Effect` payload. Conversely the
   structural axis over `Effect` payloads cannot see family C/D at all. This is
   PB-DX26's and PB-DX43's lesson arriving a third time: **a roster derived from one
   declaration construct measures that construct.** Both axes are standing roster rows.

**Why a per-caller sweep was rejected.** `Effect::MoveZone` and `Effect::PutOnLibrary`
resolve their destination from a `ZoneTarget` **at runtime**, so "is this call
same-zone" is not a property of the call site at all — it is a property of the card that
happens to be resolving. The guard therefore lives inside the two `GameState` move
helpers, which makes a renumbering same-zone move *unrepresentable* rather than merely
absent. That is the difference between a sweep and a fix, and it is why the criterion's
"or a sibling" clause was taken.

### §1.2 — APNAP (`OOS-DP9-8`): the seed's own pin was VACUOUS, and its headline family is mis-framed

**(a) The recorded deviation was recorded in the one configuration that cannot express it.**
`test_dp9_choice_inside_for_each_each_player` carried a "Recorded deviation (OOS-DP9-8)"
block ending *"Not fixed here; this test asserts the order the engine actually has."* It
ran on `fixture` — two seats, `.active_player(p(1))` — and asserted `vec![p(1), p(2)]`.
`GameStateBuilder` seeds `turn_order` in `add_player` call order, which is ascending in
every fixture in this repository, so **APNAP starting from the lowest `PlayerId` IS
ascending `PlayerId`**: rotating a list to start at its first element is the identity.
The pin would have stayed green under either rule. That is why the seed survived from
PB-DP9 (`scutemob-157`) to here — the suite reported a pinned deviation while pinning
nothing.

The same vacuity affects `fixture_3p` (also `.active_player(p(1))`) and
`pb_eng1_effect_discard_choice.rs`'s 3-player discard-order test, whose prose at `:417`
says the engine "iterates players ascending". Four stale or unfalsifiable APNAP claims
in the tree, not one.

It is now stated **structurally** rather than left as a discovered fact:
`test_dx15a_active_lowest_id_makes_apnap_and_ascending_indistinguishable` asserts the
coincidence over 2..=6 seats, plus the contrasting non-vacuous case, so the same mistake
cannot recur silently.

**(b) The seed names ONE function; there are seven order-sensitive sites.**

| site | file | status |
|---|---|---|
| `resolve_player_target_list` — `EachPlayer` / `EachOpponent` | `effects/mod.rs:8201` | **the seed's only named site**; fixed |
| `resolve_effect_target_list` — `EffectTarget::EachPlayer` / `EachOpponent` | `effects/mod.rs:~7999` | independently reachable, same `OrdMap`, same defect; fixed |
| `Effect::ForEach` | `effects/mod.rs:4520` | delegates to the above; fixed transitively |
| `Effect::LivingDeath` | `effects/mod.rs:7511` | fixed — see (c) |
| `Effect::ReturnAllFromGraveyardToBattlefield` | `effects/mod.rs:7378` | fixed; the walk decides the order permanents enter and therefore the order ETB triggers are queued |
| `Effect::Manifest` / `Effect::Cloak`, `EachOpponent` arm | `effects/mod.rs:5116`, `:5182` | **NOT fixed — see §4.3**; these collapse "each opponent" to a single `.keys().find(..)` pick, a different and larger defect |
| `resolve_cda_player_target` | `layers.rs:3093` | **deliberately not fixed** — CDA context, order-insensitive (its consumers count and sum); documented at the function |

Fixing only the function the seed named would have left `resolve_player_target_list` and
`resolve_effect_target_list` **disagreeing about the order of the same set of players**.

**(c) Two comments asserted APNAP that the code did not implement.** `Effect::LivingDeath`
read `state.players.keys().copied().collect()` then `.sort()` under a comment saying
*"Determine APNAP player order (active player first, then in turn order)"*; the
`Effect::WheelHand` arm said *"APNAP order comes from `resolve_player_target_list`'s
`PlayerTarget::EachPlayer` iteration"*. The `OOS-DX28-6` note-vs-code shape, twice,
**inside this batch's own subject matter**. Both are now true, and both are called out
in-source rather than quietly corrected.

**(d) The memo's headline family is mis-framed, and the correction is the load-bearing
part.** Row 3 says the batch "repairs the Fleshbag/Grave Pact family (10 defs)", which
reads as ten defs regaining a per-player choice. Measured:
`effects::sacrifice_permanents_for_player` (`effects/mod.rs:9479-9492`) computes
`eligible_sacrifice_targets`, **sorts, and takes the first `n`** — it asks nothing. So
`Effect::SacrificePermanents { player: EachPlayer }` has no per-player *question* at all,
and what this batch fixes for that family is the **order the sacrifices happen** (event
order, and hence trigger-queueing order), not a choice. The agency gap — nobody chooses
which creature — is a separate, pre-existing defect that PB-DX15a does **not** close, and
the probe covering that family says so in its own doc rather than implying otherwise.

Only **2** deck-legal `Complete` defs (`burglar_rat`, `geier_reach_sanitarium`) exercise
the seed's literal per-player-question claim; 3 more (`echo_of_eons`,
`whirlpool_warrior`, `winds_of_change`) reach it through the shared RNG counter.

**(e) Neither CR violation was pinned anywhere in the 4,797-test suite.** A full
`--workspace --no-fail-fast` run with **both** engine halves in place is **4,797 / 0 / 5**
— zero failures, no golden script moved, no SR-9b fingerprint moved, no seeded constant
re-observed. §0 budgeted that movement and **none came due**; that is stated here as a
paid-and-unclaimed budget rather than quietly dropped. The consequence for this batch is
strict: there is **no inherited red-before evidence anywhere**, so every probe must be
proven red by an executed revert (§5).

## §2 — APNAP half

## §3 — Same-zone half

## §4 — Rider dispositions (`OOS-DX24-1`, `OOS-DX24-7`)

## §5 — Revert matrix

## §6 — Moved pins, by name, with the CR reason

## §7 — Gates, measured

