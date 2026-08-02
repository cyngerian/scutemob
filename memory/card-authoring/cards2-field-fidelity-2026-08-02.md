# CARDS-2 — corpus printed-field fidelity audit (2026-08-02, `scutemob-181`)

Evidence record for the batch that built **SR-37** and repaired what it found. The gate itself
is `crates/engine/tests/core/cards2_printed_field_fidelity.rs`; its full rationale is in
`docs/engine-invariants.md`. This file holds the *measurement* and the *dispositions* — the
things a future batch needs and a test file is the wrong home for.

Predecessor: `memory/playtest-triage-2026-08-02.md` findings **F1** (Boon Satyr) and **F2**
(the corpus-wide cost audit), both of which this batch reproduced independently before acting
on them.

---

## 1. The measurement

Method, reproducible from the repository with no scratch files:

```
cargo run -q -p card-field-dump > /tmp/corpus.tsv          # enumerates all_cards() (SR-36)
python3 tools/refresh-card-fidelity-fixture.py \           # joins cards.sqlite, copies verbatim
    --corpus /tmp/corpus.tsv --db cards.sqlite \
    --out test-data/card-fidelity/printed-fields.tsv
cargo test --test core cards2_printed_field_fidelity       # decides equality; the failures ARE the audit
```

Corpus at `1f9fff17`: **1,804 definitions / 1,803 distinct names**. 1,801 joined to Scryfall;
the 2 that did not are the documented synthetics (`poisonous_viper`, `steel_guardian`), which
is exactly what the F2 triage predicted.

**Measured yield — 39 real corpus defects.** The gate's *raw* output was larger at each stage,
and the difference is not bookkeeping: it is the gate learning what a defect is.

| rule | field | first run | after canonicalisers | after structural exceptions (**real**) |
|---|---|---:|---:|---:|
| R2 | mana cost | 21 | 17 | **17** |
| R3 | power / toughness | 8 | 8 | **5** |
| R4 | type line | 21 | 19 | **16** |
| R5 | duplicate card name | 1 | 1 | **1** |
| | **total** | 51 | 45 | **39** |

Column 1 → 2 removed six false mismatches created by the gate's own notation (hybrid pip order,
multi-word subtypes — §1 findings 1 and 2). Column 2 → 3 removed six more that were the design
working (the meld-result shell, the CDA placeholder — findings 3 and 4).

**39 is the number to quote.** The intermediate 45 appears in this batch's second commit message
(`b7a46cb3`) and briefly in this file, both written before the structural exceptions were
recognised; that history is left as written rather than rewritten, but it is not the yield.

R2 reproduced the F2 table **exactly, card for card** — the first independent confirmation
that audit was reproducible rather than a one-off observation.

### Four findings that were the gate's own fault, not the corpus's

Recorded because a future field audit will hit the same three shapes:

1. **Hybrid pip order.** `{R/G}` and `{G/R}` are one pip printed two ways (CR 107.4e). The
   first draft normalised the printed side and not the def side, and reported four false
   mismatches. Fixed by `ordered_pair` on both sides.
2. **Multi-word subtypes.** `Land — Urza's Cave` is two land types that read as one phrase;
   `Time Lord` is one creature type that reads as two words. With no subtype catalogue in the
   repository there is no way to tokenise the printed line correctly, so R4 compares the
   multiset of *words*. Deliberate, documented, and filed as **OOS-CARDS2-2**.
3. **Meld results.** Hanweir, the Writhing Township is a never-cast shell whose characteristics
   live on `back_face` (CR 712.5b), so comparing its deliberately-empty front reported three
   mismatches that were the design working. The exception is recognised **structurally** — by
   the def being referenced as some other def's `meld_pair.melded_card_id` — never by name,
   because a name allowlist is a door any def can walk through.
4. **CDA power/toughness.** Nighthawk Scavenger prints `1+*`. No fixed number is correct, so
   R3 requires `None` from a `Complete` def and permits a placeholder from a non-`Complete`
   one. This is the ONLY field where the marker buys an exemption, and only because no correct
   value exists; a wrong mana cost gets no such pass, the printed cost being a fact the def
   could simply have copied.

---

## 2. Dispositions

### Boon Satyr — the headline, four defects in one `Complete` def

| clause | def said | printed |
|---|---|---|
| mana cost | `{2}{G}` | `{1}{G}{G}` |
| bestow cost | `{4}{G}{G}` | `{3}{G}{G}` |
| type line | Creature — Satyr | Enchantment Creature — Satyr |
| "Enchanted creature gets +4/+2" | **not authored at all** | printed |

All four repaired; the def stays `Complete` because all four were expressible. The +4/+2 is two
layer-7c statics filtered to `EffectFilter::AttachedCreature` — the shape **Rancor already
used**. The machinery was never missing; nobody reached for it.

Two things worth carrying forward:

* **The cost defect was a transposition**, so mana *value* was unchanged at 3. No arithmetic
  check could ever have caught it — which is why R2 compares structure, not mana value.
* **The engine's own bestow test had the right numbers all along**
  (`mechanics_a_d/bestow.rs`, against a mock named "Mock Bestow Satyr"). A mock-based keyword
  test and the real card def had drifted apart with nothing comparing them.

`primitives::cards2_printed_field_repair` T5 is the discriminating test, and it was **proven so
by execution**: reverting the two statics leaves the enchanted 2/2 at 2/2. That is the exact
pre-repair behaviour — bestow attached correctly and granted nothing.

### Two `Complete` defs implementing a different card's abilities

Both surfaced because **more than one** printed field was wrong, which turns out to be a
reliable signal that a def was authored from a misremembered card rather than mistyped:

| def | implemented | printed |
|---|---|---|
| `backup_agent` | Backup 1 + Lifelink, 2/3 Human Soldier, `{2}{W}` | ETB +1/+1 counter on target creature, 1/1 Human Citizen, `{1}{W}` |
| `necron_deathmark` | mandatory destroy restricted to an opponent; each player mills 2 | up to one target creature; **target player** mills 3 |

Both repaired to the printed ability; both stay `Complete` (`TargetRequirement::UpToN` and
`PlayerTarget::DeclaredTarget` both already existed, with reference implementations in
`bridgeworks_battle.rs` and `altar_of_dementia.rs`). Consequence worth knowing: **no def in the
corpus declares `KeywordAbility::Backup` any more.** The keyword's engine behaviour is covered
by `mechanics_a_d/backup.rs` against a mock, which is where a keyword test belongs — one that
rides a single real card silently becomes a test of that card's authoring.

### R5 — a card defined twice

`Legolas's Quick Reflexes` had two def files under two `CardId`s.
`CardRegistry::try_new` rejects a duplicate **CardId** and says nothing about a duplicate
**name**, so both built cleanly and both shipped.

The finding was not new. `memory/card-authoring/marker-sweep-2026-07-16.md:582-583` recorded it
seventeen days earlier and wrote "one of the two should be deleted before either is authored".
Nothing happened, because no gate could fail. **That is the generalisable point of this whole
batch: a finding written into a memory doc is a finding that will still be there next month.**

`legolasquick_reflexes.rs` deleted. The survivor keeps the `CardId` that
`test-data/test-cards/edhrec_all_commanders.json` records, and carries the deliberate W5
"no castable do-nothing" decision (`consolidated-fix-list.md` M2) which had only ever reached
one of the two files.

### Two more `Complete` defs were implementing *invented* text — both demoted

Distinct from the two above, and worse: those implemented a **different real card's** abilities,
these implement text that exists on no card at all. Both were found the same way — a mana-cost
repair prompted an oracle read.

| def | implemented | printed |
|---|---|---|
| `cyber_conversion` | "becomes an artifact until end of turn. Draw a card." | "Turn target creature face down. It's a 2/2 Cyberman artifact creature." |
| `exalted_angel` | `KeywordAbility::Lifelink` | "Whenever this creature deals damage, you gain that much life." |

Exalted Angel's is not a quibble: CR 702.15a lifelink is a **static** ability — the life gain is
part of the damage event, cannot be responded to, and cannot be countered. The printed clause is
a **triggered** ability that uses the stack and can be answered with Stifle.

Both **demoted** (`inert` / `partial`) with blocker notes naming the exact missing primitive,
rather than half-repaired, because neither is expressible:

* **no `Effect` turns an already-on-battlefield permanent face down in place** — every
  face-down path in the engine (`FaceDownKind::{Morph,Megamorph,Disguise,Manifest,Cloak}`) is an
  *entering* mechanism, and none carries the "plus artifact" Cyberman characteristics
  (**OOS-CARDS2-5**);
* **no general "whenever this permanent deals damage" `TriggerCondition`** —
  `WhenDealsCombatDamageToPlayer` is too narrow (misses combat damage to blockers and all
  noncombat damage) and `WhenDealtDamage` is the wrong direction (CR 702.111a Enrage). Also needs
  a damage-dealt `EffectAmount` for "that much" (**OOS-CARDS2-6**).

The incorrect abilities were **removed** rather than left in place, per W5 policy: an uncastable
card is better than a castable wrong one.

**These two demotions re-dealt every seeded game a second time**, through the channel the old pin
comments *did* anticipate (a completeness flip), after the type-line and mana-cost repairs had
already re-dealt them through the channel those comments missed. Both channels are real.

### Land subtypes — checked before removing, not after

`lonely_sandbar` declared `Island` and `windbrisk_heights` declared `Plains`; neither card
prints a basic land type. Before removing them the engine was checked for a CR 305.6
intrinsic-mana-ability derivation, since removing an Island subtype from a land that *relies*
on it would silently remove its ability to tap for mana. There is no such derivation in this
engine — `Characteristics.mana_abilities` is populated only from `CardDefinition.abilities` —
and both defs declare their own explicit `{T}: Add {U}` / `{T}: Add {W}`. Safe, and verified
rather than assumed.

---

## 3. The seed-pin lesson (the durable one)

Five play-server tests and three golden scripts failed. **This batch flipped ZERO completeness
markers**, and every seeded deck still re-dealt.

Root cause, verified before any pin was touched — `crates/simulator/src/deck.rs::random_deck`:

* the commander is drawn from cards that are `Complete` **AND `SuperType::Legendary` AND
  `CardType::Creature`**;
* the deck is then filled by **colour identity**, which is computed from the **mana cost**.

So a *type-line* repair and a *mana-cost* repair both move the deal without touching a marker.
Measured: the commander pool went **91 → 90** — `+Akroma, Angel of Fury` (which is Legendary),
`−Overlord of the Hauntwoods`, `−Prosperous Innkeeper` (which are not) — and
`rng.random_range(0..commanders.len())` re-picked every seat. Colour identities moved too
(`Braided Net` `{2}` → `{2}{U}` is no longer colourless).

Every one of those pins carried the comment *"re-read when a card-def batch flips a marker"*.
**That guard is too narrow. These pins are a function of the whole corpus, not of the
completeness markers.** All the comments now say so.

Three of the re-pins were not mechanical, and each is its own lesson:

1. **`test_x_value_is_forwarded_to_cast_spell_data` had silently retargeted.** Its predicate,
   `option_with_targets(v, 1)`, matches *any* action carrying candidates — and the re-dealt
   seed 9 offered no targeted cast, so the driver stopped on Deserted Temple's "untap target
   land" **activated ability**. The failure then surfaced three assertions later as "the cast
   is still offered after tapping" (it was never a cast). The predicate now says `CastSpell`.
   *A fixture predicate that is broader than the fixture's purpose does not fail when the
   fixture moves — it silently tests something else.*
2. **`TARGET_SEED` 9 → 20 had to be chosen by re-sweeping**, `seed` ∈ 0..24 × `develop` ∈
   {false, true}; the old 0..12 sweep no longer contained a usable pair, and **seed 0 reaches
   no targeted cast at all within 300 decisions**, so no step budget would have rescued the old
   pin. Of ten candidate seeds, **four (2, 10, 11, 17) are OFFERED a targeted cast and then
   refused by the engine** with "player does not have enough mana to pay the cost" once sources
   are tapped — playtest finding **F4** (the provider's colour-blind affordability shortcut
   offering what the engine rejects, an SR-38 violation) reproducing on four independent seeds.
   A fifth (seed 5, Flame Jab, "any target") would have made the `422` assertion pass for the
   wrong reason, since a player *is* a legal target for it. Terminate at seed 20 was picked
   because "destroy target creature" is the property the caller actually depends on.
3. **`test_target_option_labels_are_seat_redacted`'s leak oracle was substring-matching names**
   with no notion that a name can legitimately appear in a public zone too. The new deal put a
   Forest in a bot's hand while Forests stood on the battlefield, and the battlefield label
   "Forest" was reported as a hand leak. False positive by construction; it now subtracts the
   names the seat may legitimately see, which is the excusal its sibling test already made.

### Golden scripts

* **177** (Tyrranax Rex) and **164** (Changeling Hero) had their mana pools re-derived. Both had
  been written to **what the def said, not to the card**, so both passed for two batches while
  encoding a wrong cost. A golden script is not an independent check of a card def if it was
  generated from that def.
* **163** (Backup Agent) is **RETIRED**. Its subject — Backup Agent's Backup 1 — does not exist.
  Rewriting it in place would leave a file named and identified for Backup while testing an ETB
  counter. Nothing is lost; see the `backup_agent` note above.

---

## 4. Counts

| | before | after |
|---|---:|---:|
| definitions | 1,804 | **1,803** |
| `Complete` | 1,137 | **1,133** |
| non-`Complete` | 667 | **670** |
| coverage | 63.0% | **62.8%** |

**Four completeness flips, ALL demotions, all honest.** Coverage went **down**, and that is the
correct direction: it is the PB-DX4 pattern — *the number fell because the corpus got truer*.
Two defs implemented text that exists on no card (§2.4), one needs six absent primitives to
express its real abilities (§2.6), one has no `Cost` variant for its printed mana ability. The
denominator also fell by one, because a double-counted card stopped being counted twice.

`completeness_deviation_scan`'s floor moved in **three steps within this batch**, 667 → 666 (the
deleted duplicate) → 668 (the first two demotions) → **670** (the review fix cycles), each
re-measured directly against `all_cards()` **and** by an independent grep of `MARKER_FRAGMENTS`,
as that file's own comment instructs, rather than derived by arithmetic from the previous value.

Tests **4,165 / 0 / 5** workspace-wide. Zero engine lines (empty diff over `crates/engine/src`
and `crates/card-types/src`). PROTOCOL and HASH gate-executed unmoved; `decision_gate` 18/18.

---

## 5. Cross-references and seeds

### Cross-reference: the dropped-`{X}` class and OOS-M11-8

`chord_of_calling`, `green_suns_zenith`, `torment_of_hailfire` and `wake_the_dead` all carried
`x_count: 0` for a printed `{X}`, so each was castable at a fixed cost with X structurally
unavailable — Torment of Hailfire for `{B}{B}`, draining for zero. This is the same population
**OOS-M11-8** reasons about (`x_count` handling), approached from the other end: that seed is
about the engine's treatment of `x_count`, this is about defs that never set it. The four are
repaired and pinned by `primitives::cards2_printed_field_repair::t7`, and R2 now makes the class
unable to recur — a def printed with `{X}` and declaring `x_count: 0` fails the gate. **Nothing
in OOS-M11-8's own scope is closed by this**; the seed stands.

Also worth noting for whoever picks that seed up: `wake_the_dead.rs` carried an inline comment
reading "X cost not expressible in ManaCost struct", which was **never true** — `x_count` has
always been a field. That is the third stale "not expressible" note this batch found (see §6).

* **OOS-CARDS2-1** — the fixture is only as current as the local `cards.sqlite` snapshot.
  A card printed after that snapshot joins nothing, lands in the `# unmatched:` trailer, and R1
  then fails for a def that is perfectly correct. Loud rather than silent, which is the right
  failure, but a refresh path for the database itself does not exist in-repo
  (`tools/scryfall-import` builds it; nothing schedules it). Unfixed.
* **OOS-CARDS2-2** — R4 compares subtypes as a multiset of **words**, so it cannot tell
  `SubType("Time Lord")` from `SubType("Time") + SubType("Lord")`. The distinction is real (a
  type-changing effect matching `Lord` behaves differently) but it is a question about the DSL's
  representation rather than fidelity to the printed card, and settling it needs a subtype
  catalogue the repository does not carry. Two defs in the corpus write a two-word subtype as
  one entry (`The Soul Stone`, `Urza's Cave`); both are consistent with their printed lines
  under the word comparison. Unfixed, deliberately.
* **OOS-CARDS2-3** — `random_deck`'s sensitivity to the corpus is undocumented anywhere the
  person editing a card def would look. Section 3 above records it and every affected pin's
  comment now states it, but there is no gate: a future batch will discover it the same way,
  by watching eight tests go red. A cheap fix would be a test that pins the commander-pool
  *size* with a comment pointing at the card-def batch checklist. Not built.
* **OOS-CARDS2-4** — **an Aura spell is offered with `target_min: 0` and then refused by the
  engine.** An Aura carries its target requirement in `KeywordAbility::Enchant(...)`, which
  `casting.rs:3720` special-cases (CR 303.4a, "Aura spells require exactly one target"). The
  *provider* does not read that keyword, so the `ActionOptionView` says "announces nothing" and
  the cast 422s. **Browser-client-visible: a human clicking any Aura gets an error.** Same family
  as playtest findings F4 and F9 (SR-38: never offer what the engine rejects), and the same shape
  as CARDS-1's equip bug one link earlier in the chain — an engine special-case the offer layer
  is blind to. Found because the re-dealt COMBAT_SEED drove the S7 test driver straight into
  "Cast Hyena Umbra"; the driver now skips a refused action rather than aborting, which is a
  workaround in the *test*, not a fix. Simulator-only to fix; no engine or wire change. Unfixed.
* **OOS-CARDS2-5** — no `Effect` turns an already-on-battlefield permanent face down in place
  (see §2.4). Blocks `cyber_conversion`, and any card of the Kasmina's Transmutation / Ixidron
  family. Unfixed; the def is honestly `inert`.
* **OOS-CARDS2-6** — no general `TriggerCondition::WhenDealsDamage` and no damage-dealt
  `EffectAmount` for "gain that much life" (see §2.4). Blocks `exalted_angel`. Unfixed; the def is
  honestly `partial`.
* **OOS-CARDS2-7** — **`completeness_deviation_scan`'s needle set misses the two phrases the
  corpus actually uses for an unimplemented ability.** Its needles are `["simplif",
  "modeled as", "modelled as", "deviation", "approximat"]`. `braided_net.rs` said **"DSL gap"**
  three times and `windbrisk_heights.rs` said the condition was **"deferred"**, and both shipped
  `Complete` with a printed ability missing; neither reddened. A grep of the corpus for "DSL gap"
  / "deferred" / "not expressible" / "TODO" against `Complete` defs would size the class. Not
  done here — this batch repaired the two it tripped over, which is not a sweep.
* **OOS-CARDS2-8** — **stale "not expressible" notes are a recurring class, and nothing rechecks
  them.** Four found in this batch alone, all false by the time they were read:
  `wake_the_dead` ("X cost not expressible in ManaCost struct" — `x_count` always existed),
  `boon_satyr` (the aura static, which `rancor.rs` already used), `braided_net`
  ("TapTarget effect not in DSL" — `Effect::TapPermanent` exists) and `windbrisk_heights`
  ("attack tracking is deferred" — `Condition::YouAttackedWithNOrMore` exists). A note claiming
  a primitive is missing is written once and never revisited when the primitive lands. Cheap
  partial fix: have the DSL-gap notes name the primitive they want, so a grep can check whether
  it now exists.
* **F4 corroboration** (not a new seed) — the sweep in section 3 is independent evidence for
  `memory/playtest-triage-2026-08-02.md` F4, on four seeds the triage never examined. Whoever
  fixes the mana solver has four ready reproductions.
