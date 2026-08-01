# Primitive Batch Plan: PB-DX6 — The Last Two Unflattened Mana-Cost Payment Sites

<!-- last_updated: 2026-08-01 -->

**Generated**: 2026-08-01
**Task**: `scutemob-172` · **Branch**: `feat/pb-dx6-the-last-two-unflattened-mana-cost-payment-sites-oos-`
**Primitive**: hybrid (`{G/W}`, `{2/B}`) and Phyrexian (`{G/P}`, `{G/W/P}`) pips become *payable*
— and therefore *chargeable* — at the two payment sites PB-RS2 left standing:
`rules/engine.rs::handle_turn_face_up` and `Command::DeclareAttackers`' CR 508.1h attack tax.
Plus: `ManaPool::can_spend` / `ManaPool::spend` stop being silent on an unflattened residue.
**Seeds closed**: OOS-RS2-1 (turn-face-up), OOS-DP4-1 (attack tax)
**CR Rules**: 107.4e, 107.4f, 116.2b, 118.5, 119.4, 119.4b, 202.3f, 202.3g, 508.1c/d/f/g/h/i/j,
601.2f, 602.2b, 605.1a, 701.40b, 701.40c/d, 702.37e, 702.168d, 104.3b
**Class**: CORRECTNESS. **LIVE** on the turn-face-up half (3 `Complete`, deck-legal creature defs).
**LATENT** on the attack-tax half (0 defs carry a pipped or X attack tax).
**Cards affected**: 5 defs enter a permanent roster gate; **0 completeness flips expected** (§10)
**Dependencies**: PB-9 (`ManaCost.hybrid`/`.phyrexian`), **PB-RS2** (`ManaCost::flatten_hybrid_phyrexian`
in `card-types`, `HybridManaPayment`, the `hybrid_choices`/`phyrexian_life_payments` schema shape,
`legal_actions::resolve_hybrid_phyrexian_plan`, `debug_assert_flattened`), PB-DP4 (the attack-tax
total/debit machinery and `unpayable_tax_defenders`), M11-local S3 (`rules/queries.rs`, the
read-only public query surface this plan extends)
**Deferred items from prior PBs**: OOS-DP4-7 (`add_mana_cost` vs `multiply_mana_cost` dedup) is
**re-dispositioned, not closed** — §5.4 gives a new, stronger reason to keep them separate.
OOS-DP4-2 (CR 508.1i mana-ability window) is untouched and stays open.
**Wire expectation**: **PROTOCOL 32 → 33** (predicted, must be *computed*).
**HASH stays 70** (predicted, must be *computed*). Falsifiers for both: §7.

---

## 0. Premise — accepted, plus two sanity checks that survived and one number the brief
##    understated

`memory/primitive-wip.md`'s premise was re-verified by the coordinator against this branch and is
**not re-derived here**, per the task brief. Two claims were nonetheless spot-checked because they
are the ones most likely to be wrong, and **both held**:

1. **"Site 1a is 5 creature defs."** The 26 def files containing `hybrid: vec![` / `phyrexian:
   vec![` were partitioned by *where* the pip sits. `necropanther`, `brokkos_apex_of_forever` and
   `nethroi_apex_of_death` are all **creatures** and all appear in that 26 — the obvious way for a
   5-card roster to actually be 8 — but in every one of the three the pip is on
   `AbilityDefinition::MutateCost`, not on the def's printed `mana_cost`. `handle_turn_face_up`'s
   `TurnFaceUpMethod::ManaCost` branch reads `def.mana_cost` and nothing else, so all three are
   correctly **out** of scope. The roster of 5 stands.
2. **"3 of the 5 are `Complete`."** `kitchen_finks` declares `completeness: Completeness::Complete`
   explicitly. `boggart_ram_gang` declares it explicitly. **`blade_historian` declares no
   `completeness` field at all** and is `Complete` only by the `#[default]` derive — the
   twice-demonstrated silent-defect generator PB-DX3b and PB-DX4 both hit. It is deck-legal, so the
   roster of 3 live-wrong `Complete` defs stands, but the runner should know that one of the three
   is `Complete` by omission rather than by decision. **Do not "fix" this by demoting it** — the
   def is otherwise fine and this batch makes it correct; note it in the roster gate's comment.

**One number the brief understates, stated loudly**: the dispatch brief at
`seed-rerank-2026-07-27.md`'s PB-DX6 entry names exactly one card (`kitchen_finks`) and says
"manifest or cloak it and flip it for `{1}`". The live-wrong roster is **three** `Complete` defs
(Kitchen Finks, Blade Historian, Boggart Ram-Gang) plus two deck-illegal ones. This is the
**seventh consecutive batch in this suite whose published roster was wrong before it started**, and
it is wrong in the same direction as the other six (undercount). It does not change the design.

**A third correction the premise did not make, and it changes how every fail-before probe in this
batch must be written** — see §2.0. In a **debug** build (which is every `cargo test` run and all
of CI), `handle_turn_face_up` today does **not** silently charge `{1}`: it **panics**, inside
`debug_assert_flattened`. The "flips for `{1}`" behaviour is a **release-only** claim. Any probe
that asserts it without saying which build it was observed in is exactly the manufactured-number
failure PB-DX3's MEDIUM and all six of PB-DX5's MEDIUMs were about.

---

## 1. CR rule text (verbatim, MCP-sourced)

**CR 107.4e** — "A hybrid mana symbol is also a colored mana symbol, even if one of its components
is colorless. Each one represents a cost that can be paid in one of two ways, as represented by the
two halves of the symbol. A hybrid symbol such as {W/U} can be paid with either white or blue mana,
and a monocolored hybrid symbol such as {2/B} can be paid with either one black mana or two mana of
any type. A hybrid mana symbol is all of its component colors."

**CR 107.4f** — "Phyrexian mana symbols are colored mana symbols: {W/P} is white, {U/P} is blue,
{B/P} is black, {R/P} is red, and {G/P} is green. A Phyrexian mana symbol represents a cost that
can be paid either with one mana of its color or by paying 2 life. There are also ten hybrid
Phyrexian mana symbols. A hybrid Phyrexian mana symbol represents a cost that can be paid with one
mana of either of its component colors or by paying 2 life. A hybrid Phyrexian mana symbol is both
of its component colors."

**CR 119.4** — "If a cost or effect allows a player to pay an amount of life greater than 0, the
player may do so only if their life total is greater than or equal to the amount of the payment. If
a player pays life, the payment is subtracted from their life total; in other words, the player
loses that much life."

**CR 119.4b** — "Players can always pay 0 life, no matter what their (or their team's) life total
is, and even if an effect says players can't pay life."

**CR 701.40b** (manifest — **the whole of site 1a**) — "Any time you have priority, you may turn a
manifested permanent you control face up. This is a special action that doesn't use the stack (see
rule 116.2b). To do this, show all players that the card representing that permanent is a creature
card and what that card's mana cost is, **pay that cost**, then turn the permanent face up. The
effect defining its characteristics while it was face down ends, and it regains its normal
characteristics. (If the card representing that permanent isn't a creature card or it doesn't have
a mana cost, it can't be turned face up this way.)"

> "Pay that cost" is the card's printed mana cost, verbatim, pips and all. Nothing in CR 701.40b
> waives a hybrid or Phyrexian symbol, and CR 107.4e/107.4f define what paying one means. So an
> engine that charges `{1}` for `{1}{G/W}{G/W}` is not making a simplification; it is skipping two
> thirds of the printed cost.

**CR 701.40c / 701.40d** — a manifested card with morph (resp. disguise) may be turned face up by
**either** the CR 702.37e morph procedure **or** the CR 701.40b procedure. This is why all three
`TurnFaceUpMethod` branches share one payment block and why fixing one fixes all three.

**CR 508.1** (the whole attack-tax half). The load-bearing children:

- **508.1c** — "The active player checks each creature they control to see whether it's affected by
  any restrictions (effects that say a creature can't attack, or that it can't attack unless some
  condition is met). If any restrictions are being disobeyed, the declaration of attackers is
  illegal."
- **508.1d** — "… If a creature can't attack unless a player pays a cost, that player is not
  required to pay that cost, even if attacking with that creature would increase the number of
  requirements being obeyed. …"
- **508.1g** — "If there are any optional costs to attack with the chosen creatures (expressed as
  costs a player may pay 'as' a creature attacks), the active player chooses which, if any, they
  will pay."
- **508.1h** — "If any of the chosen creatures require paying costs to attack, or if any optional
  costs to attack were chosen, the active player **determines the total cost to attack**. Costs may
  include paying mana, tapping permanents, sacrificing permanents, discarding cards, and so on.
  Once the total cost is determined, it becomes 'locked in.' If effects would change the total cost
  after this time, ignore this change."
- **508.1i** — "If any of the costs require mana, the active player then has a chance to activate
  mana abilities (see rule 605, 'Mana Abilities')." *(Not honoured; pre-existing OOS-DP4-2.)*
- **508.1j** — "Once the player has enough mana in their mana pool, they pay all costs in any
  order. **Partial payments are not allowed.**"

**Norn's Annex rulings (2011-06-01, MCP)** — the decisive evidence for §5.2's design, and the
reason design (B) there is *rules-wrong* rather than merely inelegant:

> "If a player attacks with more than one creature, that player chooses how to pay each cost
> **individually**. For example, if you attack with two creatures, you may pay {W} for one cost and
> 2 life for the other."

> "Multiple Norn's Annexes controlled by the same player will each impose a cost to attack. Players
> choose how they pay each cost **individually**. For example, if a creature you control is
> attacking a player who controls two Norn's Annexes, you may pay {W} for one cost and 2 life for
> the other."

> "The controller of each creature attacking you or a planeswalker you control pays either {W} or
> 2 life **as attackers are being declared**."

> These three settle the two questions §5.2 has to answer: the choice is **per copy of the symbol**
> (so the pips must be replicated into the total *before* flattening), and it is announced **with
> the declaration** (so a field on `Command::DeclareAttackers` is the right channel, not a
> follow-up command).

---

## 2. Step 0 — the probes that must be observed FIRST, and how each pre-fix number is READ

**Standing discipline (mandatory, and the single most-cited failure in this suite's last four
batches)**: *every* pre-fix claim in the test module, the commit message and the close-out must be
**observed**, never reasoned to. For each probe below, the plan states the exact procedure that
produces the number. A claim that cannot be produced this way must be labelled **vacuous** in the
test module, not given a plausible figure.

The observation protocol, used unchanged everywhere below:

1. Write the probe. Run it on unmodified code. **Record the literal outcome** (error variant,
   message text, pool fields, life total, panic message).
2. Apply the fix. Run again. Record the new outcome.
3. For any claim about a *code path the fix removes*, re-observe by **reverting just that hunk**
   (`git stash`/edit/run/read/restore), never by re-reading the diff.

### 2.0 The build-mode trap — read this before writing probe A

`handle_turn_face_up` calls `player_state.mana_pool.can_spend(&mana_cost, None)` on the raw
`def.mana_cost`. `can_spend`'s first statement is `debug_assert_flattened(cost)`. Therefore:

| build | pre-fix behaviour of a manifested Kitchen Finks flip |
|---|---|
| **debug** (`cargo test`, CI) | **panics** — `"unflattened mana cost reached the payment path: 2 hybrid + 0 Phyrexian pip(s) would be paid for free"` |
| **release** | `Ok(_)`; pool debited by exactly `{1}`; both `{G/W}` pips free |

Consequences the runner must honour:

- The "flips for `{1}`" figure is a **release** figure. Produce it by **one** of these, and say in
  the test module which one was used:
  - **(preferred, cheap)** temporarily comment out the `debug_assert_flattened(cost);` line at the
    top of `can_spend`, run the probe in debug, read `pool.white/green/generic`-equivalents and the
    `Ok`, restore the line. This is the §2 step-3 protocol applied to the *guard* rather than the
    fix.
  - **(confirmatory, expensive)** `cargo test -p mtg-engine --release --test primitives
    pb_dx6 -- --nocapture` once, and paste the observed numbers.
- The **debug** pre-fix outcome (a panic) is itself a real, recordable observation and should be
  recorded too — it is the reason this bug survived: *every test build the project has ever run
  would have caught it, and no test ever put a pipped cost through this site.* That sentence is the
  batch's most useful finding and it must be stated in the test module with the panic text quoted.

The same trap applies to any Phyrexian probe on this path. It does **not** apply to the attack-tax
path pre-fix, because `combat.rs` rejects a pipped tax before reaching `can_pay_cost` — there, the
pre-fix observation is the `InvalidCommand` message, quoted verbatim.

### 2.1 Probe roster (all in `crates/engine/tests/primitives/pb_dx6_unflattened_payment_sites.rs`)

New file; **add the `mod` line to `crates/engine/tests/primitives/mod.rs`** — SR-9a, a dropped
`mod` line silently deletes the whole file's coverage.

**Mandatory probe (a) — the brief's own headline.**
`T1 manifested_kitchen_finks_flip_charges_both_hybrid_pips` — CR 701.40b, 107.4e.
Build a face-down battlefield object with `face_down_as: Some(FaceDownKind::Manifest)`,
`card_id: cid("kitchen-finks")`, controller P1, P1 holding priority. Post-fix:
- empty pool → `Err(InvalidCommand)` (the turn-face-up affordability message);
- pool `{1}{G}{G}`, `hybrid_choices: [Color(Green), Color(Green)]` → `Ok`, pool empty after;
- pool `{1}{G}{W}`, `[Color(Green), Color(White)]` → `Ok`, pool empty (CR 107.4e, each pip chosen
  independently);
- pool `{1}{G}{G}`, `[Color(Blue), …]` → `Err` naming CR 107.4e (the flattener's existing
  component check);
- pool `{1}{G}{G}`, `hybrid_choices: []` → `Ok` (documented default = first colour = Green).
- Pre-fix record: see §2.0. Both figures (debug panic text; release `Ok` + `{1}` debit).

`T2 manifested_blade_historian_and_boggart_ram_gang` — same shape, table-driven over the other two
`Complete` roster members, so the fix is proven on the whole live roster and not on one card.

**Mandatory probe (b) — the attack-tax half.**
`T3 hybrid_attack_tax_is_payable` — CR 508.1h/508.1j, 107.4e. A synthetic
`AbilityDefinition::StaticRestriction { CantAttackYouUnlessPay { cost_per_creature: {G/W} } }` on a
P2 permanent (no card def carries one — §10). One attacker into P2:
- **pre-fix**: `Err(InvalidCommand)` whose message contains `"is not payable"` and `"OOS-DP4-1"` —
  quote it verbatim in the test module;
- **post-fix**: pool `{G}`, `hybrid_choices: [Color(Green)]` → `Ok`, pool empty, attacker declared;
- pool `{W}`, `[Color(White)]` → `Ok`;
- pool `{G}`, `[Color(White)]` → `Err(InvalidCommand)` — insufficient mana, not "unpayable class".

**Phyrexian coverage on both paths (mandatory per the brief).**
`T4 turn_face_up_phyrexian_pip_payable_with_mana_or_life` — CR 107.4f, 119.4. A synthetic
creature def with `mana_cost: {1}{G/P}`, manifested:
- `[false]` + `{1}{G}` → `Ok`, life unchanged;
- `[true]` + `{1}` only, 20 life → `Ok`, life 18;
- `[true]` + `{1}` only, **1 life** → `Err(InsufficientLife)` citing CR 119.4, life unchanged;
- `[true]` + `{1}` only, **2 life** → `Ok`, life 0, and the CR 704.5a SBA loss is asserted
  separately as the *legal-but-losing* boundary (mirrors PB-RS2's test 9);
- `[true]` + **empty** pool on a **pure** `{G/P}` cost → `Ok`, life −2. This pins that the flatten
  runs *before* the `mana_value() > 0` gate and that the life deduction is a **sibling** of that
  gate, not nested inside it — the raw cost's `mana_value()` is 1, the flattened one's is 0.
`T5 phyrexian_attack_tax_payable_with_mana_or_life` — the same four cases on a
`cost_per_creature: {W/P}` restriction, i.e. **Norn's Annex, simulated**. Include the ruling's own
example as a named case: two attackers, `phyrexian_life_payments: [false, true]`, pool `{W}`,
20 life → `Ok`, pool empty, life 18.

**Mandatory probe — PB-DP4's rejection path still rejects.**
`T6 genuinely_unpayable_attack_tax_is_still_rejected` — CR 508.1h/508.1j. A `{2}` Propaganda-shaped
tax, two attackers, pool `{1}`: `Err(InvalidCommand)` with the existing "cannot pay the required …"
message. This must be **unchanged** by the batch; assert on the message, and state in the test that
its purpose is to prove the batch widened *payability*, not *acceptance*.

**Mandatory probe — X still rejects.**
`T7 x_attack_tax_is_still_rejected_and_says_only_x` — CR 107.3. `cost_per_creature` with
`x_count: 1`, one attacker into that defender: `Err(InvalidCommand)`. **Assert on the message
text**, specifically that it (i) names X, (ii) does **not** claim hybrid or Phyrexian costs are
unpayable, and (iii) cites the new seed rather than OOS-DP4-1 (which this batch closes). An
`is_err()`-only assertion here would be vacuous — the pre-fix code also errors, for a different and
now-wrong reason. This is the PB-DX2 T12 lesson applied in advance.

**Residue-guard probes** — §6.4.

**Ordering / multi-source probes** — §5.2, T8–T11.

**Wire sentinel** — §7, T12.

---

## 3. Engine change 1 — `Command` schema

### 3.1 `Command::TurnFaceUp` (`rules/command.rs`, the `TurnFaceUp` variant)

Add the two PB-RS2 fields verbatim in type, name, `#[serde(default)]` and default semantics,
mirroring `Command::ActivateAbility`'s pair:

- `hybrid_choices: Vec<crate::state::game_object::HybridManaPayment>` — CR 107.4e **via CR
  701.40b/702.37e/702.168d**. One entry per hybrid pip of the *resolved turn-face-up cost*, in cost
  order. Empty = default per pip (first colour / the colored half of a monocolored hybrid). An
  over-long vector is rejected by the flattener, not silently ignored.
- `phyrexian_life_payments: Vec<bool>` — CR 107.4f, same shape.

The doc block must say **which cost** the pips are counted against, because it differs per
`method`: `MorphCost` → the `Morph`/`Megamorph` ability's cost; `DisguiseCost` → the `Disguise`
ability's cost; `ManaCost` → `def.mana_cost`. Cite by symbol (`handle_turn_face_up`), not line.

### 3.2 `Command::DeclareAttackers` (`rules/command.rs`)

Same two fields, same `#[serde(default)]`. The doc block is **load-bearing and different**, because
the pips come from a *derived* cost. It must state, in this order:

1. The cost these index is the CR 508.1h **total**, not any printed cost.
2. The exact pip order (§5.2's "canonical order"), verbatim.
3. That the total's pip count depends on the declared attackers, so the vector length is not
   derivable from the board alone.
4. That `rules::queries::attack_tax_total` (§5.3) is the **supported** way for a client to obtain
   the cost the choices index, and that re-deriving it client-side is the drift class OOS-RS-2 was.

### 3.3 Construction-site migration — the dominant mechanical cost

Both are **enum struct variants**, so `..Default::default()` is unavailable; every literal needs two
new lines. Measured at HEAD:

| variant | files | occurrences |
|---|---|---|
| `Command::DeclareAttackers {` | **70** | **320** |
| `Command::TurnFaceUp` (all mentions, incl. `LegalAction`/`TurnFaceUpMethod`) | 24 | — |

Exact sets:
```
rg -n --multiline 'Command::DeclareAttackers \{' crates tools
rg -n --multiline 'Command::TurnFaceUp \{' crates tools
```

**Migration approach — (A), mechanical literal expansion**, exactly as PB-RS2 §3.3 chose and for
the same reasons: large diff, zero semantic risk, compiler-verified per site, `git diff --stat`
shows `+2` per site outside the handful that intentionally pass non-empty vectors.

**Option (B) — boxing `DeclareAttackers` into a `DeclareAttackersData` struct with `Default`** — is
explicitly **rejected again**, and the argument is *stronger* here than in PB-RS2, not weaker: 320
sites is where boxing looks most tempting, and it is exactly the size at which a semantic refactor
riding along with a correctness fix makes the fingerprint delta un-attributable. SR-10's own record
notes that boxing moves the protocol digest independently (closure grew 90 → 91). File as a
follow-up seed; do not take it here.

Golden scripts deserialize unchanged (`#[serde(default)]`, no new required key).

---

## 4. Engine change 2 — nothing to extract

`ManaCost::flatten_hybrid_phyrexian` already lives in `crates/card-types/src/state/game_object.rs`
as an inherent method returning `Result<(ManaCost, u32), String>`, with the CR 107.4e component
check and the over-long-vector rejection PB-RS2's review added. Both new call sites call the
**inherent method** directly (`cost.flatten_hybrid_phyrexian(...)`) and map the `String` to
`GameStateError::InvalidCommand`, matching `rules/abilities.rs`'s call site. Do **not** route
through `rules::casting::flatten_hybrid_phyrexian` — that shim exists for the cast path's own
callers and reaching into `casting` from a non-cast payment path is the layering smell PB-RS2 §4
flagged. (`crates/simulator/src/local_game.rs` still routes through the shim; leaving it is fine,
migrating it is optional and out of scope.)

**Carry-forward limitation to re-document at both new sites, not silently inherit**:
`PhyrexianMana::Hybrid(a, b)` paid with *mana* always uses `a`; `hybrid_choices` does not reach it.
No card on either roster has a hybrid-Phyrexian pip in a turn-face-up or attack-tax cost. Say so in
a comment at each site; do not add a third field.

---

## 5. Engine change 3 — the two payment sites

### 5.1 `handle_turn_face_up` (`rules/engine.rs`)

The `mana_cost` derivation block (all three `TurnFaceUpMethod` arms) is unchanged. Replace the
single payment block that follows it with the `rules/abilities.rs::handle_activate_ability` shape,
adapted — that function is the canonical, reviewed form of this fix and this site must not invent a
variant of it:

1. **Flatten first**, unconditionally when either pip vec is non-empty, mapping the error to
   `InvalidCommand`. Keep the `else { (mana_cost.clone(), 0) }` fast path so pip-free costs allocate
   nothing new.
2. **CR 119.4 check before any mutation.** `handle_turn_face_up` has **no other life component**
   (unlike `abilities.rs`, which combines `ability_cost.life_cost`), so the check is on
   `phyrexian_life` alone — but write it through the same `combined_life_cost` local with a comment
   saying the other addend is structurally zero here, so the two sites read the same and a future
   life component cannot be added past a hard-coded check. `Err(GameStateError::InsufficientLife
   { player, required, actual })`, matching `abilities.rs`.
3. **`if flat_cost.mana_value() > 0 { can_spend … else Err … ; spend }`** — the gate is on the
   **flattened** cost, and the flatten is above it. A pure `{G/P}` paid with life flattens to `{0}`,
   `mana_value() == 0`, and the gate correctly skips the mana check.
4. **Life deduction is a SIBLING of the gate, not nested inside it**, followed by
   `GameEvent::LifeLost`.
5. **`GameEvent::ManaCostPaid { player, cost: <the ORIGINAL, unflattened cost> }`** when either
   component was non-zero — matching `casting.rs`/`abilities.rs`, which emit the pipped shape so
   event consumers see what was printed. **Note**: `handle_turn_face_up` emits **no** `ManaCostPaid`
   today. Adding it is an Architecture-Invariant-4 repair (a pool debit is a state change and must
   be evented) and is in scope; it is a **new event on an existing variant**, so no wire change, but
   it *will* change the event stream of any existing test or golden script that flips a face-down
   permanent with a non-zero cost. Expect and repair that; do not suppress the event to keep a test
   green. (If the repair surface turns out to be large, the fallback is to emit it only when
   `phyrexian_life > 0 || flat_cost.mana_value() > 0`, which is already the condition — measure
   before deciding anything else.)

**Error-variant choice**: the existing insufficiency path returns
`InvalidCommand("TurnFaceUp: player cannot pay the turn-face-up cost")`. **Keep that string** for
the mana-insufficiency case (existing tests assert it) and use `InsufficientLife` only for the CR
119.4 case. Do not "harmonise" it to `InsufficientMana` in this batch.

**Atomicity**: none needed. `process_command` takes `GameState` by value and returns it only on
`Ok`, so an `Err` after a partial mutation discards the whole state (Architecture Invariant 2/3).
`abilities.rs` documents this at the same site. **Do not invent a rollback** — it would be dead
code. Place the CR 119.4 check before the deduction anyway, for legibility and to match the sibling.

### 5.2 `handle_declare_attackers` (`rules/combat.rs`) — the hard part

#### 5.2.1 The design question, and the answer

The tax total is `Σ_defender ( Σ_restriction cost_per_creature ) × attackers_against_that_defender`.
`hybrid_choices` is a **positional** vector. So: *where does the flatten happen, and against what
does position 0 mean anything?*

**Two candidate designs.**

- **(A) Replicate pips into the accumulated total, then flatten the total once.** `add_mana_cost`
  learns to carry hybrid/Phyrexian pips through, replicating them `times` times; the total therefore
  contains one copy of each pip per attacking creature per restriction; `hybrid_choices` indexes the
  total's `hybrid` vec; one `flatten_hybrid_phyrexian` call produces the flat cost and the total
  life.
- **(B) Flatten each restriction's `cost_per_creature` before multiplying**, so `hybrid_choices` has
  one entry per pip of the *printed* per-creature cost, applied identically to every copy.

**(B) is rules-wrong and is rejected on evidence, not taste.** Norn's Annex ruling 2011-06-01:
*"If a player attacks with more than one creature, that player chooses how to pay each cost
individually. For example, if you attack with two creatures, you may pay {W} for one cost and 2 life
for the other."* And the sibling ruling extends the same to two copies of the restriction. (B)
structurally cannot express either. It is also *quieter* than the bug it replaces: it would accept
the command and charge a legal-but-not-chosen total, which is the "legal but wrong" class this
project ranks as its biggest pre-alpha risk.

**Ship (A).**

#### 5.2.2 The canonical pip order (this is the contract; write it down in three places)

The order of `total.hybrid` (and independently of `total.phyrexian`) is:

> **defenders ascending by `PlayerId`** (the `BTreeMap` iteration `combat.rs` already relies on for
> SR-9b determinism) → **within a defender, one complete copy of that defender's per-creature cost
> per creature attacking that defender** → **within a copy, restrictions in `state.restrictions`
> iteration order**.

That is **copy-major**, not pip-major: for a defender with per-creature pips `[r1, r2]` and 3
attackers the total's hybrid vec is `[r1, r2, r1, r2, r1, r2]`, **not** `[r1, r1, r1, r2, r2, r2]`.

Copy-major is chosen deliberately over pip-major because it makes "creature *k*'s pips live at
offsets `[k·P, (k+1)·P)`" true, which is the only form a UI or a ruling ("you may pay {W} for one
cost and 2 life for the other") can be stated against. Pip-major would be equally deterministic and
strictly less explicable.

Determinism is inherited, not asserted: `tax_per_creature` and `attackers_per_player` are already
`BTreeMap`s *specifically* for this (`combat.rs`'s own comment: "iteration order feeds the summed
cost and the error message, and SR-9b requires determinism"), and `state.restrictions` is an ordered
`imbl::Vector`. Note that the order does **not** depend on which creature is which — creatures are
indistinguishable for cost purposes, so no attacker→offset mapping is needed or promised.

Write the order in: (i) `Command::DeclareAttackers`' doc block, (ii) `add_mana_cost`'s doc block,
(iii) the doc block of the new `rules::queries::attack_tax_total` (§5.3).

#### 5.2.3 What changes in `combat.rs`

- `unpayable_tax_defenders` **narrows to X only** and is **renamed** (suggest `x_tax_defenders`) —
  a name asserting "unpayable" when hybrid and Phyrexian are now payable is a lying identifier of
  exactly the OOS-DP7-2 class this suite keeps re-creating. The restriction-scan guard becomes
  `if cost_per_creature.x_count > 0 { x_tax_defenders.insert(...); continue; }`.
- The rejection message loses its hybrid/Phyrexian clause, cites **CR 107.3** and **CR 601.2b** (an
  X in a cost must be announced, and `Command::DeclareAttackers` has no announcement channel for
  it), and cites the **new** seed (§11) rather than OOS-DP4-1.
- The `*cost_per_creature == ManaCost::default()` free-restriction skip (PB-DP4's E7 fix) is
  **unchanged and still correct** — a pipped cost is never `== ManaCost::default()`.
- `taxed_defenders` still unions the payable map with the X set; `has_uncosted_attack_target`'s
  CR 508.1d semantics are unchanged. Re-state in the comment that an X tax is, if anything, an even
  stronger case for the CR 508.1d carve-out.
- The affordability block flattens **once**, before `can_pay_cost`, and carries `(flat_total,
  phyrexian_life)` out of the block alongside the original pipped total:
  - `flatten_hybrid_phyrexian(&hybrid_choices, &phyrexian_life_payments)` → `InvalidCommand` on a
    bad component or an over-long vector;
  - **CR 119.4** on `phyrexian_life` against `life_total`, **before any mutation**, returning
    `InsufficientLife`;
  - `casting::can_pay_cost(&pool, &flat_total)` unchanged otherwise, with the same
    "required/available" message shape (update it to print the **flattened** total, since that is
    the thing that must be payable; keep the pipped total out of the message or print both).
- The payment block (after the tapping loops) pays `flat_total`, deducts `phyrexian_life` with a
  `GameEvent::LifeLost`, and emits `ManaCostPaid` with the **original pipped total** (matching
  `abilities.rs`/`casting.rs`). Keep PB-DP4's E6 discipline: both events stay **inside** the
  `if let Some(ps)` so a missing player cannot produce an event describing a payment that did not
  happen.

#### 5.2.4 `add_mana_cost` — what happens to its `debug_assert!`

Today:
```
debug_assert!(addend.hybrid.is_empty() && addend.phyrexian.is_empty() && addend.x_count == 0, …)
```
and its doc says hybrid/Phyrexian "are rejected by the caller before this is reached (CR
107.4e/107.4f) and are debug-asserted here so the guard cannot drift."

Post-fix that doc sentence is **false**, which is a lying-comment defect in its own right. Required
changes:

- **Narrow the assert to `addend.x_count == 0`.** X is still rejected upstream (a hard `Err`), so
  this stays an unreachable-by-construction tripwire.
- **Rewrite the doc** to state the new contract: hybrid and Phyrexian pips are **carried and
  replicated** (CR 107.4e/107.4f × CR 508.1h + the Norn's Annex "individually" rulings), X is
  rejected upstream, and the replication order is copy-major per §5.2.2.
- **Do not upgrade it to an SR-4 `expect_*`.** `combat.rs` is outside SR-4's swept surface, and
  more importantly the condition is a *caller* precondition enforced by a real `Err` three
  statements upstream, not a state lookup returning `Option`. The §6 argument (a bug is not a rules
  answer) applies; a `debug_assert` behind a hard upstream reject is the right weight.

#### 5.2.5 §5.4 — `multiply_mana_cost` and OOS-DP4-7 (do NOT dedup)

`rules/engine.rs::multiply_mana_cost` (cumulative upkeep) already replicates hybrid/Phyrexian pips
with `repeat_n` and multiplies `x_count`. After this batch `add_mana_cost` looks like it could be
expressed as `multiply_mana_cost` + a field-wise add, which is what OOS-DP4-7 proposes.

**It must not be, and this batch supplies the reason the seed did not have**: `multiply_mana_cost`'s
`flat_map(repeat_n)` is **pip-major**; §5.2.2 requires **copy-major**. A "harmless" dedup would
silently re-order the attack tax's pips and therefore silently re-interpret every
`hybrid_choices` vector a client had already built — a payment-choice reassignment with no
compile error and no test failure unless a probe pins the order (T10 does; see §9). Re-disposition
OOS-DP4-7 with this note; do **not** close it and do **not** take it.

### 5.3 New query — `rules::queries::attack_tax_total`

The turn-face-up cost is knowable from the card def. **The attack-tax cost is not knowable outside
the engine**, because it is a function of the declared attacker set, the live restriction list and
the source permanents' zones. `LegalAction::DeclareAttackers { eligible, targets }` carries no
attacker set at all — the set is chosen later, in `params.rs`/`random_bot.rs`. So without a query,
every client would have to re-implement §5.2.2's accumulation, which is precisely the drift class
OOS-RS-2 was.

Add to `crates/engine/src/rules/queries.rs` (M11-local S3's read-only advisory surface — the module
doc already states the contract: queries never mutate, and the engine re-validates everything):

```
pub fn attack_tax_total(
    state: &GameState,
    player: PlayerId,
    attackers: &[(ObjectId, AttackTarget)],
) -> Option<ManaCost>
```

Returns the **unflattened** CR 508.1h total, pips in §5.2.2's canonical order, or `None` when the
total is `ManaCost::default()`. Introduces **no new public type** (`ManaCost`, `PlayerId`,
`ObjectId`, `AttackTarget` are all already public), so it cannot move the wire fingerprint.

**Anti-drift requirement, mandatory**: `handle_declare_attackers` must call **this** function to
build its own total — factor the accumulation out of the `let (attack_tax, taxed_defenders) = {…}`
block into a shared `pub(crate)` helper that both `queries::attack_tax_total` and the validation
block call. Two copies of §5.2.2's order is how this whole seed family started. (The X/`taxed_
defenders` bookkeeping stays in `combat.rs`; only the *total* is shared.)

---

## 6. Engine change 4 — making `can_spend`/`spend` fail LOUD

### 6.1 Engaging with the existing argument rather than ignoring it

`debug_assert_flattened`'s doc block in `crates/card-types/src/state/player.rs` already argues, at
length and correctly, that it cannot be an SR-4 `state::diagnostics` `expect_*`: (i) SR-4's swept
surface is `effects/mod.rs` + `rules/resolution.rs`; (ii) — the binding reason — the `expect_*`
family is a set of **`GameState` methods**, and **SR-6 bars `card-types` from referencing
`GameState`**; (iii) the diagnostics vocabulary is about *state lookups returning `Option`*, which a
cost-shape precondition is not.

**All three of those hold and none of them are what this batch changes.** This batch does not
propose an `expect_*` call. It accepts the whole argument and attacks a different sentence — the
one PB-RS2's own review added at the bottom of that block:

> "In release, this guard fires **NEVER** — release correctness rests entirely on the three call
> sites … actually flattening … Do not treat this guard as load-bearing for release correctness; it
> is a debug/test-time tripwire only."

That is an accurate description of a guard that **fails open**. When the guard is compiled out, the
function's answer is *"yes, you can pay this"* and the pool is debited as if the pips were not
there. The failure mode is a silent **undercharge** — invisible, and it corrupts game state, which
Architecture Invariant 9 says is the one thing this project will not tolerate. The question this
batch answers is not "which assertion macro" but **"what does the function return when it has been
handed something it cannot price?"**

### 6.2 The four options, ranked, with the argument

**Ranking: 2 ≻ 3 ≻ 1 ≻ 4.** Chosen: **option 2**.

**Option 2 — `can_spend` returns `false`; `spend` panics unconditionally. CHOSEN.**
The asymmetry is the argument: **`can_spend` is a question and `spend` is an instruction.**
- For the question, a truthful and conservative answer exists. A cost with an unresolved hybrid pip
  *is not payable as given* — that is literally true, not a fudge. Returning `false` converts a
  silent undercharge (fail-open, invisible, corrupts state) into a refusal (fail-closed, visible as
  "this cost can't be paid", never corrupts state). It costs nothing in a pure library: no IO, no
  panic, no signature change, no new type.
- For the instruction, **no truthful execution exists**. Proceeding undercharges (the bug); doing
  nothing leaves a paid-for cost unpaid (also wrong); there is no return channel to say so. And
  `spend`'s documented precondition is `can_spend`, which post-fix answers `false` — so reaching
  `spend` with a residue requires a caller to have skipped or ignored a documented precondition.
  That is a program bug, and `assert!` is the standard-library-sanctioned answer to a violated
  precondition on a `()`-returning mutator (`Vec` indexing panics in release for the same reason).
- Release-panic risk is bounded by construction: after this batch **all five** payment sites
  flatten, and PB-RS2 §0.6 established the other ~18 `can_spend`/`can_pay_cost` sites are provably
  pip-free on today's corpus (re-verified by the §8 roster gate). The panic is not "a panic in
  production"; it is "a panic only if somebody reintroduces the bug", which is what a tripwire is.

**Option 3 — change the signatures to `Result`.** Second best, and genuinely principled: no panic,
no IO, the error is data. Rejected for two reasons, the second of which is decisive. (a) Cost: it
reaches ~20 engine call sites plus `casting::can_pay_cost`/`pay_cost`/`*_with_context` and their
callers, in the same commit as a correctness fix and a 320-site `Command` migration. (b) **It
launders an engine bug into a rules answer.** Every caller would `?` it into
`GameStateError::InvalidCommand`, at which point "the engine has a bug" becomes indistinguishable
from "your command was illegal" — the exact distinction SR-4's whole vocabulary exists to preserve.
An `Err` that every caller converts to a legal-looking rejection is quieter than a `false`, not
louder. Worth revisiting as a standalone refactor if `ManaPool` ever grows other precondition
failures; file as a seed.

**Option 1 — panic unconditionally in both.** Rejected. `can_spend` is called from *speculative*
contexts that legitimately ask about costs nobody has committed to paying:
`crates/simulator/src/legal_actions.rs`'s affordability filtering, the `effects/mod.rs`
affordability predicate, `casting::can_pay_cost` in `combat.rs`. Turning a question into a crash
makes a UI that merely *displays* an unactivatable ability kill the game. A crash is not more
correct than a truthful "no"; it is just louder about someone else's mistake.

**Option 4 — a crate-local diagnostics channel in `card-types`.** Rejected, and it is the worst of
the four. Any channel that survives to release is either global mutable state (a `thread_local!`
counter — in the one crate whose entire value is purity and SR-9b determinism) or a return value
(which is option 3 wearing a hat). Nothing in production would read it, so it is not louder than
today's `debug_assert`; and PB-RS2's doc already rejects inventing a `card-types` mirror of
`diagnostics`.

### 6.3 Shape

```
/// True iff `cost` still carries a pip `ManaPool` cannot price (CR 107.4e/107.4f).
pub(crate) fn mana_cost_has_unflattened_residue(cost: &ManaCost) -> bool
```
(a named, directly testable predicate — the branch it guards is not observable in a debug build,
§6.4, so the *condition* must be testable even where the *branch* is not)

- `can_spend`: `debug_assert_flattened(cost)` **retained unchanged** (the diagnostic, with its
  message and `#[track_caller]`), immediately followed by
  `if mana_cost_has_unflattened_residue(cost) { return false; }` — the behavioural, all-builds,
  fail-closed answer.
- `spend`: the `debug_assert_flattened(cost)` call is **replaced** by an unconditional
  `assert!(!mana_cost_has_unflattened_residue(cost), "<same message text>")`, keeping
  `#[track_caller]` and keeping the existing message string byte-identical so the three existing
  `#[should_panic(expected = "unflattened mana cost reached the payment path")]` tests keep
  matching.
- `debug_assert_flattened`'s doc block must be **rewritten, not appended to**. The existing "Release
  note (review finding #7)" paragraph becomes false for `spend` and misleading for `can_spend`; it
  must be replaced with the §6.2 argument in short form (question vs instruction; fail-closed vs
  precondition violation), keeping the SR-4/SR-6 paragraphs verbatim since they are still correct
  and still the reason no `expect_*` appears here.

### 6.4 Testability — stated honestly, because one branch genuinely is not observable in CI

With the `debug_assert!` retained in `can_spend`, **the `return false` branch cannot execute in a
debug build**, and CI builds debug. The plan therefore does **not** claim it is tested by the suite.
Three things are required instead:

1. **Test the predicate directly** — `mana_cost_has_unflattened_residue` on hybrid-only,
   Phyrexian-only, both, and neither. Runs in every build. This covers the *condition*.
2. **Test `spend`'s panic in every build** — the existing `..._via_spend` `#[should_panic]` test
   loses its `debug_assertions` gate (the module gate `#[cfg(all(test, debug_assertions))]` becomes
   `#[cfg(test)]`, with the two `can_spend` panic tests individually re-gated
   `#[cfg(debug_assertions)]`). This is the whole point of the unconditional `assert!` — the release
   behaviour of the mutating path becomes a thing the normal suite proves.
3. **Observe the `can_spend`-returns-false branch exactly once, by execution, and record it.** Add
   a `#[cfg(not(debug_assertions))]` test asserting `!pool.can_spend(&hybrid_cost, None)`, and run
   **`cargo test -p mtg-card-types --release`** once during the implement phase, pasting the
   observed result into the close-out. `mtg-card-types` is small, so this is cheap. The test module
   must say in a comment that this test **does not run in CI** and that its claim rests on that
   single recorded run — not assert coverage it does not have.

---

## 7. Wire fingerprints — compute, never predict; falsifiers named in advance

Both predictions below are **hypotheses with named falsifiers**. Running the gates is mandatory;
a mismatch in either direction is a **stop signal**, not a re-pin.

### 7.1 `PROTOCOL_VERSION` 32 → **33** (predicted: MOVES)

**Why**: `Command` is one of the three wire frames. Two of its variants (`DeclareAttackers`,
`TurnFaceUp`) change declared shape. The closure's **type count is unchanged** —
`HybridManaPayment` and `bool` are already reachable via `CastSpellData`/`ActivateAbility`.
Exact precedent: `- 27: PB-RS2`, the same fields on two other variants.

**Falsifier — the thing that would prove this plan wrong**: if
`cargo test -p mtg-engine --test core protocol_schema` **passes unchanged** after the fields are
added, then the scanner is not seeing the new fields (a `#[serde(skip)]` slipped in, the fields
landed on the wrong type, or the scanner's `Command` walk is broken). **Stop and investigate** —
do not bump a version to make a green gate greener. PB-DX1's lesson runs the other way too.

**Full re-pin machinery (all in the same commit):**
1. `PROTOCOL_VERSION = 33` in `rules/protocol.rs`.
2. Append a `/// - 33: PB-DX6 (2026-08-01, OOS-RS2-1 + OOS-DP4-1 — …)` **History doc row** above it,
   naming both variants, citing CR 107.4e/107.4f via CR 701.40b and CR 508.1h, and stating that the
   closure's type count is unchanged. **Never edit an existing row.**
3. Append a new `ProtocolEpoch` to `PROTOCOL_HISTORY` with the fingerprint **printed by the failing
   test**. Never edit a shipped row (`history_is_append_only` / `baseline_row_matches_frozen_const`
   exist to catch exactly that).
4. Re-pin `PROTOCOL_SCHEMA_FINGERPRINT` to the same value.
5. In `crates/engine/tests/core/protocol_schema.rs`: update `protocol_version_sentinel`'s literal
   `32` → `33`, **and** re-pin `FROZEN_HISTORY_PREFIX_DIGEST` (version 32 joins the frozen prefix),
   adding a dated one-line note beside it in the existing style.

### 7.2 `HASH_SCHEMA_VERSION` **stays 70** (predicted: UNMOVED)

**Why**: `Command` has **no `HashInto` implementation** — commands are wire frames, not hashed
state. No `GameState` field is added; no hashed struct changes shape;
`GameRestriction::CantAttackYouUnlessPay`'s `cost_per_creature: ManaCost` is unchanged in *shape*
(only in the values the engine now tolerates in it). `rules::queries::attack_tax_total` is a free
function. The `player.rs` guard adds no field.

**Falsifier**: any decision during implementation to *store* something — an attack-tax plan on
`GameState`, a pending payment, a cached total — moves HASH. If `--test core hash_schema` reddens,
the batch has silently grown a state field and must stop and re-scope, not re-pin.

Run `cargo test -p mtg-engine --test core hash_schema` and record that it passed. A prediction that
was never executed is worth nothing (PB-DX5, six MEDIUMs).

### 7.3 Sentinel re-pin — by **symbol**, and the multi-line trap

`PROTOCOL_VERSION` / `HASH_SCHEMA_VERSION` appear **353 times across 56 files**. Every assertion
pinning the literal `32` must move to `33`.

- Enumerate by **symbol**, never by the literal: `rg -n 'PROTOCOL_VERSION' crates tools`.
- **PB-DX5's lesson, applied in advance**: a single-line grep for `assert_eq!(PROTOCOL_VERSION, 32`
  **structurally cannot see a multi-line `assert_eq!`**, and PB-DX5 found two such sentinels that
  only a full workspace run caught. Therefore: after the symbol-grep pass, run
  **`cargo test --workspace --no-fail-fast`** and treat the residual failures as the authoritative
  list. Do not declare the re-pin done off the grep.
- Cite by symbol in all documentation, never by line number (the PB-DX2 lesson: a line number in a
  doc-heavy batch is stale by the next paragraph).

### 7.4 Batch-level wire sentinel

`T12 pb_dx6_wire_versions` in the primitives file: `assert_eq!(PROTOCOL_VERSION, 33)` and
`assert_eq!(HASH_SCHEMA_VERSION, 70)`, with a doc comment saying which was *computed by running
which gate* — the PB-DX5 sentinel is the template.

---

## 8. Roster gate — yes, make it permanent, in the `pb_rs2_…_roster.rs` shape

**Decision: yes.** PB-RS2's `crates/engine/tests/core/pb_rs2_hybrid_phyrexian_activation_roster.rs`
pins an **exact set** (not a floor) and argues why: "this is a narrow, specific primitive shape, not
an actively-growing authoring target, so the next card that adds one should fail this test until a
human confirms its cost is actually charged." Both of this batch's shapes have exactly that
property. New file `crates/engine/tests/core/pb_dx6_turn_face_up_and_attack_tax_roster.rs`,
registered in `crates/engine/tests/core/main.rs`.

**SR-36: enumerate `all_cards()`, never grep source.** Four pinned facts:

| # | What it pins | Expected value |
|---|---|---|
| R1 | Defs with a hybrid/Phyrexian pip in `mana_cost` **and** `CardType::Creature` in `types` — the `TurnFaceUpMethod::ManaCost` roster (CR 701.40b) | **exact set of 5**: Kitchen Finks, Blade Historian, Boggart Ram-Gang, Deathrite Shaman, Vexing Shusher |
| R2 | Defs with a pip in a `Morph`/`Megamorph`/`Disguise` cost — the `MorphCost`/`DisguiseCost` roster | **exact set: empty** |
| R3 | Defs producing `AbilityDefinition::StaticRestriction { CantAttackYouUnlessPay { .. } }` | **exact set of 2**: Propaganda, Ghostly Prison |
| R4 | Of R3, those whose `cost_per_creature` has a pip **or** `x_count > 0` | **exact set: empty** |

**Non-vacuity floors are mandatory** (an exact-set assertion where both sides go empty passes
silently — R2 and R4 are pinned *empty*, so they are the exact shape that can rot into a broken
walk):
- R2's walk must assert it saw **≥ 1** `Morph`/`Megamorph`/`Disguise` ability anywhere in the corpus
  (7 defs carry one).
- R4's walk must assert R3 is non-empty.
- R1's walk must assert it saw ≥ 1 def with a non-`None` `mana_cost` at all.

**What each entry does and does not assert** (PB-DX4's `BASELINE` lesson, and its wording): an entry
asserts *only* that this def's cost carries a pip at this site — nothing about whether the def is
otherwise oracle-correct. Say so in the failure message.

**Record completeness per R1 entry in a comment**, including that `blade_historian` is `Complete`
only by the `#[default]` derive (§0), so a future reader does not mistake three explicit markers for
three decisions.

---

## 9. Simulator, harness and the rest of the workspace

### 9.1 `crates/simulator/src/legal_actions.rs`

The TurnFaceUp enumeration block calls `can_afford(state, player, &cost)` on the **raw** cost. Note
that `can_afford` does *not* go through `can_spend` — it reads pool fields directly and falls back
to `mana_solver` — so it does **not** trip the residue guard today. But it is wrong in the offering
direction: for `{1}{G/W}{G/W}` it imposes no green/white requirement at all and only checks
`pool.total() >= mana_value()`, so it offers a flip the engine will now reject (SR-38: never offer
an action the engine rejects).

Fix by reusing PB-RS2's existing machinery **verbatim**, not by writing a second copy:
- `LegalAction::TurnFaceUp` gains `hybrid_choices` / `phyrexian_life_payments`, doc'd like
  `ActivateAbility`'s.
- Each of the **four** offer sites in that block (Morph, Megamorph, Disguise, and the
  Manifest/Cloak `mana_cost` site, plus the Manifest/Cloak morph/disguise fallbacks) calls
  `resolve_hybrid_phyrexian_plan(state, player, &cost, /* other_life_cost */ 0)` when the cost has
  pips, and offers the action only if it returns `Some(plan)`. That helper already enforces CR 119.4
  legality **and** the CR 104.3b non-suicide policy as two separate named checks — do **not**
  collapse them; PB-RS2's plan and its in-source comment both warn that a future "simplification"
  merging them reintroduces bot self-kill.

### 9.2 `DeclareAttackers` in the simulator — the structural difference from PB-RS2

`LegalAction::DeclareAttackers { eligible, targets }` does **not** carry the chosen attacker set, so
the tax total is not knowable at *enumeration* time. The payment plan must therefore be built at
*command-construction* time:
- `crates/simulator/src/params.rs::action_to_command_with_params`, `DeclareAttackers` arm: after
  `params.attackers` is known, call `mtg_engine::rules::queries::attack_tax_total(state, player,
  &params.attackers)`; if `Some(total)` with pips, call `resolve_hybrid_phyrexian_plan(state,
  player, &total, 0)` and put the plan on the command; if the plan is `None`, return the command
  with empty vectors and let the engine reject (the alternative — mutating the attacker set inside a
  param mapper — is out of scope and would hide a legality problem).
- `crates/simulator/src/random_bot.rs`'s `DeclareAttackers` arm: same treatment.
- **Do not** add a plan field to `LegalAction::DeclareAttackers`. It would be a lie: the plan is not
  determined until the attacker subset is.

### 9.3 `crates/engine/src/testing/replay_harness.rs`

- `"turn_face_up"` and `"declare_attackers"` arms: thread the two new fields.
- To let a script *express* a choice, add two optional JSON keys to each action struct, mirroring
  PB-RS2's `activate_ability`/`tap_for_mana` keys and reusing **`parse_hybrid_choices`** unchanged —
  including its all-or-nothing contract, which exists precisely because
  `flatten_hybrid_phyrexian` indexes positionally and a dropped entry **shifts every later pip's
  choice**. That warning now also governs the attack-tax vector, where the positions come from
  §5.2.2's derived order; add a cross-reference at the new call sites.
- SR-9c: confirm the action struct's `deny_unknown_fields` posture before adding keys, and confirm
  all 210 approved scripts are outcome-identical with the keys omitted.

### 9.4 SR-31 equivalence ratchet (`crates/engine/tests/scripts/harness_equivalence.rs`)

`CROSS_VALIDATED_SHAPES` currently holds 13 labels including `declare_attackers`;
`turn_face_up` is **not** a cross-validated shape at all.

- **Recommended, if feasible**: add `turn_face_up:hybrid` (a manifested Kitchen Finks driven through
  both regimes). Feasibility depends on whether `initial_state` can place a face-down permanent with
  `face_down_as` set; **check before promising it**.
- **Explicitly not recommended**: `declare_attackers:hybrid`. No card def produces a pipped attack
  tax, and the JSON regime builds state from card defs, so the shape has no honest script. Record
  that in the module's "still uncovered" list rather than inventing a synthetic def to satisfy a
  ratchet.
- Adding a covering `MoveSet` **forces** a `CROSS_VALIDATED_SHAPES` entry (the ratchet asserts set
  equality in both directions). If the runner adds neither, say so in the close-out; do not leave it
  ambiguous.

### 9.5 The rest

- `tools/tui/src/play/input.rs` (3 `DeclareAttackers` sites) and `tools/play-server/src/api.rs` (1)
  must compile. There is **no** exhaustive `Command` match in `crates/view-model/src/lib.rs` (its
  exhaustive matches are on `StackObjectKind` and `KeywordAbility`, neither of which changes here),
  so no display arm is expected — **but `cargo build --workspace` is the gate**, not
  `cargo check -p mtg-engine`. Runners miss the view-model match ~50% of the time; here it should be
  a no-op, and "should be" is what the workspace build is for.
- **`bare_lookup_ratchet` ceilings**: `src/rules/combat.rs` is capped at **15** and
  `src/rules/engine.rs` at **22** bare lookups. Both files gain code in this batch. Prefer the
  existing `expect_*`/`state.player_mut(...)?` idioms already used at both sites so the ceilings do
  not move; if one must move, it needs a justification comment in the ratchet, not a silent bump.

---

## 10. Yield — expected **0 completeness flips**, argued

**0 flips, and this is a pre-commitment, not an estimate.**

- **Turn-face-up half**: the three live-wrong defs (Kitchen Finks, Blade Historian, Boggart
  Ram-Gang) are **already `Complete`**. This batch makes existing `Complete` defs behave correctly;
  nothing becomes newly *authorable*, so no marker moves. Exactly PB-DX5's shape. The two
  deck-illegal members (`deathrite_shaman` `known_wrong`, `vexing_shusher` `partial`) carry blockers
  unrelated to pip payment and must **not** be opportunistically promoted — if the runner believes
  otherwise, that is a separate oracle-verified decision requiring a citation in the commit, and the
  default is *file, do not promote*.
- **Attack-tax half**: 0 defs carry a pipped or X attack tax, so the half is purely latent.
  **Norn's Annex** (`{3}{W/P}{W/P}`, "Creatures can't attack you **or planeswalkers you control**
  unless their controller pays {W/P} for each of those creatures") is the obvious card this batch
  unlocks and it is **not in the corpus**. Do **not** author it. Its planeswalker half is a real,
  independent gap: `GameRestriction::CantAttackYouUnlessPay` is deliberately **player-only** —
  `combat.rs` scopes it to `AttackTarget::Player` on the strength of the Propaganda ruling — so a
  Norn's Annex authored today would silently let a creature attack a planeswalker for free. That
  makes it `partial` at best, which is a coverage *no-op*, and authoring it would be the exact
  yield-inflation `feedback_pb_yield_calibration` warns about. **File it as a seed** (§11) with the
  planeswalker gap named.
- **Coverage therefore holds at 1,137/1,804 = 63.0%**, with the `tools/authoring-report.py` body
  byte-identical.

**Consequence, stated in advance**: because no marker moves, **`random_deck`'s `Complete` pool is
unchanged, so no seeded deck re-deals and the play-server seed pins do not need re-reading.** If any
marker *does* move — which would contradict this section — the runner must re-read the play-server
seed fixtures (precedent `b24a9685`, and PB-DX4's Rograkh incident, where one demotion shifted every
seeded deck in the workspace) **before** declaring the batch green. Say which of the two happened.

**What this batch is worth instead of coverage**: three shipped, deck-legal creatures stop being
flippable for a third of their printed cost in release; a whole class of attack tax stops being
rejected as unpayable; and the payment path stops being able to fail *open*.

---

## 11. Seeds to file (`docs/audits/decision-point-audit.md` §8.1)

- **OOS-DX6-1** — `Command::DeclareAttackers` still cannot announce **X** in an attack tax
  (CR 107.3 / 601.2b-analogue). Needs an x-announcement channel, not a payment-choice vector.
  Carries the rejection message's citation, replacing OOS-DP4-1's.
- **OOS-DX6-2** — `GameRestriction::CantAttackYouUnlessPay` is **player-only** and does not cover
  "or planeswalkers you control", so **Norn's Annex** is not authorable as `Complete` even after
  this batch. Names the one card and the one field.
- **OOS-DX6-3** — `ManaPool::can_spend`/`spend` could be `Result`-returning (§6.2 option 3). Records
  the argument *against* doing it inside a correctness batch and *for* revisiting it standalone.
- **OOS-DX6-4** — `Command::DeclareAttackers` boxing into `DeclareAttackersData` (§3.3 option B),
  the SR-10 treatment, deferred for digest-attributability.
- **OOS-DP4-7 — re-dispositioned, not closed**: §5.4's copy-major/pip-major divergence is a new,
  stronger reason not to dedup `add_mana_cost` onto `multiply_mana_cost`.
- **Verify every cite by symbol on closure.** OOS-DP6-8's documentation-rot class has bitten this
  suite twice; both PB-DX1 riders' §8.1 cites were stale when checked.

---

## 12. Verification checklist

- [ ] Step-0 probes written **first**; every pre-fix number **observed** per §2, with the build mode
      (§2.0) stated for each; any unobservable claim labelled **vacuous**, not estimated
- [ ] `handle_turn_face_up`: flatten before the `mana_value() > 0` gate; CR 119.4 check before any
      mutation; life deduction a **sibling** of the gate; `ManaCostPaid` emits the **original**
      pipped cost; all three `TurnFaceUpMethod` arms covered by the one block
- [ ] `combat.rs`: pips replicated into the total (**copy-major**, §5.2.2), flattened once; CR 119.4
      pre-mutation; `x_tax_defenders` renamed and narrowed; rejection message no longer claims
      hybrid/Phyrexian are unpayable and cites OOS-DX6-1
- [ ] `add_mana_cost` assert narrowed to `x_count`; its **doc rewritten**, not appended to
- [ ] `rules::queries::attack_tax_total` added; `handle_declare_attackers` calls the **same** shared
      accumulation helper (no second copy of the pip order)
- [ ] `can_spend` fail-closed + `spend` unconditional `assert!`; `debug_assert_flattened`'s doc
      **rewritten** to replace the now-false "fires NEVER" paragraph
- [ ] `cargo test -p mtg-card-types --release` run **once**, result pasted into the close-out (§6.4)
- [ ] All `Command::DeclareAttackers` (320) and `Command::TurnFaceUp` literals migrated;
      `rg 'Command::(DeclareAttackers|TurnFaceUp) \{'` shows no site missing the fields
- [ ] `PROTOCOL_VERSION` **computed**, not assumed: gate run, 32 → 33 if and only if it reddened;
      History doc row appended; `PROTOCOL_HISTORY` row appended; `PROTOCOL_SCHEMA_FINGERPRINT`
      re-pinned from test output; `protocol_version_sentinel` and `FROZEN_HISTORY_PREFIX_DIGEST`
      updated in `tests/core/protocol_schema.rs`
- [ ] `HASH_SCHEMA_VERSION` **computed**: `--test core hash_schema` run and green at 70; if it moved,
      **stop**
- [ ] Sentinels re-pinned by **symbol** grep **and** confirmed by
      `cargo test --workspace --no-fail-fast` (multi-line `assert_eq!`s, PB-DX5)
- [ ] `pb_dx6_turn_face_up_and_attack_tax_roster.rs` pins R1–R4 with non-vacuity floors
- [ ] Simulator offers no unpayable and no suicidal plan on either path; CR 119.4 and CR 104.3b stay
      two separate named checks
- [ ] SR-31 ratchet: `turn_face_up:hybrid` added **or** its absence recorded with a reason
- [ ] 210 golden scripts green, 0 new skips; any repair CR-cited, none by weakening an assertion
- [ ] `bare_lookup_ratchet` ceilings for `combat.rs` (15) / `engine.rs` (22) unmoved
- [ ] `cargo build --workspace` (TUI + play-server + simulator + view-model)
- [ ] `cargo test --all`; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --check` **and**
      `tools/check-defs-fmt.sh` (SR-35)
- [ ] Coverage re-measured with `tools/authoring-report.py`; **0 flips** confirmed, or the
      play-server seed pins re-read (§10)
- [ ] Benches spot-checked (`full_turn_4p`, `priority_cycle_4p`) — the flatten is once per
      declaration, not per attacker, so no movement is expected; measure, don't assert
- [ ] Every cite in the plan and the seeds re-verified **by symbol** on closure

---

## 13. Risks & edge cases

1. **HIGHEST — writing a pre-fix number instead of reading one.** §2.0's debug-vs-release split is a
   trap purpose-built to produce a plausible false claim ("it charges {1}"), which is true only in a
   build nobody runs. Four of the last five batches shipped exactly this defect. Read every number.
2. **The positional index against a derived cost.** `hybrid_choices[i]` indexes a cost the client
   cannot see. Mitigated structurally by §5.3's query plus §5.2.2's written-down order, but it
   remains the weakest joint in the design. A probe (T10) must pin the order explicitly — two
   defenders, two restrictions, two attackers, an asymmetric plan, and an assertion on *which* mana
   was spent — or a future refactor will silently permute it.
3. **A "harmless" dedup of `add_mana_cost` onto `multiply_mana_cost`** silently re-orders those pips
   (§5.4). T10 is the only thing that would catch it.
4. **320-site migration fatigue.** The likeliest miss is the TUI or `tools/play-server`.
   `cargo build --workspace` is the gate.
5. **Fingerprint re-pin without a bump.** `protocol.rs` documents this exact cheat and
   `history_is_append_only` / `frozen_prefix_is_pinned` exist to catch it. Take the value from the
   failing test's output.
6. **`spend`'s unconditional `assert!` reddening an unrelated suite.** If any existing test hands a
   pipped cost to a non-flattening path, it will now panic in release too. **That is the guard
   working.** Fix by flattening at that site, or prove the test was encoding the bug — never by
   weakening the assert or re-gating it to debug.
7. **The new `ManaCostPaid` event on the turn-face-up path** changes event streams (§5.1.5). Expect
   golden-script and unit-test repairs. Repair them with a CR citation; do not delete the event.
8. **CR 508.1i is still not honoured** (OOS-DP4-2): choices are announced with the declaration, so a
   player cannot activate mana abilities *between* determination and payment. The Norn's Annex
   ruling ("pays either {W} or 2 life as attackers are being declared") makes the *choice* timing
   correct; the *mana* timing deviation is pre-existing and unchanged. Do not let this batch's
   design notes imply otherwise.
9. **`{0}` and CR 118.5.** A `{0}` restriction is skipped before it can reach the pip logic
   (PB-DP4's E7 fix). Unchanged, but confirm a `cost_per_creature` that is *all* Phyrexian-paid-
   with-life still reaches the payment block: it flattens to `{0}` with `phyrexian_life > 0`, so the
   `total != ManaCost::default()` guard must be evaluated on the **pipped** total, not the flattened
   one, or the whole payment silently vanishes. **This is a real and easy bug to write.** Pin it
   with T11.
10. **CR 119.4b.** Paying 0 life is always legal. `phyrexian_life == 0` must never hit the CR 119.4
    guard — mirror `abilities.rs`'s `if combined_life_cost > 0` wrapper exactly.
