# PB-DX20b — REACHABILITY probes: executed revert matrix

Scope: the two files this task owns.

* `crates/simulator/tests/pb_dx20b_enchant_offer_channel.rs` (new, 4 probes `c1`–`c4`)
* `tools/play-server/src/main.rs` — `tests::test_dx20b_imprisoned_offer_excludes_the_artifact_over_http`
  (appended to the existing `#[cfg(test)]` module; no new file)

Every row below was **executed**: revert applied, rebuild confirmed by cargo recompiling the
affected crate, test run, verbatim panic captured, revert restored with `git checkout --`, and the
whole set re-run GREEN afterwards. `git diff` over `crates/engine/src`, `crates/card-defs/src` and
`crates/card-types/src` is **empty** at the end.

The panic **LINE** is recorded for every row, not just "it failed" — PB-DX48 had a channel suite
report 3/3 RED where every probe was in fact failing on a corroboration assertion rather than on its
verdict.

---

## The three reverts

| id | what | file | why this one |
|---|---|---|---|
| **R-A** | `imprisoned_in_the_moon` back to `EnchantTarget::Permanent` (the pre-PB-DX20b declaration, `OOS-DX20-10` verbatim) | `crates/card-defs/src/defs/imprisoned_in_the_moon.rs` | the **over-wide** direction — the HIGH |
| **R-B** | drop `has_card_types: f.has_card_types.clone()` from `casting::enchant_filter_to_target_filter` | `crates/engine/src/rules/casting.rs` | the **engine lowering** — proves the new field is load-bearing at the ONE arithmetic, not merely present in the card def |
| **R-C** | `imprisoned_in_the_moon` to `EnchantTarget::Creature` (the obvious over-correction; what `kayas_ghostform` actually shipped — `OOS-DX20-5`) | `crates/card-defs/src/defs/imprisoned_in_the_moon.rs` | the **under-wide** direction. Added because R-A and R-B are BOTH over-wide reverts and neither can redden `c2`/`c3`, which exist for the other direction. Without R-C those two rows would be honestly UNDISCRIMINATED. |

---

## Results

| probe | R-A | R-B | R-C |
|---|---|---|---|
| `c1_the_offer_set_is_exactly_the_printed_enchant_line` | **RED** | **RED** | **RED** |
| `c2_every_offered_target_is_accepted_by_the_engine` | GREEN (by construction — see below) | GREEN (same) | **RED** |
| `c3_a_land_target_resolves_and_the_aura_is_attached_to_it` | GREEN (by construction) | GREEN (same) | **RED** |
| `c4_the_bot_path_sees_the_same_offer_set_and_its_aura_stays_attached` | **RED** | **RED** | **RED** |
| HTTP `test_dx20b_imprisoned_offer_excludes_the_artifact_over_http` | **RED** | **RED** | **RED** |

**No row is UNDISCRIMINATED.** The two GREEN cells are *stated controls*, not gaps, and the reason
is structural rather than incidental: `c2` submits only targets the offer layer itself named, and
`c3` submits a **Land**. Under an over-wide declaration a Land is still legal and every
printed-legal candidate is still offered — so an over-wide revert *cannot* redden either, whatever
assertions they carry. That is precisely why R-C exists, and R-C reddens both. This asymmetry is
disclosed in the test file's own module doc as well as here.

---

## Verbatim failure lines

### R-A (`EnchantTarget::Permanent`)

`c1` — `crates/simulator/tests/pb_dx20b_enchant_offer_channel.rs:311` (the SET assertion, the verdict):

```
assertion `left == right` failed: CR 702.5a — 'Enchant creature, land, or planeswalker'. The offer layer must name exactly those three permanents on this board and nothing else. Missing means the declaration is too NARROW (the OOS-DX20-5 shape); extra means it is too WIDE (OOS-DX20-10, the HIGH this batch closes).
  left: {"Anointed Procession", "Chandra, Flamecaller", "Dragonspeaker Shaman", "Island", "Sol Ring"}
 right: {"Chandra, Flamecaller", "Dragonspeaker Shaman", "Island"}
```

`c4` — `pb_dx20b_enchant_offer_channel.rs:545` (assertion 1, the bot's answer space):

```
assertion `left == right` failed: CR 702.5a: the bot path derives its announcement from the same `legal_targets_per_slot` the browser does, so it must see the same three names
  left: {"Anointed Procession", "Chandra, Flamecaller", "Dragonspeaker Shaman", "Island", "Sol Ring"}
 right: {"Chandra, Flamecaller", "Dragonspeaker Shaman", "Island"}
```

HTTP — `tools/play-server/src/main.rs:11173` (assertion 2, THE HIGH — the wire-shaped exclusion,
not a corroboration):

```
CR 702.5a -- 'Enchant creature, land, or planeswalker'. Sol Ring is an ARTIFACT and the browser was offered it as a target. That is OOS-DX20-10 verbatim: candidates were [Object {"id": Number(201), "kind": String("object"), "label": String("Island"), "owner": String("Human-1"), "value": Object {"Object": Number(201)}}, Object {"id": Number(204), "kind": String("object"), "label": String("Sol Ring"), "owner": String("Human-1"), "value": Object {"Object": Number(204)}}, Object {"id": Number(206), "kind": String("object"), "label": String("Island"), "owner": String("Bot-2"), "value": Object {"Object": Number(206)}}]
```

### R-B (drop `has_card_types` from the lowering)

Identical panic text and identical lines to R-A on all three rows (`c1` at `:311`, `c4` at `:545`,
HTTP at `main.rs:11173`, with `Sol Ring` present in the wire candidates array). That is the
expected shape: an empty `TargetFilter.has_card_types` is no restriction at all, so the filter
degenerates to exactly what `EnchantTarget::Permanent` meant.

### R-C (`EnchantTarget::Creature`)

`c1` — `:311`:

```
  left: {"Dragonspeaker Shaman"}
 right: {"Chandra, Flamecaller", "Dragonspeaker Shaman", "Island"}
```

`c2` — `:361` (the candidate the probe was told to submit is no longer offered at all):

```
'Island' must be an offered candidate (see c1)
```

`c3` — `:419` (the engine REFUSES a printed-legal Land, which is the under-wide defect end to end):

```
CR 702.5a: a Land is a printed-legal target for 'Enchant creature, land, or planeswalker': Rejected(InvalidTarget("declared 1 target(s) but 1 could not be matched to a requirement slot"))
```

`c4` — `:545`: `left: {"Dragonspeaker Shaman"}` vs the three.

HTTP — `main.rs:11186` (assertion 3, the other direction — and note the candidate array is
**empty**, i.e. the browser is offered a cast it cannot legally complete):

```
CR 702.5a: a Land IS a printed-legal target, so land 206 must be offered. Missing it means the declaration was narrowed too far -- the OOS-DX20-5 shape. Candidates: [], board: [(206, "Island", ["Land"]), (201, "Island", ["Land"]), (204, "Sol Ring", ["Artifact"])]
```

---

## Restore and re-run

```
git checkout -- crates/card-defs/src/defs/imprisoned_in_the_moon.rs
git checkout -- crates/engine/src/rules/casting.rs
git diff --stat crates/engine/src crates/card-defs/src crates/card-types/src   # EMPTY
cargo test -p mtg-simulator --test pb_dx20b_enchant_offer_channel              # 4 passed / 0 failed
cargo test -p play-server test_dx20b                                           # 1 passed / 0 failed
```

---

## `c4`'s bot-path A/B, MEASURED (not asserted-unchanged)

Same fixture, same seed (`20_20_20`), two `HeuristicBot` seats, no human, driven to `Halted`.
Measured at HEAD and again under **R-A**, via a throwaway scratch test (`zz_scratch_dx20b_ab.rs`,
written, run, deleted, never committed) that mirrors `c4`'s fixture exactly:

| metric | HEAD (`Filtered[Creature, Land, Planeswalker]`) | R-A (`Permanent`) |
|---|---|---|
| commands in the journal | **63** | **63** |
| `rejection_count()` | **2** | **2** |
| the two refusals | `DeclareAttackers` × 2, `InvalidCommand("Object ObjectId(2) is not a creature")` | **byte-identical** |
| Aura on the battlefield at the halt | yes | yes |
| host the bot attached it to | `ObjectId(2)` — Dragonspeaker Shaman | **same** |
| bot's own candidate SET | **3** names | **5** names |

**Verdict: the bot's PLAY is unchanged; its ANSWER SPACE is not.** The reason is stated rather than
left to coincidence: `targeting::plan_targets` takes the FIRST candidate `legal_targets_per_slot`
returns, that function walks `state.objects()` in ascending `ObjectId` order, and the lowest-id
battlefield permanent here is the Dragonspeaker Shaman — legal under **both** declarations. So the
narrowing removes two candidates the bot was never going to pick. A fixture in which the artifact
had the lowest id would have moved the bot's pick; this one does not, and saying "unchanged"
without that sentence would have been a coincidence reported as a property.
