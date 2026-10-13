# The C3 blocks of the v3 interaction probe (scutemob-268): the client side of the five server fields
# C3 added, each on a REAL game state reached by seed through `v3scenario` (CHOSEN `c3_*`, plus C2's
# auto-pass scenarios). Not run by itself — `v3interact.py` owns the browser, the `Table` class and the
# ok / BAD / COULD-NOT-RUN counters, and calls `run()` here (`--c3-only`, `--c3-blocks`, and the default run).
#   exile       per-player exile: every seat's EX pile (2x2 box, 1x5 strip, the board across) shows that
#               seat's own `exile_by_owner` count and top card; the popover and the seat menu list that
#               seat's cards only; no shared `EX*` label. FABRICATED: `exile_by_owner` deleted from the
#               server's answer in flight -> the old shared `EX*` pile comes back (old-server fallback).
#   tax         commander tax: the command pile's badge is `+2n` for every commander with n > 0 and absent
#               for 0, on every container of every seat; dimmer when that commander is not in the command
#               zone; the popover says what the next cast costs; a commander in the command zone has its star.
#   gameid      the order key is `summary.game_id`; the same seed re-dealt through the BARE API (the C2 gap)
#               does not inherit the order; old orders are pruned. FABRICATED: `game_id` stripped in flight
#               -> the C2 heuristic key still works.
#   targets     the red `targeted` outline is on exactly the cards whose ids are in the stack's
#               `target_refs`; a targeted player's seat is marked; a targeted spell / ability on the stack
#               is marked; hovering a stack item draws one red arrow per target that is on screen and none
#               for a withheld id (`id: null` — a REAL one, seed 3, not fabricated).
#   stackpick   a stack item as a target: while the target stage of a real counter-style spell is open the
#               candidate stack items are lit; a click on one presses the picker's own button for THAT
#               candidate (two candidates carry the same label here) and the picker's Confirm posts its
#               value; an arrow dragged from the action onto the other one posts the other value
#   serverpass  auto-pass through the server parameter: ONE `POST /api/game/action` carries `pass_until`
#               for a whole run, its body is the panel's settings, and each outcome maps to the panel state
#               C2 defined (until -> Off; unless -> paused, the server's reason shown, Resume sends the
#               anchor; a real decision -> handed back). Off and the boxes are locked while the request is
#               in flight. FABRICATED: a 400 `invalid_body` to any request carrying `pass_until` -> the
#               client loop takes over (one request per pass) and Off cancels it. A reload never passes.
import asyncio, json, time

import v3scenario as scn

C3_BLOCKS = ("exile", "tax", "gameid", "targets", "stackpick", "serverpass")
AP = ".v3 .autopass"

# Every seat container on the table (the peeked copy excluded) with its piles as drawn.
JS_PILES = """() => [...document.querySelectorAll('.v3 [data-seat]')].filter((e) => !e.classList.contains('cell-pop')).map((e) => ({
  seat: e.dataset.seat, where: e.className.split(' ')[0],
  piles: Object.fromEntries([...e.querySelectorAll('.pile[data-pile]')].map((p) => [p.dataset.pile, {
    lbl: (p.querySelector('.lbl')?.textContent ?? '').trim(),
    top: p.querySelector(':scope > .card')?.dataset.oid ?? null,
    back: !!p.querySelector(':scope > .card.back'), empty: !!p.querySelector(':scope > .empty'),
    star: p.querySelectorAll(':scope > .card .tag.cmdr').length,
    tax: [...p.querySelectorAll('.tax')].map((t) => ({ text: t.textContent.trim(), away: t.classList.contains('away'), commander: t.dataset.commander,
      w: Math.round(t.getBoundingClientRect().width), px: parseFloat(getComputedStyle(t).fontSize),
      inside: t.getBoundingClientRect().right <= p.getBoundingClientRect().right + 1 && t.getBoundingClientRect().left >= p.getBoundingClientRect().left - 1 })),
  }])),
}))"""

# What the stack's targets look like on the page.
JS_TARGETED = """() => {
  const off = (e) => !!e.closest('.decision-host');
  const cards = [...document.querySelectorAll('.v3 .card[data-oid]')];
  return {
    marked: [...new Set(cards.filter((e) => e.classList.contains('targeted') && !off(e)).map((e) => Number(e.dataset.oid)))].sort((a, b) => a - b),
    markedOff: cards.filter((e) => e.classList.contains('targeted') && off(e)).length,
    unmarked: [...new Set(cards.filter((e) => !e.classList.contains('targeted') && !off(e)).map((e) => Number(e.dataset.oid)))],
    drawn: [...new Set(cards.filter((e) => !off(e)).map((e) => Number(e.dataset.oid)))],
    seats: [...new Set([...document.querySelectorAll('.v3 [data-seat].seat-targeted')].map((e) => e.dataset.seat))].sort(),
    seatBoxes: [...document.querySelectorAll('.v3 [data-seat]')].filter((e) => !e.classList.contains('cell-pop')).map((e) => [e.dataset.seat, e.classList.contains('seat-targeted')]),
    ring: [...document.querySelectorAll('.v3 [data-seat].seat-targeted .seat-name')].map((e) => getComputedStyle(e).outlineStyle + ' ' + getComputedStyle(e).outlineColor),
    items: [...document.querySelectorAll('.v3 .stack-box .stack-item')].map((e) => ({ id: Number(e.dataset.stackId), src: e.dataset.sourceOid === undefined ? null : Number(e.dataset.sourceOid), targeted: e.classList.contains('targeted') })),
  };
}"""

JS_AIMED = """() => {
  const svg = document.querySelector('.v3 svg.v3-arrows');
  return { arrows: [...document.querySelectorAll('.v3 svg.v3-arrows path.arrow.aimed')].map((p) => ({
             from: Number(p.dataset.fromStack), oid: p.dataset.toOid === undefined ? null : Number(p.dataset.toOid),
             seat: p.dataset.toSeat ?? null, stack: p.dataset.toStack === undefined ? null : Number(p.dataset.toStack),
             stroke: getComputedStyle(p).stroke, len: Math.round(p.getTotalLength()) })),
           amber: document.querySelectorAll('.v3 svg.v3-arrows path.arrow.live, .v3 svg.v3-arrows path.arrow.chosen').length,
           pe: svg ? getComputedStyle(svg).pointerEvents : null };
}"""

JS_PANEL = """() => {
  const q = (s) => document.querySelector('.v3 .autopass ' + s);
  const stop = [...document.querySelectorAll('.v3 .decision-host .btn.stop')];
  return { on: [...document.querySelectorAll('.v3 .autopass .mode.on')].map((e) => e.dataset.mode),
           dis: Object.fromEntries([...document.querySelectorAll('.v3 .autopass .mode[data-mode]')].map((e) => [e.dataset.mode, e.disabled])),
           boxes: [...document.querySelectorAll('.v3 .autopass input[data-stop]')].map((e) => e.disabled), step: q('select.step')?.disabled ?? null,
           status: (q('.status')?.innerText ?? '').replace(/\\n/g, ' ').trim(), resume: q('.resume') ? (q('.resume').disabled ? 'disabled' : 'enabled') : null,
           serverClass: !!document.querySelector('.v3.server-passing'), panelClass: !!document.querySelector('.v3 .autopass.server-run'),
           stopShown: stop.filter((e) => getComputedStyle(e).display !== 'none').length, stopButtons: stop.length };
}"""


def refs_of(v):
    return [r for _, r in scn.stack_refs(v)]


def spell_of_object(v):
    """{object id: stack item id} — the stack item an OBJECT ref to that id means: the one non-copy SPELL whose
    card it is. An ability is not its source (it is targeted by `stack_object` refs only); a copy shares its
    original's source id and has no card; two claimants = ambiguous = none. Stated here independently of the page."""
    claims = {}
    for i in v["state"]["zones"]["stack"]:
        if i["kind"] == "spell" and not i.get("is_copy") and i.get("source_object_id") is not None:
            claims.setdefault(i["source_object_id"], []).append(i["id"])
    return {src: ids[0] for src, ids in claims.items() if len(ids) == 1}


def all_ids(v):
    """Every object id this seat's view places somewhere (battlefields, hands, graveyards, exile, command zones)."""
    z = v["state"]["zones"]
    out = set()
    for zone in ("battlefield", "hand", "graveyard", "command_zone"):
        for cards in z[zone].values():
            out |= {c["object_id"] for c in cards if c.get("object_id") is not None and not c.get("hidden")}
    return out


def pass_index(v):
    return next(x["index"] for x in scn.actions(v) if x["kind"] == "PassPriority")


async def run(I, browser, a, base, w, h):
    """The C3 section. `I` is the running `v3interact` module (its Table, check, could_not, counters)."""
    check, could_not, Table, NotReached, Missing, CELL = I.check, I.could_not, I.Table, I.NotReached, I.Missing, I.CELL
    print("---- C3 (scutemob-268): the new seat-view fields and the server-side pass-until", flush=True)

    # ------------------------------------------------------------------ exile
    async def blk_exile(T):
        pg = T.pg
        r = await T.at("c3_exile")
        v = r.view
        piles = scn.exile_piles(v)
        seats = list(v["state"]["players"])
        me = scn.human(v)

        async def boxes():
            dom = await pg.evaluate(JS_PILES)
            rows, bad, stale = 0, [], []
            for box in dom:
                ex = {k: p for k, p in box["piles"].items() if k.startswith("EX")}
                if not ex:
                    bad.append(f"{box['where']}[{box['seat']}] has no exile pile")
                    continue
                key, p = next(iter(ex.items()))
                mine = piles.get(box["seat"], [])
                want_lbl = f"EX {len(mine)}"
                top = mine[0] if mine else None
                want_top = None if not top else (None if top.get("hidden") else str(top["object_id"]))
                rows += 1
                if key != "EX" or "*" in p["lbl"]:
                    stale.append(f"{box['where']}[{box['seat']}] {key!r} {p['lbl']!r}")
                if p["lbl"] != want_lbl or p["top"] != want_top or p["empty"] != (not mine):
                    bad.append(f"{box['where']}[{box['seat']}] shows {p['lbl']!r} top {p['top']} empty {p['empty']}, want {want_lbl!r} top {want_top}")
            counts = {s: len(piles.get(s, [])) for s in seats}
            check("exile: every seat's pile (2x2 box, 1x5 strip, the board across) shows that seat's own exile count and top card",
                  rows >= len(seats) + 1 and not bad and set(piles) == set(seats),
                  f"{rows} piles over {len({b['seat'] for b in dom})} seats, exile_by_owner counts {counts}; wrong: {bad[:4]}")
            check("exile: no pile carries the shared `EX*` label", rows > 0 and not stale, f"{rows} piles; stale {stale[:4]}")
            shared = len(v["state"]["zones"]["exile"])
            check("exile: the scenario tells per-player from shared (two owners, the shared list is longer than either)",
                  shared == sum(counts.values()) and sorted(counts.values())[-1] < shared and sum(1 for n in counts.values() if n) >= 2, f"shared exile {shared}, per seat {counts}")
        await T.part("exile: the piles", boxes)

        async def popovers():
            owner = next(s for s in seats if s != me and piles.get(s))
            empty = next(s for s in seats if not piles.get(s))
            for seat, sel in ((owner, f'{CELL}[data-seat="{owner}"] .pile[data-pile="EX"]'), (me, f'.v3 .board[data-seat="{me}"] .zonebox .pile[data-pile="EX"]'),
                              (empty, f'{CELL}[data-seat="{empty}"] .pile[data-pile="EX"]')):
                await T.click(await T.point(sel))
                title = await T.text(".v3 .popover .panel-title")
                got = sorted(await pg.evaluate("() => [...document.querySelectorAll('.v3 .popover .card[data-oid]')].map((e) => Number(e.dataset.oid))"))
                backs = await T.n(".v3 .popover .card.back")
                want = sorted(c["object_id"] for c in piles.get(seat, []) if not c.get("hidden"))
                hidden = sum(1 for c in piles.get(seat, []) if c.get("hidden"))
                check(f"exile: the popover of {'your' if seat == me else 'an opponent' if piles.get(seat) else 'an empty'} pile lists that seat's exiled cards only",
                      got == want and backs == hidden and f"{seat}'s exile".lower() in title.lower() and "shared" not in title.lower(),
                      f"{seat}: {[c['name'] for c in piles.get(seat, [])]} -> popover oids {got} (+{backs} backs), want {want}; title {title!r}")
                await pg.keyboard.press("Escape")
                await pg.wait_for_timeout(150)
            await T.shot("c3-exile")
        await T.part("exile: the popovers", popovers)

        async def menu():
            owner = next(s for s in seats if s != me and piles.get(s))
            await T.click(await T.corner(f'{CELL}[data-seat="{owner}"]'), button="right")
            items = await pg.locator(".v3 .ctx.seat-ctx button").all_inner_texts()
            item = next((t for t in items if "exile" in t.lower()), "")
            await pg.locator(".v3 .ctx.seat-ctx button", has_text="exile").click(timeout=2000)
            await pg.wait_for_timeout(150)
            got = sorted(await pg.evaluate("() => [...document.querySelectorAll('.v3 .popover .card[data-oid]')].map((e) => Number(e.dataset.oid))"))
            check("exile: the seat menu's \"View exile\" counts and lists that seat's own cards",
                  f"({len(piles[owner])})" in item and got == sorted(c["object_id"] for c in piles[owner]), f"{owner}: menu item {item!r}, popover {got}")
            await pg.keyboard.press("Escape")
            await pg.wait_for_timeout(150)
        await T.part("exile: the seat menu", menu)

        async def old_server():
            def strip(view):
                view["state"]["zones"].pop("exile_by_owner", None)
            T.mut = strip
            try:
                await T.load()
                dom = await pg.evaluate(JS_PILES)
                lbls = sorted({p["lbl"] for box in dom for k, p in box["piles"].items() if k.startswith("EX")})
                shared = len(v["state"]["zones"]["exile"])
                check("exile: FABRICATED old server (`exile_by_owner` deleted from the real answer in flight) -> the shared `EX*` pile, labelled as shared",
                      lbls == [f"EX* {shared}"], f"labels {lbls}, shared list {shared}")
            finally:
                T.mut = None
                await T.load()
        await T.part("exile: without the field", old_server)

    # ------------------------------------------------------------------ commander tax
    async def blk_tax(T):
        pg = T.pg

        async def badges(tag, v):
            me = scn.human(v)
            tx = scn.taxes(v)
            dom = await pg.evaluate(JS_PILES)
            rows, bad, tiny = 0, [], []
            for box in dom:
                p = box["piles"].get("CMD")
                if p is None:
                    bad.append(f"{box['where']}[{box['seat']}] has no command pile")
                    continue
                rows += 1
                want = sorted((f"+{2 * n}", c, not scn.in_command_zone(v, box["seat"], c)) for c, n in tx.get(box["seat"], {}).items() if n > 0)
                got = sorted((t["text"], t["commander"], t["away"]) for t in p["tax"])
                if got != want:
                    bad.append(f"{box['where']}[{box['seat']}] {got}, want {want}")
                tiny += [f"{box['where']}[{box['seat']}] {t['text']} {t['px']}px inside {t['inside']}" for t in p["tax"] if t["px"] < 8.5 or not t["inside"] or t["w"] < 8]
            taxed = {s: {c: n for c, n in x.items() if n} for s, x in tx.items() if any(x.values())}
            zero = [s for s, x in tx.items() if not any(x.values())]
            check(f"tax ({tag}): the command pile's badge is +2n for every commander with n > 0 and absent for 0, on every container of every seat",
                  rows >= len(tx) + 1 and not bad and bool(taxed), f"{rows} command piles; commander_tax {tx}; wrong: {bad[:4]}")
            check(f"tax ({tag}): a badge is readable and stays inside its pile", not tiny, f"{sum(len(b['piles'].get('CMD', {}).get('tax', [])) for b in dom)} badges; {tiny[:4]}")
            return me, tx, taxed, zero, dom

        async def recast():
            r = await T.at("c3_tax_recast")
            v = r.view
            me, tx, taxed, zero, dom = await badges("cast twice", v)
            (commander, n), = [(c, k) for c, k in tx[me].items() if k >= 2][:1]
            mine = [t for b in dom if b["seat"] == me for t in b["piles"]["CMD"]["tax"]]
            away = not scn.in_command_zone(v, me, commander)
            check("tax: the human's own commander, cast twice, shows +4 on your command pile", [t["text"] for t in mine] == [f"+{2 * n}"] and n == 2,
                  f"{commander}: cast count {n}, badge {[t['text'] for t in mine]}, in the command zone {not away}")
            check("tax: a commander that is not in the command zone keeps its badge, drawn quieter, on the empty slot", away and all(t["away"] for t in mine)
                  and all(b["piles"]["CMD"]["empty"] for b in dom if b["seat"] == me),
                  f"{commander} in zone {not away}; away class {[t['away'] for t in mine]}; slot empty {[b['piles']['CMD']['empty'] for b in dom if b['seat'] == me]}")
            check("tax: a seat whose commanders were never cast has no badge", bool(zero) and all(not b["piles"]["CMD"]["tax"] for b in dom if b["seat"] in zero), f"untaxed seats {zero}")
            await T.click(await T.point(f'.v3 .board[data-seat="{me}"] .zonebox .pile[data-pile="CMD"]'))
            title = await T.text(".v3 .popover .panel-title")
            check("tax: the command pile's popover says what the next cast costs", commander.lower() in title.lower() and f"costs {2 * n} more" in title.lower(), repr(title[:200]))
            await T.shot("c3-tax")
            await pg.keyboard.press("Escape")
            await pg.wait_for_timeout(150)
        await T.part("tax: cast twice", recast)

        async def in_zone():
            r = await T.at("c3_tax_in_zone")
            v = r.view
            me, tx, taxed, zero, dom = await badges("back in the command zone", v)
            commander = next(c for c, n in tx[me].items() if n >= 1 and scn.in_command_zone(v, me, c))
            mine = [b["piles"]["CMD"] for b in dom if b["seat"] == me]
            oid = next(c["object_id"] for c in v["state"]["zones"]["command_zone"][me] if c["name"] == commander)
            check("tax: a taxed commander in the command zone shows +2n on the card itself (not the quieter form)",
                  bool(mine) and all([t["text"] for t in p["tax"]] == [f"+{2 * tx[me][commander]}"] and not p["tax"][0]["away"] and p["top"] == str(oid) for p in mine),
                  f"{commander} (oid {oid}), cast count {tx[me][commander]}: {[(p['top'], p['tax']) for p in mine][:1]}")
            check("tax: a commander in the command zone carries the commander star (C1 LOW)", bool(mine) and all(p["star"] == 1 for p in mine), f"stars on your command pile(s): {[p['star'] for p in mine]}")
        await T.part("tax: in the command zone", in_zone)

    # ------------------------------------------------------------------ game_id as the order key
    async def blk_gameid(T):
        pg = T.pg
        HAND = ".v3 .hand .card[data-oid]"
        KEYS = """() => { const out = { game: null, orders: [] };
          try { out.game = JSON.parse(localStorage.getItem('v3-game') || 'null'); } catch (e) {}
          for (let i = 0; i < localStorage.length; i++) { const k = localStorage.key(i); if (k && k.startsWith('v3-order:')) out.orders.push(k); }
          return out; }"""

        async def hand_dom():
            return await pg.evaluate(I.JS_OIDS, HAND)

        async def key_and_redeal():
            r = await T.at("rearrange_hand")
            gid = r.view["summary"].get("game_id")
            old = [c["object_id"] for c in scn.my_hand(r.view)]
            # twelve stale orders from "earlier games", older than anything this page will save
            await pg.evaluate("() => { for (let i = 0; i < 12; i++) localStorage.setItem('v3-order:stale-' + i, JSON.stringify({ hand: [1, 2, 3], at: 1000 + i })); }")
            await T.drag(await T.point(HAND, nth=0), await T.point(HAND, fx=0.8, nth=2))
            await T.idle(400)
            moved = await hand_dom()
            k = await pg.evaluate(KEYS)
            check("gameid: the order is stored under the server's `summary.game_id`",
                  bool(gid) and (k["game"] or {}).get("key") == f"id:{gid}" and f"v3-order:id:{gid}" in k["orders"] and moved != old[:len(moved)],
                  f"game_id {gid!r}; v3-game {k['game']}; order keys {[x for x in k['orders'] if 'stale' not in x]}")
            check("gameid: old games' orders are pruned (twelve stale keys planted, at most eight games kept)",
                  len(k["orders"]) <= 8 and f"v3-order:id:{gid}" in k["orders"], f"{len(k['orders'])} `v3-order:*` keys left: {sorted(k['orders'])}")
            await T.load()
            check("gameid: the order survives a reload of the same game", (await hand_dom()) == moved, f"{(await hand_dom())[:5]} / {moved[:5]}")

            # the C2 gap: the SAME seed dealt again over the bare API (not this page's deal button) — same
            # players, same counters, the same object ids — is a different game and must not inherit the order
            r2 = scn.reach(T.port, "rearrange_hand")
            if not r2.ok or r2.why:
                raise NotReached(f"re-deal over the API: {r2.why}")
            gid2 = r2.view["summary"].get("game_id")
            want = [c["object_id"] for c in scn.my_hand(r2.view)]
            await T.load()
            dom = await hand_dom()
            k2 = await pg.evaluate(KEYS)
            check("gameid: a redeal changes `game_id` (same seed, dealt over the bare API)", bool(gid2) and gid2 != gid, f"{gid!r} -> {gid2!r}")
            check("gameid: the game re-dealt through the bare API does NOT inherit the order (the documented C2 gap)",
                  want == old and dom == want[:len(dom)] and len(dom) >= 3 and dom != moved[:len(dom)] and (k2["game"] or {}).get("key") == f"id:{gid2}",
                  f"same object ids {want == old}; rearranged {moved[:5]}; the new game shows {dom[:5]}, seat view {want[:5]}; key {(k2['game'] or {}).get('key')!r}")
        await T.part("gameid: key, prune, bare-API redeal", key_and_redeal)

        async def no_id():
            def strip(view):
                (view.get("summary") or {}).pop("game_id", None)
            T.mut = strip
            try:
                r = await T.at("rearrange_hand")
                old = [c["object_id"] for c in scn.my_hand(r.view)]
                await T.drag(await T.point(HAND, nth=0), await T.point(HAND, fx=0.8, nth=2))
                await T.idle(400)
                moved = await hand_dom()
                k = await pg.evaluate(KEYS)
                await T.load()
                again = await hand_dom()
                key = (k["game"] or {}).get("key") or ""
                check("gameid: FABRICATED old server (`game_id` stripped from the real answer in flight) -> the C2 client key still keeps the order over a reload",
                      bool(key) and not key.startswith("id:") and moved != old[:len(moved)] and again == moved, f"key {key!r}; {old[:4]} -> {moved[:4]}, after reload {again[:4]}")
            finally:
                T.mut = None
        await T.part("gameid: without the field", no_id)

    # ------------------------------------------------------------------ targeted + stack arrows
    async def blk_targets(T):
        pg = T.pg
        ITEM = lambda sid: f'.v3 .stack-box .stack-item[data-stack-id="{sid}"]'

        async def marks(tag, v):
            refs = refs_of(v)
            obj = sorted({r["id"] for r in refs if r["kind"] == "object" and r.get("id") is not None})
            players = sorted({r["name"] for r in refs if r["kind"] == "player"})
            stack_ids = {r["id"] for r in refs if r["kind"] == "stack_object"}
            d = await pg.evaluate(JS_TARGETED)
            drawn = set(d["drawn"])
            want = sorted(set(obj) & drawn)
            check(f"targets ({tag}): the red outline is on exactly the drawn cards whose ids are in `target_refs`, on every copy, and on no other",
                  d["marked"] == want and not (set(d["marked"]) & set(d["unmarked"])) and d["markedOff"] == 0 and len(drawn) > 3,
                  f"object refs {obj} (drawn: {want}); outlined {d['marked']}; {len(drawn)} cards on the page; copies of a target left unmarked {sorted(set(d['marked']) & set(d['unmarked']))}; decision-panel copies marked {d['markedOff']}")
            want_seats = [(s, s in players) for s, _ in d["seatBoxes"]]
            check(f"targets ({tag}): a targeted player's seat is marked — every container of that seat, and no other seat",
                  d["seats"] == players and [list(x) for x in want_seats] == d["seatBoxes"] and (not players or all("solid" in x for x in d["ring"])),
                  f"player refs {players}; marked {d['seats']}; containers {d['seatBoxes']}; ring {d['ring'][:2]}")
            spells = spell_of_object(v)
            want_items = sorted(i["id"] for i in v["state"]["zones"]["stack"] if i["id"] in stack_ids or (spells.get(i.get("source_object_id")) == i["id"] and i["source_object_id"] in obj))
            got_items = sorted(i["id"] for i in d["items"] if i["targeted"])
            check(f"targets ({tag}): a stack item is marked exactly when a `stack_object` ref names its stack id, or it is a SPELL and an object ref names its card (never an ability whose source is targeted)",
                  got_items == want_items and len(d["items"]) == len(v["state"]["zones"]["stack"]), f"stack {[(i['id'], i['source_object_id'], i['kind']) for i in v['state']['zones']['stack']]}; marked {got_items}, want {want_items}")
            return obj, players, drawn

        async def hover(tag, v, drawn):
            """One arrow per ref that has something on screen to point at; none for a withheld id."""
            src_items = spell_of_object(v)      # an object ref can only mean the SPELL that object is, never an ability of it
            sids = {i["id"] for i in v["state"]["zones"]["stack"]}
            total, withheld, bad, colours = 0, 0, [], set()
            for item in v["state"]["zones"]["stack"]:
                want = []
                for r in item.get("target_refs") or []:
                    if r["kind"] == "player":
                        want.append(("seat", r["name"]))
                    elif r["kind"] == "object" and r.get("id") is None:
                        withheld += 1
                    elif r["kind"] == "object" and (r["id"] in drawn or (r["id"] in src_items and src_items[r["id"]] != item["id"])):
                        want.append(("oid", r["id"]))
                    elif r["kind"] == "stack_object" and r["id"] in sids and r["id"] != item["id"]:
                        want.append(("stack", r["id"]))
                await pg.mouse.move(*(await T.point(ITEM(item["id"]))))
                await pg.wait_for_timeout(220)
                s = await pg.evaluate(JS_AIMED)
                got = [("seat", x["seat"]) if x["seat"] is not None else ("stack", x["stack"]) if x["stack"] is not None else ("oid", x["oid"]) for x in s["arrows"]]
                total += len(got)
                colours |= {x["stroke"] for x in s["arrows"]}
                if sorted(map(str, got)) != sorted(map(str, want)) or any(x["from"] != item["id"] or x["len"] < 20 for x in s["arrows"]) or s["amber"] or s["pe"] != "none":
                    bad.append(f"{item['source_name']} (stack id {item['id']}, refs {item.get('target_refs')}): arrows {s['arrows']}, want {want}")
                if item is v["state"]["zones"]["stack"][-1]:
                    await T.shot(f"c3-aimed-{tag.split()[0]}")
            await pg.mouse.move(5, 5)
            await pg.wait_for_timeout(220)
            gone = await pg.evaluate(JS_AIMED)
            check(f"targets ({tag}): hovering a stack item draws exactly one arrow per target on screen, from that item, and none for a withheld id",
                  not bad and total > 0, f"{total} arrow(s) over {len(v['state']['zones']['stack'])} stack item(s), {withheld} withheld ref(s) drew nothing; wrong: {bad[:2]}")
            check(f"targets ({tag}): those arrows are red (not the amber \"choosing\" arrow), let the pointer through, and go when the pointer leaves",
                  colours == {"rgb(255, 85, 85)"} and gone["arrows"] == [] and gone["pe"] == "none", f"stroke {sorted(colours)}; after leaving {len(gone['arrows'])} arrow(s); overlay pointer-events {gone['pe']}")
            return withheld

        async def permanent():
            r = await T.at("c3_target_permanent")
            obj, players, drawn = await marks("a permanent", r.view)
            check("targets: the scenario's target is a permanent drawn on the table", bool(obj) and set(obj) <= drawn, f"{[(i['source_name'], i['targets']) for i in r.view['state']['zones']['stack']]}")
            await hover("a permanent", r.view, drawn)
            item = r.view["state"]["zones"]["stack"][-1]
            await pg.locator(ITEM(item["id"])).focus(timeout=2000)
            await pg.wait_for_timeout(220)
            s = await pg.evaluate(JS_AIMED)
            check("targets: focusing a stack item (keyboard) draws its arrows too", len(s["arrows"]) == len([x for x in item["target_refs"] if x.get("id") in drawn]) > 0, f"{len(s['arrows'])} arrow(s) for {item['source_name']}")
            await pg.evaluate("() => document.activeElement?.blur()")
        await T.part("targets: a permanent", permanent)

        async def spell_and_player():
            r = await T.at("c3_target_spell_and_player")
            obj, players, drawn = await marks("a spell on the stack + a player", r.view)
            check("targets: the scenario has a ref to a spell on the stack and a ref to a player", bool(players) and any(o not in drawn for o in obj),
                  f"{[(i['id'], i['source_name'], i.get('source_object_id'), i['target_refs']) for i in r.view['state']['zones']['stack']]}")
            await hover("a spell on the stack + a player", r.view, drawn)
        await T.part("targets: a spell and a player", spell_and_player)

        async def withheld():
            r = await T.at("c3_target_withheld")
            obj, players, drawn = await marks("a withheld id", r.view)
            n = await hover("a withheld id", r.view, drawn)
            d = await pg.evaluate(JS_TARGETED)
            check("targets: a REAL withheld id (`id: null`, seed 3 — not fabricated) outlines no card at all",
                  n >= 1 and obj == [] and d["marked"] == [], f"{[(i['source_name'], i['targets'], i['target_refs']) for i in r.view['state']['zones']['stack']]}; outlined {d['marked']}")
        await T.part("targets: a withheld id", withheld)

        async def stack_object():
            r = await T.at("c3_target_stack_object")
            obj, players, drawn = await marks("a stack object", r.view)
            await hover("a stack object", r.view, drawn)
        await T.part("targets: a stack object", stack_object)

        async def ability_source():
            # Review fix: an object ref whose id is an ABILITY's source names the permanent, not the ability.
            r = await T.at("c3_ability_source_targeted")
            v = r.view
            obj, players, drawn = await marks("an ability whose source is targeted", v)
            await hover("an ability whose source is targeted", v, drawn)
            stack_ids = {x["id"] for _, x in scn.stack_refs(v) if x["kind"] == "stack_object"}
            pairs = [(a_, b) for a_, b in scn.ability_source_targets(v) if a_["id"] not in stack_ids]
            if not pairs:
                raise NotReached("every such ability is also named by a `stack_object` ref here; nothing to tell apart")
            abil, by = pairs[0]
            src = abil["source_object_id"]
            d = await pg.evaluate(JS_TARGETED)
            item = next((i for i in d["items"] if i["id"] == abil["id"]), None)
            check("targets: REAL state — an ability on the stack is NOT outlined when another stack item targets its source permanent; the permanent on the battlefield is",
                  item is not None and item["targeted"] is False and src in drawn and src in d["marked"],
                  f"{abil['source_name']}'s {abil['kind']} (stack id {abil['id']}, source oid {src}) targeted-class {item and item['targeted']}; "
                  f"{by['source_name']} (stack id {by['id']}) refs {by.get('target_refs')}; source drawn {src in drawn}, outlined {src in d['marked']}")
            await pg.mouse.move(*(await T.point(ITEM(by["id"]))))
            await pg.wait_for_timeout(220)
            s = await pg.evaluate(JS_AIMED)
            to_perm = [x for x in s["arrows"] if x["oid"] == src]
            end = await pg.evaluate("""(sel) => { const p = document.querySelector(sel); if (!p) return null; const e = p.getPointAtLength(p.getTotalLength());
                const hit = (q) => [...document.querySelectorAll(q)].some((el) => { const r = el.getBoundingClientRect(); return e.x >= r.left && e.x <= r.right && e.y >= r.top && e.y <= r.bottom; });
                return { card: hit('.v3 .card[data-oid="' + p.dataset.toOid + '"]'), stack: hit('.v3 .stack-box .stack-item') }; }""",
                                    f'.v3 svg.v3-arrows path.arrow.aimed[data-to-oid="{src}"]')
            check("targets: REAL state — the arrow for that ref ends on the source permanent's card, not on the ability's stack item",
                  len(to_perm) == 1 and end == {"card": True, "stack": False}, f"arrows from stack id {by['id']}: {s['arrows']}; the arrow's end lies on {end}")
            await T.shot("c3-ability-source")
            await pg.mouse.move(5, 5)
        await T.part("targets: an ability whose source is targeted", ability_source)

    # ------------------------------------------------------------------ a stack item as a click target
    async def blk_stackpick(T):
        pg = T.pg
        ITEM = lambda sid: f'.v3 .stack-box .stack-item[data-stack-id="{sid}"]'
        LIT = "() => [...document.querySelectorAll('.v3 .stack-box .stack-item.target-ok')].map((e) => Number(e.dataset.stackId)).sort((a, b) => a - b)"

        def setup(v):
            a_ = scn.stack_candidate_actions(v)[0]
            cands = [c for c in scn.candidates(a_) if c["kind"] == "stack_object"]
            return a_, cands, f'.v3 .decision-float .act[data-action-index="{a_["index"]}"]'

        async def click():
            r = await T.at("c3_stack_candidate")
            a_, cands, ACT = setup(r.view)
            cand = cands[-1]          # the LAST of the same-labelled candidates: a guess by label would press the first
            others = [i["id"] for i in r.view["state"]["zones"]["stack"] if i["id"] not in {c["id"] for c in cands}]
            before = await pg.evaluate(LIT)
            await T.click(await T.point(ACT))
            if not await T.wait_for(".v3 .target-picker", 1500):
                raise Missing("the target picker did not open")
            lit = await pg.evaluate(LIT)
            check("stackpick: the open target stage lights exactly the stack items that are its candidates",
                  before == [] and lit == sorted(c["id"] for c in cands), f"{a_['label']}: candidates {[(c['id'], c['label']) for c in cands]}; lit {lit} (before the stage opened: {before}); other stack items {others}")
            await T.click(await T.point(ITEM(cand["id"])))
            sel = await pg.evaluate("() => [...document.querySelectorAll('.v3 .target-picker button.candidate')].map((b) => b.classList.contains('selected'))")
            chosen = await pg.evaluate("() => [...document.querySelectorAll('.v3 .stack-box .stack-item.target-chosen')].map((e) => Number(e.dataset.stackId))")
            arrow = await T.n(f'.v3 svg.v3-arrows path.arrow.chosen[data-to-stack="{cand["id"]}"]')
            order = [c["id"] for c in scn.candidates(a_)]
            check("stackpick: a click on a stack item selects THAT candidate in the picker (same label twice: by position, not by name), and submits nothing",
                  sel == [c == cand["id"] for c in order] and chosen == [cand["id"]] and arrow == 1 and not T.posts,
                  f"picker buttons selected {sel} for candidates {order}; item marked chosen {chosen}; chosen arrow {arrow}; POSTs {len(T.posts)}")
            await T.shot("c3-stackpick")
            await pg.locator(".v3 .target-picker .picker-actions .confirm").click(timeout=2000)
            got = await T.wait_posts(1)
            p = T.posts[-1] if got else None
            check("stackpick: the picker's own Confirm posts that candidate's value, byte for byte, and the server takes it",
                  got and len(T.posts) == 1 and I.post_matches(p, a_["index"], "targets", [cand["value"]]) and p["status"] == 200, I.post_detail(p) if p else "no POST")
        await T.part("stackpick: a click", click)

        async def drop():
            r = await T.at("c3_stack_candidate")
            a_, cands, ACT = setup(r.view)
            cand = cands[0]
            mid = await T.drag(await T.point(ACT), await T.point(ITEM(cand["id"])), release=False,
                               during=lambda: pg.evaluate("(lit) => [document.querySelectorAll('.v3 svg.v3-arrows path.arrow.live').length, eval(lit)()]", LIT))
            await pg.mouse.up()
            got = await T.wait_posts(1)
            p = T.posts[-1] if got else None
            check("stackpick: an arrow dragged from the action onto a stack item posts that candidate's value",
                  mid[0] == 1 and mid[1] == sorted(c["id"] for c in cands) and got and len(T.posts) == 1 and I.post_matches(p, a_["index"], "targets", [cand["value"]]) and p["status"] == 200,
                  f"mid-drag [live arrow, lit stack items] {mid}; " + (I.post_detail(p) if p else "no POST within 5 s"))
        await T.part("stackpick: a drop", drop)

    # ------------------------------------------------------------------ auto-pass through the server parameter
    async def blk_serverpass(T):
        pg = T.pg

        async def stops(**want):
            for key in ("spell", "ability", "combat"):
                await pg.locator(f'{AP} input[data-stop="{key}"]').set_checked(bool(want.get(key)), timeout=2000)

        async def go(mode):
            first = T.last_view
            await pg.locator(f'{AP} .mode[data-mode="{mode}"]').click(timeout=2000)
            settled = await T.idle(900, 60000)
            return first, settled

        def runs():
            return [p for p in T.posts if isinstance(p["body"], dict) and "pass_until" in p["body"]]

        def one_run(first, until, unless, anchor=None):
            """Exactly one POST, and it is the pass + this rule — parsed and byte for byte."""
            if len(T.posts) != 1 or len(runs()) != 1:
                return False
            p = T.posts[0]
            rule = {"until": until, "unless": unless}
            if anchor:
                rule["anchor"] = anchor
            want = {"seq": first["decision"]["seq"], "action_index": pass_index(first), "params": {}, "pass_until": rule}
            return p["body"] == want and p["raw"] == I.compact(want) and '"combat"' not in p["raw"]

        def outcome(p):
            return ((p or {}).get("view") or {}).get("pass_until") or {}

        NONE = {"spell": False, "ability": False, "attackers": False}

        async def until():
            r = await T.at("autopass_round")
            t0, me = r.view["state"]["turn"]["number"], scn.human(r.view)
            await stops()
            first, settled = await go("mynext")
            v, p = T.last_view, (T.posts[0] if T.posts else None)
            o = outcome(p)
            check("serverpass: a whole \"until my next turn\" run is ONE `POST /api/game/action` carrying `pass_until`, and its body is the panel's rule",
                  settled and one_run(first, {"kind": "my_next_turn"}, NONE), f"{len(T.posts)} POST(s), {len(runs())} with pass_until: {(p or {}).get('raw', '')[:240]}")
            check("serverpass: the server made the run (29 passes, as the client loop made in C2) and stopped at `until`",
                  o.get("stop") == "until" and o.get("passes") == 29 and v["state"]["turn"]["active_player"] == me and v["state"]["turn"]["number"] > t0,
                  f"pass_until {o}; turn {t0} -> {v['state']['turn']['number']} ({v['state']['turn']['active_player']})")
            s = await pg.evaluate(JS_PANEL)
            check("serverpass: `until` -> the panel is back to Off, says the server's reason and the pass count, no Resume",
                  s["on"] == ["off"] and bool(o.get("reason")) and o["reason"] in s["status"] and "29 passes" in s["status"] and s["resume"] is None and not s["serverClass"], f"on {s['on']}; status {s['status'][:170]!r}; resume {s['resume']}")
            lines = len((p or {}).get("view", {}).get("events", []))
            feed = await T.n(".v3 .decision-float")      # the page took the batch: the new decision is up in the centre
            dur = await pg.evaluate("() => performance.getEntriesByType('resource').filter((e) => e.name.endsWith('/api/game/action')).map((e) => Math.round(e.duration))")
            check("serverpass: the one response carries the whole run's event lines and the page takes them", lines > 29 and feed == 1, f"{lines} event lines in one response; the request took {dur} ms")
            await T.shot("c3-serverpass-until")
        await T.part("serverpass: until my next turn", until)

        async def eot_and_step():
            r = await T.at("autopass_round")
            t0 = r.view["state"]["turn"]["number"]
            await stops()
            first, settled = await go("eot")
            o = outcome(T.posts[0] if T.posts else None)
            check("serverpass: \"until end of turn\" is one request (`end_of_turn`) and stops at the next turn",
                  settled and one_run(first, {"kind": "end_of_turn"}, NONE) and o.get("stop") == "until" and T.last_view["state"]["turn"]["number"] == t0 + 1 and (await pg.evaluate(JS_PANEL))["on"] == ["off"],
                  f"{len(T.posts)} POST(s); {o}; turn {t0} -> {T.last_view['state']['turn']['number']}")
            r = await T.at("autopass_round")
            await stops(ability=True)
            await pg.locator(f"{AP} select.step").select_option("End", timeout=2000)
            first, settled = await go("step")
            o = outcome(T.posts[0] if T.posts else None)
            check("serverpass: \"until step\" sends the chosen step and the ticked boxes (`ability` here), and stops at that step",
                  settled and one_run(first, {"kind": "step", "step": "End"}, {"spell": False, "ability": True, "attackers": False}) and o.get("stop") == "until" and T.last_view["state"]["turn"]["step"] == "End",
                  f"{(T.posts[0]['raw'] if T.posts else '')[:220]}; {o}; step {T.last_view['state']['turn']['step']}")
        await T.part("serverpass: end of turn, a step", eot_and_step)

        async def unless_spell():
            r = await T.at("autopass_spell")
            t0, step0, me = r.view["state"]["turn"]["number"], r.view["state"]["turn"]["step"], scn.human(r.view)
            await stops(spell=True)
            first, settled = await go("mynext")
            p = T.posts[0] if T.posts else None
            o = outcome(p)
            said = scn.bot_spells((p or {}).get("view", {}).get("events", []), T.last_view)
            check("serverpass: with the spell box ticked the one request carries `unless.spell` and the server stops on a real bot spell",
                  settled and one_run(first, {"kind": "my_next_turn"}, {"spell": True, "ability": False, "attackers": False}) and o.get("stop") == "unless" and bool(said) and o.get("reason") == said[0]["text"],
                  f"{o}; SpellCast lines in the response {[e['text'] for e in said]}")
            s = await pg.evaluate(JS_PANEL)
            check("serverpass: `unless` -> paused: the mode stays chosen, the server's reason text is shown, Resume is offered",
                  s["on"] == ["mynext"] and bool(o.get("reason")) and o["reason"] in s["status"] and s["resume"] == "enabled", f"on {s['on']}; status {s['status'][:170]!r}; resume {s['resume']}")
            await T.shot("c3-serverpass-unless")
            T.posts.clear()
            before = T.last_view
            await pg.locator(f"{AP} .resume").click(timeout=2000)
            settled = await T.idle(900, 60000)
            o2 = outcome(T.posts[0] if T.posts else None)
            s2 = await pg.evaluate(JS_PANEL)
            ended = (o2.get("stop") == "until" and s2["on"] == ["off"]) or (o2.get("stop") in ("unless", "decision") and s2["resume"] is not None)
            check("serverpass: Resume is one more request, carrying the anchor of where the mode was chosen",
                  settled and one_run(before, {"kind": "my_next_turn"}, {"spell": True, "ability": False, "attackers": False}, {"turn": t0, "step": step0}) and ended,
                  f"{(T.posts[0]['raw'] if T.posts else '')[:260]}; {o2}; on {s2['on']}, resume {s2['resume']}")
            # a reload of a paused run never passes
            if s2["resume"] is None:
                await T.at("autopass_spell")
                await stops(spell=True)
                await go("mynext")
            paused = (await pg.evaluate(JS_PANEL))["resume"] is not None
            await T.load()
            await T.idle(1500)
            s3 = await pg.evaluate(JS_PANEL)
            check("serverpass: a reload of a paused run sends nothing and comes back Off", paused and not T.posts and s3["on"] == ["off"] and s3["resume"] is None, f"paused before {paused}; after the reload POSTs {len(T.posts)}, on {s3['on']}, resume {s3['resume']}")
        await T.part("serverpass: the spell stop", unless_spell)

        async def unless_attack():
            r = await T.at("autopass_combat")
            await stops(combat=True)
            first, settled = await go("mynext")
            o = outcome(T.posts[0] if T.posts else None)
            s = await pg.evaluate(JS_PANEL)
            hit = [x["name"] for x in scn.attacks_on_human(T.last_view)]
            check("serverpass: the panel's `combat` box is sent as `attackers` (never `combat`) and the server stops when a bot attacks the human",
                  settled and one_run(first, {"kind": "my_next_turn"}, {"spell": False, "ability": False, "attackers": True}) and o.get("stop") == "unless" and bool(hit) and s["resume"] is not None and o.get("reason", "") in s["status"],
                  f"{(T.posts[0]['raw'] if T.posts else '')[:200]}; {o}; attacking the human {hit}; status {s['status'][:120]!r}")
        await T.part("serverpass: the attack stop", unless_attack)

        async def decision():
            r = await T.at("c3_discard_ahead")
            await stops()
            first, settled = await go("mynext")
            o = outcome(T.posts[0] if T.posts else None)
            s = await pg.evaluate(JS_PANEL)
            d = (T.last_view.get("decision") or {}).get("kind")
            check("serverpass: a real decision on the way (the cleanup discard) stops the server run and is left unanswered",
                  settled and one_run(first, {"kind": "my_next_turn"}, NONE) and o.get("stop") == "decision" and d not in (None, "Priority"), f"{o}; the human now has a {d} decision; {len(scn.my_hand(T.last_view))} cards in hand")
            picker = await T.n(".v3 .decision-host .picker-area") + await T.n(".v3 .decision-float")
            check("serverpass: `decision` -> handed back: paused with the reason, Resume waits for the answer, the decision is on screen",
                  s["on"] == ["mynext"] and s["resume"] == "disabled" and o.get("reason", "") in s["status"] and picker > 0 and len(T.posts) == 1,
                  f"on {s['on']}; resume {s['resume']}; status {s['status'][:170]!r}; decision box / picker on screen {picker}")
            await T.shot("c3-serverpass-decision")
        await T.part("serverpass: a real decision", decision)

        async def locked():
            r = await T.at("autopass_round")
            await stops()
            idle = await pg.evaluate(JS_PANEL)
            T.delay_ms = 900          # hold the page's one POST so the request can be looked at in flight (content untouched)
            try:
                await pg.locator(f'{AP} .mode[data-mode="mynext"]').click(timeout=2000)
                await pg.wait_for_timeout(350)
                mid = await pg.evaluate(JS_PANEL)
                await pg.locator(f'{AP} .mode[data-mode="off"]').click(force=True, timeout=2000)     # pressed anyway
                await pg.evaluate("(sel) => document.querySelector(sel)?.click()", f'{AP} .mode[data-mode="off"]')
                await T.idle(900, 60000)
            finally:
                T.delay_ms = 0
            end = await pg.evaluate(JS_PANEL)
            o = outcome(T.posts[0] if T.posts else None)
            check("serverpass: while the one request is in flight Off, the boxes and the step are disabled and the panel says the run cannot be stopped",
                  mid["dis"].get("off") is True and all(mid["boxes"]) and mid["step"] is True and "cannot be stopped" in mid["status"] and mid["on"] == ["mynext"] and mid["panelClass"]
                  and idle["dis"].get("off") is False and not any(idle["boxes"]),
                  f"in flight: disabled {mid['dis']}, boxes {mid['boxes']}, step {mid['step']}, status {mid['status'][:150]!r}")
            check("serverpass: the decision panel's own Stop button is not shown during a server run (it could not stop it)",
                  mid["serverClass"] and mid["stopShown"] == 0, f".v3.server-passing {mid['serverClass']}; Stop buttons shown {mid['stopShown']} of {mid['stopButtons']}")
            check("serverpass: Off pressed anyway mid-request changes nothing: still one request, the run completes, then everything is enabled again",
                  len(T.posts) == 1 and o.get("stop") == "until" and end["on"] == ["off"] and end["dis"].get("off") is False and not any(end["boxes"]) and not end["serverClass"],
                  f"{len(T.posts)} POST(s); {o}; after: on {end['on']}, disabled {end['dis']}")
        await T.part("serverpass: in flight", locked)

        async def fallback():
            r = await T.at("autopass_round")
            t0, me = r.view["state"]["turn"]["number"], scn.human(r.view)
            await stops()
            refused = []

            async def old_server(route):
                body = route.request.post_data or ""
                if '"pass_until"' in body:
                    refused.append(body)
                    return await route.fulfill(status=400, content_type="application/json",
                                               body=json.dumps({"error": "invalid body: unknown field `pass_until`", "kind": "invalid_body"}))
                await route.fallback()
            await pg.route("**/api/game/action", old_server)
            try:
                first, settled = await go("mynext")
                v = T.last_view
                ok_posts = [p for p in T.posts if p["status"] == 200]
                plain = all("pass_until" not in (p["body"] or {}) and (p["body"] or {}).get("params") == {} for p in ok_posts)
                s = await pg.evaluate(JS_PANEL)
                err = await T.n(".v3 .decision-host .error, .v3 .error-strip")
                check("serverpass: FABRICATED old server (400 `invalid_body` to the request carrying `pass_until`) -> the client loop takes over, one plain pass per request",
                      settled and len(refused) == 1 and len(ok_posts) == 29 and plain and v["state"]["turn"]["active_player"] == me and v["state"]["turn"]["number"] > t0 and s["on"] == ["off"],
                      f"{len(refused)} refused, then {len(ok_posts)} plain passes to turn {v['state']['turn']['number']} ({v['state']['turn']['active_player']}); on {s['on']}; status {s['status'][:120]!r}; error strips {err}")

                # and the fallback loop is the cancellable one: Off stops it mid-run
                r = await T.at("autopass_round")
                await stops()
                refused.clear()
                T.delay_ms = 150
                try:
                    await pg.locator(f'{AP} .mode[data-mode="mynext"]').click(timeout=2000)
                    for _ in range(80):
                        if len([p for p in T.posts if p["status"] == 200]) >= 2:
                            break
                        await pg.wait_for_timeout(50)
                    mid = await pg.evaluate(JS_PANEL)
                    started = len([p for p in T.posts if p["status"] == 200])
                    await pg.locator(f'{AP} .mode[data-mode="off"]').click(timeout=2000)
                    await T.idle(1200, 20000)
                    n1 = len(T.posts)
                    await pg.wait_for_timeout(1200)
                finally:
                    T.delay_ms = 0
                v = T.last_view
                reached = v["state"]["turn"]["number"] > t0 and v["state"]["turn"]["active_player"] == me
                check("serverpass: on the fallback loop Off is enabled and cancels the run (C2's \"Off cancels a run\", now on the path that can be cancelled)",
                      started >= 2 and mid["dis"].get("off") is False and not mid["serverClass"] and len(T.posts) == n1 and not reached and (await pg.evaluate(JS_PANEL))["on"] == ["off"],
                      f"Off pressed after {started} passes (Off disabled then: {mid['dis'].get('off')}); {n1} requests in all, none later ({len(T.posts)}); turn {v['state']['turn']['number']}")
            finally:
                await pg.unroute("**/api/game/action", old_server)
        await T.part("serverpass: the fallback loop", fallback)

    RUN = {"exile": blk_exile, "tax": blk_tax, "gameid": blk_gameid, "targets": blk_targets, "stackpick": blk_stackpick, "serverpass": blk_serverpass}
    wanted = [x.strip() for x in (getattr(a, "c3_blocks", "") or "").split(",") if x.strip()] or list(C3_BLOCKS)
    unknown = [x for x in wanted if x not in RUN]
    if unknown:
        print(f"COULD-NOT-RUN unknown C3 block(s) {unknown}")
        raise SystemExit(2)
    errs = []
    for name in wanted:
        ctx = await browser.new_context(viewport={"width": w, "height": h})
        pg = await ctx.new_page()
        T = Table(pg, base, a.port, errs, a.out)
        await T.route()
        try:
            await RUN[name](T)
        except NotReached as e:
            could_not(f"c3 {name}", str(e))
        except Exception as e:  # noqa: BLE001
            check(f"{name}: the block ran to its end", False, f"{type(e).__name__}: {str(e).splitlines()[0][:260]}")
        if T.loaded:
            try:
                await pg.mouse.move(1, 1)
                await pg.keyboard.press("Escape")
                await pg.wait_for_timeout(200)
                m = await pg.evaluate(I.JS_NOSCROLL)
                check(f"{name}: no page scroll and no box scroll at the end of the block", m["boxes"] >= 5 and not m["bad"], f"{m['boxes']} boxes measured; {m['bad']}")
            except Exception as e:  # noqa: BLE001
                check(f"{name}: no page scroll and no box scroll at the end of the block", False, f"could not measure: {str(e).splitlines()[0][:200]}")
        await ctx.close()
    check("C3: zero page errors", not errs, str(errs[:5]))
