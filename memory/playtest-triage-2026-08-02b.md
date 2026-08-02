# Second-human-playtest triage — 2026-08-02

Source: `test-data/bot testing notes 2.md` (user's notes from the second extended human
playtest of the M11-local browser client, 2026-08-02 11:30). Findings are numbered
**G1..G13** so they cannot collide with the first triage's F1..F10
(`memory/playtest-triage-2026-08-02.md`). Every functional claim was verified against code
on this branch (`scutemob-183`, forked from main `8195b109` = `b76b1df4` + collect
bookkeeping) by seven read-only chain-verification passes before anything was written here.
**Zero fixes implemented** — this file is a triage record and a dispatch proposal.

Line cites are snapshots (OOS-DP6-8 class) — re-verify by symbol before building on them.
Bare filenames are unique across the workspace but several resolve somewhere non-obvious:
`events.rs` and `command.rs` are `crates/engine/src/rules/`; `stubs.rs` is
`crates/card-types/src/state/`; `params.rs`, `setup.rs`, `deck.rs`, `local_game.rs`,
`legal_actions.rs`, `heuristic_bot.rs`, `random_bot.rs` are `crates/simulator/src/`;
`session.rs`, `api.rs`, `view.rs`, `main.rs` are `tools/play-server/src/`; `redact.rs` and
`event_view.rs` are `crates/view-model/src/`; the `.svelte`/`.js` files are under
`tools/play-server/frontend/src/lib/` except the `Zone*.svelte` set and `cardTooltip.js`,
which are the replay viewer's, shared in place via the `$viewer` alias.

> **The notes file itself is not on this branch.** It lives untracked in the main worktree
> (`/home/skydude/projects/scutemob/test-data/bot testing notes 2.md`). This task's
> conflict-safety constraint (scutemob-182 in flight) permitted exactly one file, so the
> source was read in place rather than committed. Whoever commits it later should not
> re-triage from it — this file is the record.

## Headline

**The single most important finding is G1, and it was not in the notes as a bug report.**
The user wrote "i clicking confirm once a forest was chosen did nothing". That one dead
button is `structuredClone()` called on a Svelte 5 `$state` proxy, which throws
`DataCloneError` into an uncaught handler — and the same three-line pattern sits in
`SearchPicker`, `PartitionPicker` and `CostPicker`. **Five CR flows the project believes
shipped have never worked in a browser**: library search (CR 701.23), scry (701.22a),
surveil (701.25a), sacrifice additional costs (118.8) and Squad (702.157a). UI-1 and UI-2
each proved their *wire* with HTTP probes and neither could test the *component*, because
no frontend test harness exists (M11-local plan §8 R7). This is that risk collecting.

Beyond G1, the shape of this triage differs sharply from the first one. The first triage's
headline was "**zero engine bugs**". This one has two genuine engine gaps (G3, G7), both of
which bump the wire — and three of the defects (G2, G4, G5) are cases where **a correct
engine implementation exists and is unreachable**, the F7 pattern repeating three more times.

**Also worth stating plainly**: the user's own two causal hypotheses were both wrong, and
both were wrong in the direction of blaming a mechanism that is innocent. "Could have been
the priority pass being set to until my turn?" (G3) — the auto-passer explicitly refuses to
answer non-priority decisions and would have handed control back. "Bots are still wasting
mana" (G5) reads as an F5 regression — F5 is genuinely closed and the fix is in the binary
the user played. A playtester reports symptoms accurately and mechanisms speculatively;
triage exists to keep the second from being filed as the first.

## Classification summary

| G | Finding | Class | Proposed task |
|---|---|---|---|
| G1 | Picker Confirm is a no-op (`structuredClone` on a `$state` proxy) | **NEW DEFECT (HIGH)** | UI-4 |
| G2 | Mulligan re-rolls every seat's deck and commander (CR 103.5) | **NEW DEFECT (HIGH)** | SIM-4 |
| G3 | Effect-driven discard has no decision point (CR 701.9b) | **NEW DEFECT** | ENG-1 |
| G4 | Activation costs have no payment channel (Yahenni, Altar) | **NEW DEFECT** | SIM-6 |
| G5 | Bots waste mana: non-atomic auto-tap + bots announce no targets | **NEW DEFECT** (not an F5 regression) | SIM-5 |
| G6 | Deadly Rollick uncastable for free | **KNOWN LIMITATION** (R4) | none |
| G7 | Targets absent from the event log | **NEW DEFECT (engine gap)**; stack half **REFUTED** | ENG-2 |
| G8 | Concede placement / cancel-vs-concede semantics | **UX FEATURE** (+ G1 interaction) | UI-5 |
| G9 | Show the whole library when searching | **UX FEATURE** (rules-aligned, CR 701.23a) | UI-6 |
| G10 | "Tap land for mana" clutters the action list | **UX FEATURE** (do NOT hide — see entry) | UI-5 |
| G11 | Hover card name interferes with the card image | **UX FEATURE** | UI-5 |
| G12 | Artifacts/enchantments should sit above lands | **UX FEATURE** | UI-5 |
| G13 | Same-name lands should stack when tapped | **UX FEATURE** | UI-5 |

---

## Verified findings

### G1 — Picker Confirm is a dead button: `structuredClone` on a Svelte 5 `$state` proxy
**NEW DEFECT, HIGH.** Notes: *"i clicking confirm once a forest was chosen did nothing"*.

`ActionBar.svelte:205` declares `let activeOption = $state(null)`; `:349` assigns the plain
store object into it, which Svelte 5 wraps in a proxy and deep-proxies on read. The
`$derived` chain `:279` → `:282` → the `template={currentShape.template}` prop at `:675`
therefore hands each picker a **Proxy**. `SearchPicker.svelte:112` then calls
`structuredClone(template)`. The structured-clone algorithm rejects proxies outright:

```
$ node -e 'const p=new Proxy({SearchLibrary:{found:30}},{}); try{structuredClone(p)}catch(e){console.log(e.name,e.message)}'
DataCloneError #<Object> could not be cloned.
```

(Reproduced independently at triage time, not taken from the investigation.) The throw
escapes an ordinary Svelte 5 DOM handler — there is **no `try` on the picker emit path**, no
`<svelte:boundary>` anywhere, and no global handler — so the DOM is untouched: picker stays
open, no error strip, no HTTP request. That is precisely "did nothing". (The frontend does
have four `try` blocks — `api.js:46`, `stores.js:120`, `stores.js:153`,
`PlayApp.svelte:161` — but the throw happens inside `SearchPicker.emit`'s own synchronous
click handler, *before* `onConfirm`, so none of them is in the chain. Stated precisely here
because a UI-4 worker who greps for `try`, finds four hits and discounts the diagnosis would
be wrong.)

`grep -rn '\$state.snapshot' tools/play-server/frontend/src/` returns **nothing**;
`package.json:13` pins `svelte: ^5.45.2`. `$state.snapshot()` exists in Svelte 5 for exactly
this reason.

**The blast radius is three components and five CR flows**, all on the same proxied-template
path (verified by grep at triage time — exactly three hits, no more):

| Site | Dead flow |
|---|---|
| `SearchPicker.svelte:112` | library search (CR 701.23) — UI-1 / F8 |
| `PartitionPicker.svelte:139` | scry (CR 701.22a) **and** surveil (CR 701.25a) — UI-1 / F8 |
| `CostPicker.svelte:150` | sacrifice (CR 118.8) **and** Squad (CR 702.157a) — UI-2 / F9 |

`DiscardPicker`, `TargetPicker`, `ValuePrompt`, `AttackerPicker` and `BlockerPicker` do not
clone and are unaffected — which is why the browser works at all. `JSON.stringify` reads
through proxies fine (`api.js:29-35`), so the submit path itself is innocent.

**Why nothing caught it**: the wire is genuinely proven —
`main.rs:3695 test_ui1_search_pick_is_answered_over_http` drives a real non-default search
pick and asserts the card left the library; `params.rs:240-242`/`:613-620` forward the
answer; `view.rs:800` deserializes it. Every layer below the component is tested and
correct. `ActionBar.svelte:120-138` states in its own header that the components are
untested. **Generalisable: an end-to-end HTTP probe that starts below the browser proves
the channel and says nothing about the only part a human touches.**

**Honest limit on this diagnosis**: it is derived from source plus a node reproduction of
the throw, not from an observed browser stack trace (`node_modules` is not installed in this
worktree and there is no frontend harness). It is falsifiable in one step — open the console
during a search and look for `DataCloneError`. **Do that first in UI-4**, before editing.

**Fix**: 3 lines — `$state.snapshot(x)` for `structuredClone(x)` at the three sites, or one
snapshot in `ActionBar` before `template` is passed down. Strongly recommended alongside:
wrap the emit in `try/catch` and route failures to the existing error strip
(`ActionBar.svelte:523-534`), because a silent throw is what turned a 3-line bug into a
conceded game. Longer term this is a **class** (any `structuredClone`/`postMessage`/
`IndexedDB` on Svelte state) and deserves a grep gate.

### G2 — A mulligan re-rolls every seat's deck and commander
**NEW DEFECT, HIGH.** Notes: *"mulligans seem to change decks instead of drawing a new hand"*.

**CONFIRMED, and worse than the coordinator's hypothesis framed it.** `POST
/api/game/mulligan` throws the whole `GameState` away and rebuilds the table from a
perturbed seed. Because `DeckSource::RandomPerSeat` draws each seat's **commander and 99**
from that same seed's RNG, all four seats get a brand-new deck *and a new commander* — and
the command zone is public (CR 903.6), so the user watched three opponents' commanders
change.

Chain: `PlayApp.svelte:469` → `main.rs:176` → `api.rs:1248-1249` →
`session.rs:378-412` (`self.game = game`, a fresh `LocalGame::start`, **no `Command` is ever
constructed**) → `setup.rs:352-362` `redeal` (`seed: redeal_seed(...), ..cfg.clone()` — the
`..cfg.clone()` carrying `RandomPerSeat` forward is the load-bearing detail) →
`setup.rs:206` one `StdRng` for the whole build → `setup.rs:227-239` per-seat `random_deck`
→ `deck.rs:53` `commanders[rng.random_range(..)]`, `:93-94` nonlands, `:100-102` nonbasic
lands, `:145` basics. **Every card in every deck is a function of the seed.**

**The engine's correct implementation is unreachable dead code.**
`commander.rs:802-877 handle_take_mulligan` is CR 103.5-faithful — hand into library,
seeded `library.shuffle`, `LibraryShuffled`, draw 7; the card multiset is invariant across
it. `handle_keep_hand` (`:894-`) even enforces the bottoming count. But
`legal_actions.rs:427-429` and `local_game.rs:931-932` both gate the mulligan actions on
`turn_number == 0`, while `builder.rs:59` defaults `turn_number: 1` and `setup.rs:302` calls
only `first_turn_of_game()`. **Nothing in the workspace ever builds a turn-0 state**, so
both arms are dead and `PlayerState::mulligan_count` is permanently 0 on this path. This is
the third instance in two triages of the F7 pattern: engine correct, caller blind.

**CR 103.5** (MCP-verified): *"To take a mulligan, a player shuffles the cards in their hand
back into their library, draws a new hand of cards equal to their starting hand size, then
puts a number of those cards equal to the number of times that player has taken a mulligan
on the bottom of their library in any order."* The mulligan is a permutation of a **fixed**
library ∪ hand multiset; rebuilding from a new seed replaces the multiset. Two further
sentences of the same rule also break: bot seats are never asked to declare, and a bot that
implicitly kept has its hand discarded on every human mulligan. CR 903.5's deck-construction
contract goes with it. (Correction to the dispatch brief: **CR 103.4 is starting life
totals**, not the mulligan rule; 103.5 alone, with 103.5c for the multiplayer free first.)

**Scope of the damage**: all seats' decks, commanders, hands and library orders change.
Starting player does not (`builder.rs:237`, `setup.rs:215-217` — the human is always seat 1).
Bot RNG seeds do not (`session.rs:385` uses the unperturbed cfg). **`DeckSource::Fixed` is
immune** — with it, `redeal`'s perturbed seed reaches only the shuffle, which is the
CR-correct behaviour — but `session.rs:165` hard-codes `RandomPerSeat` for every play-server
game, so the browser never takes the immune path.

**Fix**: ~40-60 lines, simulator + play-server, **0 engine lines**. Factor `setup.rs:227-244`
into `resolve_decks(cfg)`; have `session::new_game` resolve once and store
`cfg.decks = DeckSource::Fixed(resolved)`. `redeal` then needs no change at all. Optional
residual (~20 lines): per-seat RNG streams at `setup.rs:280` so only the mulliganing seat
reshuffles — which `redeal`'s own doc (`setup.rs:344-351`) already says belongs here.

**The missing gate is why this shipped**: `crates/simulator/tests/setup.rs:294`
`test_redeal_produces_a_different_hand` asserts only that the *hand* differs. Nothing asserts
the deck and commander are **unchanged**. Add that pin.

**Three adjacent defects found on the same walk**, each worth a seed:
1. **A mulligan can fail outright.** `redeal` re-runs `validate_deck` (`setup.rs:250-256`)
   against freshly rolled decks, so it can return `InvalidDeck`/`NoDeckForSeat` on a table
   that built fine a moment earlier. A rules operation that cannot fail is implemented as
   one that can.
2. **`GameResult.seed` does not reproduce a mulliganed game.** `session.rs:386-390` passes
   the *base* seed to `LocalGame::start` while the state came from `redeal_seed(...)`;
   `local_game.rs:289` reports it verbatim. `/api/game/report` works around it by shipping
   `mulligan_count` separately (`view.rs:673-678`) — `GameResult` itself has no such term.
3. **`Command::TakeMulligan`/`KeepHand` and `DecisionKind::Mulligan` are workspace-wide dead
   code** for every simulator-built game (fuzzer, TUI, play-server). Exercised only by
   direct-`Command` engine tests — a coverage cliff behind a green suite.

**And it was documented, just not in the terms that mattered**: `setup.rs:336-351`,
`session.rs:359-369`, `api.rs:1207-1224` and `PlayApp.svelte:466-467` all describe the
whole-table rebuild and even name the commander re-roll. None says the **decklists** change;
every doc frames it as re-rolling *seats*, which reads as "new hands". The playtester is the
first to name the consequence. **Generalisable: a limitation documented in terms of its
mechanism, not its consequence, does not warn anyone.**

### G3 — Fell Specter: effect-driven discard has no decision point at all
**NEW DEFECT.** Notes: *"Fell Spector entered, and the bot chose me — card was automatically
discarded — could have been the priority pass being set to until my turn?"*

**The user's hypothesis is REFUTED, with evidence.** `stores.js:401-405` stops the
auto-passer on any non-`Priority` decision, and its own doc (`:342-347`) says why: *"an
auto-passer that answered them with a default would be making the human's decisions for
them — the exact defect UI-1 existed to remove."* Had a decision existed, control would have
come back. **Exactly one mechanism is at fault, and it is upstream.**

`effects/mod.rs:1202-1208` executes `Effect::DiscardCards` inline with no deferral branch —
unlike the search/scry/surveil arms at `:3606`, `:3695`, `:3771` — calling
`discard_cards` (`:9368-9376`), which picks `.min_by_key(|id| id.0)`: **the lowest
`ObjectId`, i.e. the leftmost/oldest card in hand**, and moves it at `:9394`. The human is
never asked.

**CR 701.9b** (MCP-verified): *"By default, effects that cause a player to discard a card
allow the affected player to choose which card to discard."* Fell Specter prints no "at
random" and no "of your choice", so the default applies. Direct violation.

The card def is **correct and innocent** (`fell_specter.rs:26-38`, `Complete`), and there is
no `chooser` field on the effect to have set — `card_definition.rs:1407-1411` declares
`DiscardCards { player, count }` and nothing more. The bot targeting the human is also
correct and *does* have a blocking decision (`BlockingDecision::TriggerTargets`, PB-DP8).

`engine.rs:145-166` confirms only three blocking kinds exist (`CleanupDiscard`,
`TriggerTargets`, `EffectChoice`) and `stubs.rs:906-922` confirms the resolution-time
question type has three variants, **all library-zone**. So the coordinator's hypothesis is
CONFIRMED with a refinement that changes the fix size: PB-DP7 built cleanup discard only,
but PB-DP9's `EffectChoice` *is* generic resolution-time-choice machinery. **The gap is one
missing question variant, not missing machinery.** `StubProvider` has nothing to offer
because nothing was ever recorded (`legal_actions.rs:346-413`, three arms, then `return`).

**Distinct from F8, one link earlier.** F8 was a view/params-layer gap: the decision existed
and the candidate data was thrown away between provider and browser (UI-1 closed the browser
half). Here there is no decision, no `LegalAction`, no candidate set — nothing for any client
to echo. Same distinction CARDS-1 drew for `OOS-CARDS1-3` ("there is no action to pick" vs
"the picker never asks"). No existing seed covers it: `OOS-DP7-6` is *cleanup* discard,
`OOS-DP9-1/7` are scry/search, `OOS-DP8-2` is trigger targets. **File a new seed.**

**Fix**: one PB-sized batch, smaller than PB-DP7/DP9 because the suspension/replay
machinery, the wire command, the provider arm and `DiscardPicker.svelte` all already exist.
New `EffectChoiceQuestion::Discard` + `EffectChoiceAnswer::Discard` in `stubs.rs:906-922`;
suspend-and-replay in the `DiscardCards` arm; arms in `default_effect_choice_answer`
(`effects/mod.rs:380-400`), `handle_answer_effect_choice`, `state/hash.rs:3211-3223`,
`replay_harness.rs:1117-1139`. `private_to()` stays `Some(player)` — the hand is hidden and
the answerer owns it, so Invariant 7 holds by the same argument as the library questions.
**This bumps PROTOCOL (SR-8) and probably HASH — both gate-computed, never predicted.**
Watch: `discard_cards`' second caller is `Effect::WheelHand` (`:1236`), which snapshots hand
size before disposal, so a replay must not double-count; short-circuit `count >= hand.len()`
(a full-hand discard needs no choice, and skipping the question avoids perturbing fuzz
seeds). Corpus check: **21 defs** use `Effect::DiscardCards` (23 occurrences) and **zero**
print "at random" or
"opponent chooses", so the CR 701.9b default covers the entire live corpus and the missing
`chooser` field can be deferred without a known-wrong def.

**Adjacent, and the most valuable thing on this walk — the "engine picks for the player"
census.** `effects/mod.rs`' own module doc (`:15`) states the policy: *"deterministic
fallback in M7: the engine picks the first matching option"*. Sites where a player is
entitled to choose and does not: `:4222` `SacrificePermanents` (CR 701.21a), `:3157`
Bolster, `:3274` Amass, `:3450` top-N zone move, `:4467`/`:4479` auto-select-all,
`:6027`, `:7324`/`:7386`, `:2822`, `:5904`/`:5523`/`:8940`, `:4143` optional cost (never
pays), `:2943` ("any one color" adds colorless). **Every one of those carries a
`deferred to M10+` comment. `discard_cards` does not** — its doc states the behaviour as a
design property (*"first by ObjectId, deterministic"*). That is why it survived the PB-DP
decision-point audit and surfaced only in a human's game. **Generalisable: a deliberate
placeholder that documents its mechanism instead of its debt is invisible to every audit
that greps for the debt.** Cheapest follow-on once the discard question exists:
`SacrificePermanents` (`:4222`) — a human losing an unchosen creature is the same complaint
class as `OOS-UI2-5`.

### G4 — Yahenni / Altar of Dementia: activation costs have no payment channel
**NEW DEFECT.** Notes: *"Yahenni: could not activate sacrifice ability in response to a Crux
of Fate … this happened during the first testing run … same thing happened with Altar of
Dementia"*.

**The dispatch brief's framing was too narrow, and the correction matters.** The question
posed was whether `StubProvider` offers activated abilities while a spell is on the stack.
**It does.** `legal_actions.rs:861` iterates the battlefield at function-body level, outside
any stack guard; `:873` is the *only* stack gate and applies solely to `sorcery_speed`
abilities, which neither card is (`timing_restriction: None`). The human gets priority, and
the button appears.

What is missing is the **cost-payment channel**. `LegalAction::ActivateAbility`
(`legal_actions.rs:93-102`) has four fields and **no sacrifice**; `legal_actions.rs:883-918`
checks mana/hybrid/Phyrexian/life and **never consults `ability.cost.sacrifice_filter`**, so
the offer is emitted even with zero eligible creatures; `view.rs:1465-1472`
`additional_costs_view` early-returns for anything that is not `CastSpell`, so
`ActionBar.svelte:318` never enters the cost stage and `CostPicker` never renders; and
`params.rs:339-345` hardcodes `sacrifice_target: None`. The engine then refuses:
`abilities.rs:1033-1038` → `InvalidCommand("ability requires sacrificing a permanent as
cost: sacrifice_target must be Some (CR 602.2)")` → **422**.

**This is exactly the SR-38 shape UI-2 fixed for `CastSpell` and never extended to
`ActivateAbility`** — same defect, adjacent command. The engine is innocent: the wire field
exists (`command.rs:111-116`) and
`crates/engine/tests/casting/animated_creature_sacrifice_cost.rs:298-313` drives it through
`process_command` successfully today. **Same root cause for both cards.** (One correction to
the notes: Altar of Dementia's printed cost is `Sacrifice a creature:` with **no tap symbol**
— MCP-verified, and the def matches. The tap is irrelevant either way.)

**Not previously filed — and the user's "this happened during the first testing run" is both
right and easy to misread.** The first notes do carry a sacrifice-cost failure
(`test-data/bot testing notes.md:76-78`, Life's Legacy, *"ui provides no option to sacrifice
a create"*) — but that is a **spell** additional cost (CR 118.8), which became F9 and was
closed by UI-2. Yahenni and Altar are **activation** costs on a different command. So the
user saw the same *shape* twice and reasonably called it the same bug; UI-2 closed the spell
half and the activation half was never filed. **Do not let the recollection downgrade this to
a recurrence — it is the unclosed sibling.** The nearest seeds are likewise all spells:
`OOS-UI2-4` is scoped
to `CastSpellData.additional_costs`; `OOS-OS6-1` (→ PB-DX12) is the **multi**-sacrifice case
needing a wire reshape, whereas single `sacrifice_target` already exists — so this is
strictly cheaper and disjoint. `README.md` limitations 18-20 name only the spell side.

**Fix**: simulator + play-server, **0 engine lines, 0 wire changes, PROTOCOL/HASH unmoved**
(`LegalAction`/`ActionParams` are simulator types, not protocol types). ~150 lines:
descriptor on `LegalAction::ActivateAbility` mirroring `abilities.rs:1070-1108`' filter match
*including* the `object_cant_be_sacrificed` check (`:1063`); **suppress the offer when the
eligible set is empty** (the SR-38 half, mirroring `offerable_cast_plan`); an `ActionParams`
channel forwarded at `params.rs:345`; `additional_costs_view` generalised (`SacrificeCostView`
is reusable as-is); `validate_additional_cost_params` extended. **No new frontend
component** — `CostPicker` is already wired through `option.costs`.

**Systematic, not two cards**: 53 defs reference `Cost::Sacrifice(`, 9 reference
`Cost::DiscardCard` (the identical gap — fold it in while the channel is open). The **TUI has
the same defect** at `tools/tui/src/play/input.rs:287`.

**Latent card-def defect found on the walk**: `yahenni_undying_partisan.rs` prints "Sacrifice
**another** creature" but leaves `exclude_self: false`, which `flatten_cost_into`
(`replay_harness.rs:4622`) copies into `sacrifice_exclude_self`. **Yahenni will legally
sacrifice itself the moment the channel exists** — same family as the
`wight_of_the_reliquary`/`vampire_gourmand` notes in
`memory/card-authoring/marker-sweep-2026-07-16.md:788,802`. One-line fix; do it in the same
batch or it becomes a fresh bug the fix creates.

### G5 — Bots still waste mana: non-atomic auto-tap, and bots announce zero targets
**NEW DEFECT — and *not* an F5 regression.** Notes: *"bots are still wasting mana during the
beginning of their turns"*.

**F5 is genuinely closed and the fix was in the binary the user played.**
`heuristic_bot.rs:244` scores `TapForMana` **0** vs `PassPriority` **1** (`:249`); selection
is a strict max (`:326-337`) and `PassPriority` is always offered (`legal_actions.rs:441`),
so `HeuristicBot` can never *choose* to tap. The stale-binary hypothesis is refuted by
timestamps: the play-server the user played (PID 1255158, `--bot heuristic --seed 0`) started
`10:57:05` with a binary mtime of `10:57:14`; SIM-2 merged `08:28:20`, the collect `09:38:39`,
main's HEAD `10:29:46`, and the notes were written `11:30:35`. The play-server seats
`HeuristicBot` by default and by live confirmation (`main.rs:51-54`, `session.rs:190-191`,
and `GET /api/game` returning `"bot":"Heuristic"`).

**The live mechanism is different and worse.** The human's `submit` path applies
tap-then-cast **atomically** (`local_game.rs:549` → `apply_sequence`, documented at `:694`
as existing precisely to prevent "the tap succeeded but the cast was rejected"). The **bot**
path does not: `local_game.rs:462-468` builds `[taps…, cast]` and `:471-472` applies them
**one at a time**; on failure `:474-491` passes priority and breaks, **with the taps already
committed and the error `e` discarded, never logged**. `auto_tap_commands_for`
(`:652-691`) prices only the mana cost and never looks at targets, so it funds casts that
cannot legally be announced.

**Why the cast is rejected — structural**: bots announce **zero targets, always**.
`random_bot.rs:142` (shared by `HeuristicBot` via `heuristic_bot.rs:19`) builds
`ActionParams::default()` and fills only attackers/blockers (`:150-182`);
`Bot::choose_targets` has **zero call sites outside the bot impls' own `choose_action`**
(already recorded at `memory/m11-session-plan.md:117`) — the trait method is implemented four
times and reached from the driver never; its one non-impl mention is a delegating test
wrapper at `crates/simulator/tests/sim2_mana_intelligence.rs:1157`. The offer gate never
checks targets either — `grep
TargetRequirement crates/simulator/src/legal_actions.rs` returns nothing. The engine then
refuses at `casting.rs:5931` (`"expected {}..={} target(s) but got {}"`) or `casting.rs:3730`
for Auras (= `OOS-CARDS2-4`). The human is unaffected only because target requirements are
surfaced in the **play-server view layer** (`view.rs:1922`), not the simulator.

**Measured live, from the user's own still-running game** (`GET /api/game/report`, read-only;
1,282 commands, 29 turns, `violations: []`): bots tapped 72 times and cast 17 times.
Grouping consecutive tap runs, **18 of 38 runs are followed immediately by that player's
`PassPriority` with no cast — 26 wasted taps (36%)** — and the engine emitted exactly **18
`ManaPoolsEmptied` events, on exactly those turns**: a 1:1 match with pools destroyed at a
step boundary (CR 500.4). Turn 28, Bot-4, **Upkeep: taps 6 sources, then passes.** A 6-mana
plan at upkeep can only be an auto-tap for an instant-speed cast, and `HeuristicBot` cannot
choose `TapForMana` — so these are necessarily `advance()`'s plan surviving a rejected cast.
That is "bots wasting mana during the beginning of their turns", exactly as reported.

**Honest limit**: the journal records applied commands only, so the rejected command and its
error string are unrecoverable (`local_game.rs:474` throws `e` away). Missing targets is
overwhelmingly likely — it is structural and hits every targeted spell — but some share could
be the greedy-solver `OOS-SIM2-1` or the `OOS-CARDS2-4` Aura family. Settle it by logging `e`
or replaying `seed 0, players 4, heuristic, 2 mulligans` with the error printed.

**Fix**, in value order: (1) route `advance()`'s sequence through the existing
`apply_sequence` (`local_game.rs:700`) as `submit` already does — near-one-line, and it turns
every episode into a harmless no-op; (2) fill `ActionParams.targets` in
`random_bot::action_to_command` (or finally call the dead `Bot::choose_targets`) — without
this bots still cannot cast any targeted spell, they just stop wasting mana trying;
(3) stop swallowing `e` at `:474`; (4) SR-38: suppress cast offers whose targets cannot be
satisfied, which also covers `OOS-CARDS2-4`. Note (1) moves recorded fuzz seeds only where a
cast is rejected — and per `OOS-UI2-1` the fuzzer never casts at all.

### G6 — Deadly Rollick uncastable for free
**KNOWN LIMITATION (R4). Not a defect at any layer.** Notes: *"could not cast for free when
my commander was on the board"*.

The card def is **correct** — `deadly_rollick.rs:19-35` declares
`AbilityDefinition::AltCastAbility { kind: AltCostKind::CommanderFreeCast, .. }` and is
deck-legal `Complete`. The engine **fully supports it** — `casting.rs:170`, and
`casting.rs:2348-2375` implements CR 118.9 including the 2020-04-17 "any commander will do"
ruling and the `is_phased_in()` check; tests at
`crates/engine/tests/mechanics_a_d/domain_and_freecast.rs:671,747,802`. Send
`Command::CastSpell { alt_cost: Some(CommanderFreeCast) }` today and it is accepted.

The break is structural in the provider, exactly where R4 says. `LegalAction::CastSpell`
(`legal_actions.rs:67-73`) has three fields and **no `alt_cost`**; the hand loop gates on the
*printed* cost (`:586-587`), so the card is offered only when you can pay `{3}{B}`; and
`params.rs:267` hardcodes `alt_cost: None`. R4, verbatim (`memory/m11-session-plan.md:1672`):
*"`StubProvider` gaps: no Adventure …, no alt-costs (Spectacle/Surge/Escape/Flashback…), no
modes, no Convoke/Improvise/Delve. A human will hit these."* Still true. (Two sub-claims have
since been partly closed — modes are offered, and Mutate/Morph set `alt_cost` via dedicated
variants — the generic gap stands.)

**Census, so the limitation is sized rather than gestured at**: `AltCastAbility` appears in
**35 defs — 26 deck-legal `Complete`** (19 by the `#[default]` derive). All **four**
`CommanderFreeCast` defs (`deadly_rollick`, `flawless_maneuver`, `deflecting_swat`,
`fierce_guardianship`) are `Complete` and all four are uncastable for free from the browser.
Widening past `AltCastAbility` — Buyback 3, Overload 3, Evoke 3, Bestow 2, Emerge 1, Cleave 1,
Disturb 1, plus keyword-carried Escape 4, Morph 4, Madness 3, Miracle 3, Foretell 2,
Megamorph 2, Disguise 1, Spectacle 1, Surge 1, and 4 `adventure_face` — **~70 defs carry an
alternative or optional cast mode the browser cannot offer.**

**No task proposed.** This is the documented cost of `StubProvider` being a bot move
generator rather than a rules-complete action enumerator, and closing it is a substantial
provider project (an `alt_cost` field or a `CastWithAltCost` sibling variant, plus mirroring
each engine eligibility predicate for SR-38). Worth its own milestone-scale decision, not a
successor batch. **Do not touch `deadly_rollick.rs`.**

### G7 — Targets are absent from the event log (the stack half is refuted)
**Split verdict.** Notes: *"still hard to parse events clearly — Fell Spector: couldnt see
who was target in any event log — targeting should be part of the stack and cards sections"*.

The leading clause — *"still hard to parse events clearly"* — is a standing complaint carried
over from the first playtest, and UI-3 has already shipped against it (the 3-tier event feed).
It is dispositioned here as **partially addressed, no separate task**: the user's own
elaboration names one concrete cause, and that cause is below. If event legibility is still
poor after G7 lands, it needs a fresh observation to act on, not a re-file of this line.

**Stack: REFUTED — already implemented end to end.** `StackItemView.targets: Vec<String>`
(`view-model/src/lib.rs:182-183`) is built at `:524-528` via `format_target` (`:696-714`),
redacted per-target (`redact.rs:241-248`: *"A player target is always public (CR 108.1)"*),
and rendered by `ZoneStack.svelte`'s `<!-- Targets -->` block as `→ player:Alice`. Triggered
abilities do carry their targets onto the stack object (`abilities.rs:9166`, PB-DP8). So a
Fell Specter ETB **does** show its target while it is on the stack — which suggests what the
user actually read was the event feed, and/or a stack that emptied between bot passes before
they could look.

**Event log: NEW DEFECT, and it is an engine gap — the data never existed.**
`events.rs:136-140` `SpellCast { player, stack_object_id, source_object_id }`, `:189-192`
`AbilityActivated`, `:199-203` `AbilityTriggered` — **none carries targets.** The only
target-bearing events are `PermanentTargeted` (`:750-754`), which by its own doc fires for a
**battlefield permanent** and exists to trigger Ward, and `TargetsChanged` (`:1392-1400`),
emitted only by `Effect::ChangeTargets`. **Fell Specter targets a *player*, so it emits
nothing at all** — the event stream genuinely contains zero information about who was hit.
The user's report is exactly accurate.

The view model is not the culprit and should not be blamed: `event_view.rs:112-129` states
*"There is deliberately no payload field: anything the client could need must have been
rendered into `text` by code that consulted the viewer, otherwise it is a path around
Invariant 7."* The three arms (`:758`, `:813`, `:826`) render no target because they have
none to render — `:826-836` produces the literal line the human read, *"A triggered ability
of {n} goes on the stack"*. The frontend is likewise innocent (`EventFeed.svelte:269-272`
renders `text`, and its header says so).

**Invariant 7**: targets are public (CR 601.2c announcement, CR 405.1), with one refinement —
*object* targets are not unconditionally public (a face-down permanent's identity, CR 708.2),
and `redact_stack` already routes them through `viewer_may_identify`. An event-level renderer
must use the existing `card_or` gate (`event_view.rs:172-186`) for object targets and may
print player targets unconditionally.

**Fix**: **engine, and it bumps the wire (SR-8, PROTOCOL currently 33), plus `state/hash.rs`
if the payload is hashed.** Two options — add `targets` to the three variants, or add one
`TargetsAnnounced { stack_object_id, targets }` event at CR 601.2c/603.3d announcement time.
**Cheaper third option worth weighing first**: widen `PermanentTargeted` into a `Targeted`
event covering `Target::Player` — still a wire change, but one variant, and it reuses the
`event_view.rs:664-675` arm that already renders correctly. Downstream is mechanical:
view-model arms; **zero** play-server change; **zero** frontend change (`{ev.text}` picks it
up). The notes' "and cards sections" ask (highlight a permanent currently being targeted) is
separate and **needs no engine change at all** — it is a derived `PermanentView` field
computed from `state.stack_objects()`, buildable today.

---

## UX items (no code verification needed — carried as feature work)

Grouped as the first triage did. Each was scoped to its owning file so the successor task
does not have to rediscover it. **UI-3 (`scutemob-180`) already shipped** the 2×2 board,
sticky seat/action/hand rows, the 3-tier event feed, combat display, "pass until X" and
`TargetPicker` seat segmentation — none of the below is re-work.

### G8 — Concede placement and cancel-vs-concede semantics
*"had to cancel and concede, which ended the game? — i thought the concede button would
concede the choice — this option should be next to new game, not in the priority changing
area"*. Concede is `LegalAction::Concede`, appended **unconditionally** by
`local_game.rs:850-863` (including while a blocking decision stands), pulled into the
controls group at `ActionBar.svelte:195-197`, rendered `:584-596`, submitted as
`onAct(index, {})` → `params.rs:250`. It correctly ends the game; there is **no
confirmation dialog anywhere in the client**.

**A cancel-during-decision handler already exists and is correct** —
`ActionBar.svelte:360-369 cancelChain`, wired to all eight pickers and to Escape (`:468-472`);
it aborts the picker and submits nothing, leaving the decision pending, which is right (a
blocking decision is not cancellable). **Why the user still hit Concede**:
`legal_actions.rs:405-431` early-returns with *only* the answer action when a blocking
decision is outstanding — no `PassPriority` — so the action row was literally
`[answer the search] [Concede]`, and **G1 had made the answer button dead**. Concede was the
only live control on screen. Fix G1 and this becomes an ordinary UX complaint; fix only this
and the dead end remains.
**Fix**: move Concede to the header next to "New game" (`PlayApp.svelte:409-436`) keeping the
same `option.index` submission; add a confirm step; relabel the picker's "Cancel" to "Back"
so it does not read as a peer of Concede. Caveat: Concede is only in the payload when the
human holds a decision, so a header button must render **disabled with a reason** during bot
turns rather than vanish. **Small, ~80-120 lines, 2 files, no server change.**

### G9 — Show the whole library when searching
*"only showed legal basic lands — should be able to view whole library when searching —
current view is too cumbersome — should be a list which you can check"*.

**The filter is correct engine behaviour**, not a defect: candidates are built by
`matches_filter` over the library (`effects/mod.rs:3526-3562`) and *are the answer space* —
`handle_answer_effect_choice` (`:649-656`) refuses anything outside it, so offering non-lands
would be offering illegal answers (an SR-38 violation). **But CR 701.23a is on the user's
side**: *"To search for a card in a zone, look at **all** cards in that zone (even if it's a
hidden zone)."* So this is a rules-*alignment* request, not merely taste.

Two halves of very different size. **(a) Presentation** — turn the wrapped button grid
(`SearchPicker.svelte:157-172`, `max-height: 11rem`) into a scrollable checkable list; ~60
lines, one file, zero server change. **(b) Whole library — BLOCKED on a new channel.** The
view model never enumerates any library, *including the viewer's own*
(`redact.rs:15`, and `lib.rs:90` carries only `library_size`). That is why
`view.rs:1659-1677 question_card_label`/`question_cards` exists as a deliberate narrow
channel, pinned by `test_ui1_view_rs_reads_game_state_in_exactly_the_two_known_places` at
**two** raw `GameState` reads. Recommended route: extend `AnswerShapeView::PickOne`
(`view.rs:380-386`) with a look-only `all_cards`, keeping `candidates` as the selectable
subset — a play-server DTO, so **no PROTOCOL change** (the engine-side alternative would bump
PROTOCOL *and* HASH via `state/hash.rs:3211`). **The gate will go red on purpose and must be
re-pinned to three with a written CR 701.23a entitlement argument, plus a test asserting a
foreign seat never receives it.** That is the gate working, not a nuisance.
**Medium, ~150-250 lines across 2 Rust + 1 Svelte file + an HTTP probe.**

### G10 — "Tap land for mana" clutters the legal-action list
*"should be removed from the list of legal actions — clutters up the list"*. Pure frontend:
the kind is already on the wire (`view.rs:1065`) and `ActionBar.svelte:196` already
partitions by kind, so no server tag is needed.

**Do NOT hide it — that would remove a capability, and the evidence is unambiguous.**
`auto_tap_commands_for` (`local_game.rs:652-655`) begins
`let Command::CastSpell(cast) = command else { return None; }` — **auto-tap covers casts and
nothing else.** Activated-ability costs are paid straight from the pool
(`abilities.rs:831-834`), and `PayEcho`/`PayCumulativeUpkeep`/`PayRecover` are offered only
when the *existing* pool covers them (`legal_actions.rs:468-544`, whose comment at `:454-463`
says outright *"CR 608.2g lets the player activate mana abilities first — so TapForMana must
stay available alongside these"*). Hide it and a human can never activate a mana-cost ability
(including every Equip), never pay echo/cumulative upkeep/recover, and never float mana ahead
of a cost-increase effect. **Proposed instead**: a third collapsed-by-default action group,
"mana sources (N)", optionally one row per land *name* with a count (folds into G13).
**Small, ~60-100 lines, 1 file.**

> **Filed separately, not part of the UX batch**: `ActivateAbility` is offered when
> `can_afford` (`legal_actions.rs:1752-1756`, pool **plus untapped sources**) says it is
> payable, but the human `submit` path never auto-taps for it — so clicking an ability with
> an empty pool 422s. That is the mirror of what SIM-1 fixed for casts, it belongs on the SIM
> track, and it is the underlying reason manual land-tapping still feels mandatory. It also
> compounds G4: for Yahenni you can hit both failures on one click.

### G11 — Hover card name interferes with the card image
The image itself is fine (`cardTooltip.js:60-106`, a shared fixed-position div with an edge
flip). The interfering text is the **browser-native `title=` attribute on the same element**,
drawn by browser/OS chrome above every z-index, at the cursor — exactly where the image is
anchored. **This cannot be fixed with CSS**; the `title` must go or move. Nine conflicting
sites: `ZoneBattlefield.svelte:73/75, 141/143, 172/174, 207/209, 241/243`,
`ZoneHand.svelte:74/76`, `ZoneGraveyard.svelte:30/32`, `ZoneExile.svelte:28/30`,
`SeatCard.svelte:118/120`. **Fix**: give `cardTooltip` a caption parameter rendered inside the
floating div, delete the `title` at each site. **Small, ~80 lines, 6 files.**

### G12 — Artifacts and enchantments should sit above lands
Entirely client-side — the server sends a flat `HashMap<String, Vec<PermanentView>>` with no
ordering semantics (`view-model/src/lib.rs:205`). `ZoneBattlefield.svelte` renders Creatures
`:59`, Lands `:129`, Planeswalkers `:161`, Artifacts/Enchantments `:192`, Other `:230`. Move
one block. **Subtlety to state in the brief**: the classifier (`:23-35`) is first-match and
tests `Land` before `Artifact`, so an artifact land renders in Lands regardless — if it should
ride with the artifacts, the classifier changes too. **Trivial, 1 file.**

### G13 — Same-name lands should stack when tapped
No stacking of any kind exists today (`ZoneBattlefield.svelte:129-158`, keyed per
`object_id`). **Three constraints the brief must carry**: (1) tapped and untapped must **not**
merge — tap state is the information the request is about, so the group key is at minimum
`(name, tapped)`; (2) merge only genuinely fungible permanents — `PermanentView` carries
`counters`, `attached_to`, `is_commander`, `is_token`, `summoning_sick`, `damage_marked`, and
a land with a counter or an aura is not interchangeable, so require every rendered field
identical; (3) **decide the click path explicitly** — `PlayApp.svelte:223-229` matches actions
by a single `object_id`, so a stack must either nominate a representative (first untapped) or
expand on click. Leaving it implicit means clicking a 5-Forest stack is undefined.
**Small/medium, ~80-120 lines.**

---

## Proposed successor tasks

The F1-F10 → `scutemob-174..181` shape: small single-concern batches, each on its own branch,
named by track. Ordering below is by value, and the first two are worth dispatching before
the rest are read.

| # | Task | Findings | Track | Scope | Wire |
|---|---|---|---|---|---|
| 1 | ✅ **UI-4** — picker Confirm hotfix — SHIPPED `scutemob-185` (merge `b031d39e`) | G1 | frontend | **3 lines** + error surfacing + a grep gate | none |
| 2 | ✅ **SIM-4** — mulligan preserves the deck — SHIPPED `scutemob-187` (merge `dcb1fe55`) | G2 (+3 adjacent seeds) | simulator | ~40-60 lines, 0 engine | none |
| 3 | ✅ **SIM-5** — bot cast discipline — SHIPPED `scutemob-188` (merge `e185a2ff`) | G5 | simulator | ~100-150 lines | none |
| 4 | ✅ **SIM-6** — activation-cost payment channel — SHIPPED `scutemob-189` | G4 (+ 8 `exclude_self` defs) | simulator + play-server | ~150 lines, 0 engine | none |
| 5 | **UI-5** — UX polish batch 2 | G8, G10, G11, G12, G13 | frontend | ~400-500 lines, ~10 files | none |
| 6 | **ENG-1** — effect-driven discard decision | G3 | engine | one PB-sized batch | **PROTOCOL + likely HASH** |
| 7 | **ENG-2** — targets in the event log | G7 | engine | one variant + view-model arms | **PROTOCOL** |
| 8 | **UI-6** — whole-library search view | G9 | play-server + frontend | ~150-250 lines | none (DTO only) |
| — | *(no task)* | G6 | — | documented R4; milestone-scale to close | — |

**Dispatch notes, in the order they matter:**

- **UI-4 must go first and must go alone.** It is three lines, it unblocks five CR flows, and
  every other frontend task's manual verification is currently invalid without it — a worker
  testing UI-5 today would find every picker dead and misattribute it. Its first step is to
  *falsify* the diagnosis in a browser console, not to edit. Its real deliverable may be the
  frontend test harness (plan §8 R7, deferred at M11 close and now visibly overdue).
- **ENG-1 and ENG-2 both bump PROTOCOL. Consider merging them into one engine batch** — the
  sentinel re-pin and the full `--workspace --no-fail-fast` confirmation run are paid once
  instead of twice, and PB-DX5/DX6 have already shown that re-pin is the expensive part. The
  counter-argument is that they are unrelated subsystems and a merged batch is harder to
  review; the coordinator should decide, not the worker. **Either way: compute PROTOCOL and
  HASH from the failing gate's own output. Never predict them** — CARDS-1 found the
  criterion's "PROTOCOL 32" already stale.
- **SIM-5 and SIM-6 both touch `legal_actions.rs` and should not run in parallel.** The
  2026-08-02 collect's lesson stands: parallel workers sharing a crate produce semantic
  conflicts that survive a clean textual merge.
- **SIM-4 needs the missing gate**, not just the fix: a test asserting deck and commander are
  **unchanged** across a redeal. `test_redeal_produces_a_different_hand` passing is why this
  shipped.
- **G4's brief must include the Yahenni `exclude_self` one-liner.** Fixing the channel without
  it converts a dead ability into a wrong one.
- **UI-5's brief must forbid hiding `TapForMana`** (see G10) and must resolve the shared-
  component question **once, up front**: G11, G12 and G13 all land in the replay viewer's
  `$viewer` components, so the worker chooses edit-in-place (UI-3's `CombatView` precedent —
  both surfaces benefit) or a play-local fork (`PlayBoard.svelte`'s precedent — when the two
  surfaces want opposite things). Three separate answers inside one file is the failure mode.
- **UI-6 is split out of UI-5 deliberately.** It moves an Architecture-Invariant-7 gate, and
  MR-M11-01's lesson is that a gate movement must not ride along inside a UX batch.

## Seeds to file (none filed here — this branch may touch one file)

`OOS-G1-1` picker `structuredClone` class (any structured-clone/postMessage/IndexedDB call on
Svelte state) · `OOS-G2-1` mulligan can fail `validate_deck` · `OOS-G2-2` `GameResult.seed`
does not reproduce a mulliganed game · `OOS-G2-3` `Command::TakeMulligan`/`KeepHand` /
`DecisionKind::Mulligan` are workspace-wide dead code (turn-0 states are never built) ·
`OOS-G3-1` effect-driven discard has no decision point · `OOS-G3-2` the "engine picks for the
player" census, `SacrificePermanents` first · `OOS-G4-1` activation-cost payment channel (53
`Cost::Sacrifice` + 9 `Cost::DiscardCard` defs; TUI `input.rs:287` too) · `OOS-G4-2` Yahenni
`exclude_self` · `OOS-G5-1` bot command sequences are non-atomic · `OOS-G5-2` bots announce
zero targets / `Bot::choose_targets` is dead · `OOS-G5-3` `local_game.rs:474` swallows the
rejection error · `OOS-G6-1` ~70 defs carry an unofferable alternative cast mode ·
`OOS-G7-1` no `GameEvent` carries targets · `OOS-G10-1` `ActivateAbility` is offered but the
human path never auto-taps for it.

## Method and limits

Seven parallel read-only chain walks, each required to cite `file:line` it had actually read
rather than repeat the dispatch brief's line numbers — which caught two brief errors (G2's
CR 103.4/103.5 mix-up, G4's "StubProvider doesn't offer activated abilities"). G1's
`DataCloneError` was reproduced independently at triage time in node, and its three call
sites re-grepped, before being written up. G5's numbers come from `GET /api/game/report` on
the user's own still-running server, read-only — not from a rebuild.

**What was not done**: nothing was built or tested (`cargo` was not run; this worktree has no
`target/`), no fix was implemented, and no file outside this one was modified. G1 is a
source-plus-reproduction diagnosis, not an observed browser trace. G5's attribution of the
rejected casts to missing targets is strongly but not exhaustively evidenced — the error
string is discarded before it reaches the journal.
