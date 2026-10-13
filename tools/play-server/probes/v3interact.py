# Headless interaction check for the v3 play surface (`/?v3`): the things the greedy sweep
# (v3sweep.py) never touches. One game, one viewport; each check prints ok / BAD:
#   focus by click, focused cell greyed + "across", pile popover (open, Escape closes),
#   seat right-click menu (+ "Bring across the table"), one-second peek (arms on the cell,
#   never on a card; closes on leave), hover overlay, size slider turns auto-fit off and
#   boards may then scroll, auto toggle restores the fit, decision minimise / re-open,
#   a new decision re-opening the minimised box, a turn change changing who is across, the
#   grey-out lifting for blockers, a commander-zone choice being answerable (those four on the
#   server's real answer rewritten in flight), the ?card-window route receiving a hover over a storage event.
# Needs a play-server on 127.0.0.1 (never the owner's 3040):
#   cargo run -p play-server -- --port 3041 --players 4 --bot heuristic --seed 0
#
# C2 (scutemob-267), after those: each interaction on a REAL decision. Every block re-deals by seed
# through the API, drives to its scenario with `v3scenario.drive_until` (same directory — the scenario
# table, the policies and the determinism proof live there), loads `/?v3` in a fresh browser context and
# acts with real pointer events. A block whose scenario cannot be reached — or is reached at another step
# than the scout recorded (drift) — prints `COULD-NOT-RUN <block>` and never counts as ok. The blocks
# (`--blocks` picks some):
#   arrows    drag from a decision card item to an opponent's card / a player's seat: the live arrow,
#             `.arrow-dragging`, `.target-ok` on candidates only, the POST body (action_index + the
#             candidate's `value` byte for byte), 200, the stack; a drop on a non-candidate and Escape
#             mid-drag submit nothing
#   list      the same action by click: the picker lists every candidate; a click on a highlighted
#             card selects it in the picker and draws the chosen arrow; the picker's Confirm submits
#   table     the arrow from a source card on the table (your battlefield, your hand)
#   menu      card context menu: source / target entries with their action index, "Target with" ends
#             in the arrow's POST body, counters, the "No action uses this card" line, the seat menu
#   blockers  the real DeclareBlockers: the box matches `legal_blocks`; click-click, undo, drag; the
#             picker's select follows; its Confirm posts `[[blocker, attacker]]`; combat shows the block;
#             Escape mid-drag cancels the drag only
#   order     drag-rearranging the hand and the battlefield: order kept over a reload, a plain click
#             still plays the land it is on, the played land drops out, a drawn card appends;
#             a new table dealt from the menu does not inherit the order
#   sick      the creature just cast is outlined, and is not on the human's next turn; every creature on
#             screen is outlined exactly when the seat view says so
#   autopass  the panel: until my next turn / end of turn / a step; the spell and combat stops on a real
#             bot spell and a real attack; Resume; Off cancels; a reload never starts passing; no run
#             can start while a picker is open (review fix: it left a stale picker behind)
#   plus, at the end of every block: no page scroll, no box scroll (both axes); and zero page errors.
#
# C3 (scutemob-268), after C2: the client side of the new server fields — per-player exile, commander
# tax, `game_id` as the order key, the `targeted` outline + stack-item arrows, auto-pass as ONE request
# carrying `pass_until`. The blocks are `v3c3.py` (same directory; its header lists them); `--c3-only`
# runs them alone, `--c3-blocks` picks some. C2's `autopass` block still runs and still passes on the
# server path, with ONE check rewritten ("Off cancels a run" -> "Off is disabled while the one server
# request is in flight"; the cancel itself is asserted on the fallback loop in v3c3's `serverpass`).
# C3 (scutemob-268), card images (`hover_checks`, part of the C1 run, alone with --images-only):
#   v3 reads pictures only from the server's local store (`GET /api/card-image`), so these run with no
#   network. A store MISS is made for real: the image requests of one hover are re-addressed to a name
#   the store cannot hold, the server answers its own 404, and the hover overlay and the card window
#   must each show their text chip (never nothing). A store HIT (the natural request) must load the
#   picture (naturalWidth > 0) from /api/card-image; when the server's store does not hold that card
#   the hit checks are COULD-NOT-RUN (exit 2) — fill the store, or expect that against an empty one.
#   No request to a Scryfall host may be made by the page at any point.
# C3 (scutemob-268), after those: the `verify` flow — the never-committed -260 `verify.py` ("play until a
# picker, the stack and combat each appear, assert every usability criterion") re-expressed for v3; the
# mapping table old criterion -> v3 assertion is the comment block above `blk_verify`. `--verify-only` runs it alone.
# Run:  python3 tools/play-server/probes/v3interact.py --port 3041 --seed 3 --out /tmp/v3i
#       python3 tools/play-server/probes/v3interact.py --port 3041 --out /tmp/v3i --verify-only
#       python3 tools/play-server/probes/v3interact.py --port 3041 --seed 3 --out /tmp/v3i --images-only
#       python3 tools/play-server/probes/v3interact.py --port 3041 --out /tmp/v3i --c2-only [--blocks arrows,menu]
# Exit 0 = every check ok, 1 = a check is BAD, 2 = COULD-NOT-RUN (no server / no table / a C2 scenario
# not reached). BAD wins over COULD-NOT-RUN.
import argparse, asyncio, json, os, sys, time
from playwright.async_api import async_playwright

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import v3scenario as scn  # noqa: E402
import v3c3  # noqa: E402  (the C3 blocks: exile, tax, gameid, targets, serverpass — see its header)

RESULTS = []
COULD_NOT = []     # C2 blocks that did not run; any entry makes the exit code 2 (unless something is BAD)


def check(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(("ok   " if ok else "BAD  ") + name + (f" — {detail}" if detail else ""), flush=True)


def could_not(block, why):
    COULD_NOT.append(block)
    print(f"COULD-NOT-RUN {block} — {why}", flush=True)


# ======================================================================================
# C2 (scutemob-267): each interaction on a real decision. See the header for the blocks.
# ======================================================================================
C2_BLOCKS = ("arrows", "list", "table", "menu", "blockers", "order", "sick", "autopass")
CELL = ".v3 .cell:not(.cell-pop)"


class NotReached(Exception):
    """The block's scenario could not be reached: COULD-NOT-RUN, never a pass."""


class Missing(Exception):
    """An element the block has to act on is not on the page: the part is BAD."""


def compact(x):
    """JSON as the browser's `JSON.stringify` writes it, for the byte-for-byte body checks."""
    return json.dumps(x, separators=(",", ":"), ensure_ascii=False)


# The sweep's measurement (v3sweep.py JS_MEASURE), plus the auto-pass panel: nothing scrolls, on either axis.
JS_NOSCROLL = """() => {
  const de = document.documentElement, b = document.body, bad = [];
  if (!(de.scrollHeight === innerHeight && de.scrollWidth === innerWidth) || (b && (b.scrollHeight > innerHeight || b.scrollWidth > innerWidth)))
    bad.push('page ' + [de.scrollWidth, de.scrollHeight, innerWidth, innerHeight].join('/'));
  let boxes = 0;
  for (const sel of ['.v3 .focused', '.v3 .board', '.v3 .cell:not(.cell-pop)', '.v3 .autopass'])
    document.querySelectorAll(sel).forEach((el, i) => {
      boxes += 1;
      if (el.scrollHeight > el.clientHeight + 1 || el.scrollWidth > el.clientWidth + 1)
        bad.push(sel + '[' + i + '] ' + [el.scrollWidth, el.clientWidth, el.scrollHeight, el.clientHeight].join('/'));
    });
  return { bad, boxes };
}"""

# What an arrow drag looks like from the page, mid-gesture.
JS_ARROW = """(arg) => {
  const has = (sel) => [...document.querySelectorAll(sel)].map((e) => e.classList.contains('target-ok'));
  const fl = document.querySelector('.v3 .decision-float');
  return { live: document.querySelectorAll('.v3 svg.v3-arrows path.arrow.live').length,
           dragging: !!document.querySelector('.v3.arrow-dragging'),
           tgt: arg.tgt ? has(arg.tgt) : [], oth: arg.oth ? has(arg.oth) : [],
           okCards: [...new Set([...document.querySelectorAll('.v3 .card.target-ok')].map((e) => Number(e.dataset.oid)))],
           okSeats: [...new Set([...document.querySelectorAll('.v3 [data-seat].target-ok')].map((e) => e.dataset.seat))],
           float: fl ? [getComputedStyle(fl).opacity, getComputedStyle(fl).pointerEvents] : null };
}"""

JS_OIDS = "(sel) => [...document.querySelectorAll(sel)].map((e) => Number(e.dataset.oid))"


class Table:
    """One page in one fresh browser context, with the page's OWN api traffic recorded: every
    `POST /api/game/action` (raw body, parsed body, status, the seat view that came back) and the
    latest seat view the page received. The probe reads the game from those — never from an extra
    GET while the page is acting — and drives scenarios through `v3scenario` only between loads."""

    def __init__(self, pg, base, port, errs, out):
        self.pg, self.base, self.port, self.out = pg, base, port, out
        self.posts, self.last_view, self.loaded = [], None, False
        self.inflight, self.last_io = 0, time.monotonic()
        self.mut, self.delay_ms = None, 0        # rewrite of the server's answer / a slowed action POST
        pg.on("pageerror", lambda e: errs.append(str(e)))
        pg.on("request", self._request)
        pg.on("requestfailed", self._failed)
        pg.on("response", self._response)

    @staticmethod
    def _api(url):
        return "/api/game" in url

    def _request(self, rq):
        if self._api(rq.url):
            self.inflight += 1
            self.last_io = time.monotonic()

    def _failed(self, rq):
        if self._api(rq.url):
            self.inflight -= 1
            self.last_io = time.monotonic()

    def _response(self, resp):
        if self._api(resp.url):
            asyncio.ensure_future(self._read(resp))

    async def _read(self, resp):
        rq = resp.request
        entry = None
        if rq.method == "POST" and rq.url.endswith("/api/game/action"):
            raw = rq.post_data or ""
            try:
                body = json.loads(raw)
            except Exception:  # noqa: BLE001
                body = None
            entry = {"raw": raw, "body": body, "status": resp.status, "view": None}
            self.posts.append(entry)
        try:
            v = await resp.json() if resp.status == 200 else None
        except Exception:  # noqa: BLE001
            v = None
        if isinstance(v, dict) and "state" in v:
            self.last_view = v
            if entry is not None:
                entry["view"] = v
        self.inflight -= 1
        self.last_io = time.monotonic()

    async def route(self):
        async def handler(route):
            if self.delay_ms and route.request.method == "POST":
                await asyncio.sleep(self.delay_ms / 1000)
            if self.mut is None:
                return await route.continue_()
            resp = await route.fetch()
            if resp.status != 200:
                return await route.fulfill(response=resp)
            v = await resp.json()
            self.mut(v)
            await route.fulfill(status=200, content_type="application/json", body=json.dumps(v))
        await self.pg.route("**/api/game", handler)
        await self.pg.route("**/api/game/action", handler)

    async def idle(self, quiet=500, max_ms=45000):
        """Until the page has had no api request in flight for `quiet` ms. False = it never settled."""
        end = time.monotonic() + max_ms / 1000
        while time.monotonic() < end:
            await self.pg.wait_for_timeout(80)
            if self.inflight <= 0 and (time.monotonic() - self.last_io) * 1000 >= quiet:
                return True
        return False

    async def load(self):
        try:
            await self.pg.goto(f"{self.base}/?v3")
            await self.pg.wait_for_selector(".v3 .cell", timeout=10000)
        except Exception as e:  # noqa: BLE001
            raise NotReached(f"the table did not render: {str(e).splitlines()[0]}")
        await self.idle(400)
        self.posts.clear()
        self.loaded = True

    async def at(self, name):
        """Re-deal and drive to a CHOSEN scenario over the API, then load the page on it."""
        r = scn.reach(self.port, name)
        if not r.ok:
            raise NotReached(f"scenario {name!r}: {r.why}")
        if r.why:      # reached, but not at the recorded step: the game is not the one the scout saw
            raise NotReached(f"scenario {name!r}: {r.why}")
        await self.load()
        return r

    async def n(self, sel):
        return await self.pg.locator(sel).count()

    async def text(self, sel):
        return (await self.pg.locator(sel).first.inner_text()).replace("\n", " ").strip() if await self.n(sel) else ""

    async def point(self, sel, fx=0.5, fy=0.5, nth=0):
        loc = self.pg.locator(sel)
        if await loc.count() <= nth:
            raise Missing(f"no `{sel}`" + (f" #{nth}" if nth else ""))
        box = await loc.nth(nth).bounding_box()
        if not box:
            raise Missing(f"`{sel}` is not laid out")
        return box["x"] + box["width"] * fx, box["y"] + box["height"] * fy

    async def corner(self, sel):
        """The empty bottom-right corner of a box (where the C1 seat-menu check clicks)."""
        loc = self.pg.locator(sel)
        if await loc.count() == 0:
            raise Missing(f"no `{sel}`")
        box = await loc.first.bounding_box()
        return box["x"] + box["width"] - 6, box["y"] + box["height"] - 6

    async def covered(self, sel, xy):
        """'' when the topmost element at the point is inside `sel`, else what is on top of it."""
        return await self.pg.evaluate(
            "(a) => { const e = document.elementFromPoint(a.x, a.y); return e && e.closest(a.sel) ? '' : 'covered by ' + (e ? e.tagName.toLowerCase() + '.' + String(e.className && e.className.baseVal !== undefined ? e.className.baseVal : e.className).split(' ').join('.') : 'nothing'); }",
            {"sel": sel, "x": xy[0], "y": xy[1]})

    async def drag(self, src, dst, during=None, release=True, steps=16):
        """A real pointer drag: down on `src`, `steps` moves to `dst` (so pointermove fires), `during()`
        evaluated with the button still down, then up."""
        m = self.pg.mouse
        await m.move(*src)
        await m.down()
        for i in range(1, steps + 1):
            await m.move(src[0] + (dst[0] - src[0]) * i / steps, src[1] + (dst[1] - src[1]) * i / steps)
            await self.pg.wait_for_timeout(12)
        await self.pg.wait_for_timeout(120)
        seen = await during() if during else None
        if release:
            await m.up()
        return seen

    async def click(self, xy, button="left"):
        await self.pg.mouse.click(xy[0], xy[1], button=button)
        await self.pg.wait_for_timeout(200)

    async def wait_posts(self, n=1, ms=5000):
        for _ in range(ms // 80):
            if len(self.posts) >= n:
                break
            await self.pg.wait_for_timeout(80)
        await self.idle(250, 5000)
        return len(self.posts) >= n

    async def wait_for(self, sel, ms=2000):
        for _ in range(ms // 60):
            if await self.n(sel):
                return True
            await self.pg.wait_for_timeout(60)
        return False

    async def shot(self, name):
        await self.pg.screenshot(path=os.path.join(self.out, f"v3i-c2-{name}.png"))

    async def part(self, label, fn):
        """One part of a block. A missing element or an exception makes it BAD, and the block goes on."""
        try:
            await fn()
        except NotReached as e:
            could_not(label, str(e))
        except Exception as e:  # noqa: BLE001
            check(f"{label}: ran to its end", False, f"{type(e).__name__}: {str(e).splitlines()[0][:260]}")
            try:
                await self.pg.mouse.up()
                await self.pg.keyboard.press("Escape")
            except Exception:  # noqa: BLE001
                pass


def post_matches(p, index, field, value):
    """The action POST the page sent: this action, this parameter — parsed AND byte for byte."""
    body = p.get("body") or {}
    return (body.get("action_index") == index and (body.get("params") or {}).get(field) == value
            and f'"{field}":{compact(value)}' in p["raw"])


def post_detail(p):
    return f"HTTP {p['status']} {p['raw'][:200]}"


def on_stack(v, source_name, target):
    return any(i.get("source_name") == source_name and target in (i.get("targets") or []) for i in (v or {}).get("state", {}).get("zones", {}).get("stack", []))


def source_actions(acts, oid):
    """The indexes a card's menu must list under "Use": the answerable actions carrying its object id, plus a
    declare action listing it as eligible (`targeting.svelte.js` `actionsFrom`, `lib/v2/prompt.js` `answerable`)."""
    def usable(x):
        if x["kind"] in ("PassPriority", "Concede"):
            return False
        return x["target_min"] <= 0 or not x["target_slots"] or any(s["candidates"] for s in x["target_slots"])
    return sorted(x["index"] for x in acts if usable(x) and (
        x["object_id"] == oid or any(e["id"] == oid for e in ((x["attack"] or {}).get("eligible") or []) + ((x["block"] or {}).get("eligible") or []))))


def name_of(v, oid):
    z = v["state"]["zones"]
    for zone in ("hand", "battlefield", "command_zone"):
        for cards in z[zone].values():
            for c in cards:
                if c.get("object_id") == oid:
                    return c["name"]
    return None


# ---------------------------------------------------------------- arrows (from a decision card item)
async def blk_arrows(T):
    pg = T.pg
    r = await T.at("target_burn")
    v = r.view
    a = scn.target_actions(v, "opp+player")[0]
    N, cands, opp = a["index"], scn.candidates(a), scn.opp_battlefield_ids(v)
    cand = next(c for c in cands if c["kind"] == "object" and c["id"] in opp)
    cand_ids = {c["id"] for c in cands if c["kind"] == "object"}
    other = next(oid for oid in opp if oid not in cand_ids)
    seats = [c for c in cands if c["kind"] == "player" and c["label"] != scn.human(v)]
    spell = name_of(v, a["object_id"])
    ACT = f'.v3 .decision-float .act[data-action-index="{N}"]'
    TGT, OTH = f'{CELL} .card[data-oid="{cand["id"]}"]', f'{CELL} .card[data-oid="{other}"]'

    async def snap():
        return await pg.evaluate(JS_ARROW, {"tgt": TGT, "oth": OTH})

    async def reset():
        await pg.mouse.up()
        await pg.keyboard.press("Escape")
        await pg.wait_for_timeout(200)
        if not await T.n(ACT):
            await T.load()

    check("arrows: the action is a decision card item carrying its index", await T.n(ACT) == 1, f"{a['label']} = action {N}: {await T.n(ACT)} `.act[data-action-index]`")
    svg = await pg.evaluate("(() => { const s = document.querySelectorAll('.v3 svg.v3-arrows'); return [s.length, s[0] ? getComputedStyle(s[0]).pointerEvents : null, s[0] ? getComputedStyle(s[0]).position : null]; })()")
    check("arrows: one fixed overlay at the root that lets the pointer through", svg == [1, "none", "fixed"], str(svg))

    async def escape():
        src, dst = await T.point(ACT), await T.point(TGT)
        mid = ((src[0] + dst[0]) / 2, (src[1] + dst[1]) / 2)
        before = await T.drag(src, mid, during=snap, release=False)
        await pg.keyboard.press("Escape")
        await pg.wait_for_timeout(150)
        after = await snap()
        await pg.mouse.move(*dst, steps=6)
        await pg.mouse.up()
        await T.idle(500)
        check("arrows: Escape mid-drag cancels the arrow and nothing is submitted",
              before["live"] == 1 and after["live"] == 0 and not after["dragging"] and not T.posts,
              f"live arrow {before['live']} -> {after['live']}, dragging after {after['dragging']}, POSTs {len(T.posts)}")
        await reset()
    await T.part("arrows: Escape mid-drag", escape)

    async def miss():
        st = await T.drag(await T.point(ACT), await T.point(OTH), during=snap)
        await T.idle(600)
        check("arrows: a drop on a card that is not a candidate submits nothing",
              st["live"] == 1 and st["oth"] and not any(st["oth"]) and not T.posts,
              f"{name_of(v, other)} (oid {other}): live {st['live']}, target-ok {st['oth']}, POSTs {len(T.posts)}")
        await reset()
    await T.part("arrows: a drop on a non-candidate", miss)

    async def hit():
        src, dst = await T.point(ACT), await T.point(TGT)
        st = await T.drag(src, dst, during=snap, release=False)
        await T.shot("arrow-live")
        check("arrows: the arrow follows the pointer during the drag", st["live"] == 1, f"{st['live']} `path.arrow.live`")
        check("arrows: the root carries .arrow-dragging during the drag", st["dragging"], f"decision float opacity / pointer-events {st['float']}")
        check("arrows: the candidate card in the opponent's cell is .target-ok", bool(st["tgt"]) and all(st["tgt"]), f"{cand['label']} (oid {cand['id']}): {st['tgt']}")
        check("arrows: only candidates are .target-ok", bool(st["oth"]) and not any(st["oth"]) and set(st["okCards"]) <= cand_ids,
              f"non-candidate {name_of(v, other)}: {st['oth']}; lit cards {sorted(st['okCards'])}, candidates {sorted(cand_ids)}")
        check("arrows: the player candidates light their seats", bool(seats) and {c["label"] for c in seats} <= set(st["okSeats"]), f"seats lit {st['okSeats']}, player candidates {[c['label'] for c in cands if c['kind'] == 'player']}")
        await pg.mouse.up()
        got = await T.wait_posts(1)
        p = T.posts[-1] if got else None
        check("arrows: the drop posts that action with the candidate's value, byte for byte",
              got and len(T.posts) == 1 and post_matches(p, N, "targets", [cand["value"]]),
              post_detail(p) if p else "no POST /api/game/action within 5 s")
        check("arrows: the server accepts it (200) and the spell is on the stack with that target",
              got and p["status"] == 200 and on_stack(p["view"], spell, f"object:{cand['label']}"),
              (f"HTTP {p['status']}, stack {json.dumps((p['view'] or {}).get('state', {}).get('zones', {}).get('stack'))[:240]}" if p else "no POST"))
        gone = await snap()
        check("arrows: the live arrow and the drag class are gone after the drop", gone["live"] == 0 and not gone["dragging"], str({k: gone[k] for k in ('live', 'dragging')}))
    await T.part("arrows: the drop on a candidate", hit)

    async def seat():
        r2 = await T.at("target_burn")
        a2 = scn.target_actions(r2.view, "opp+player")[0]
        pc = next(c for c in scn.candidates(a2) if c["kind"] == "player" and c["label"] != scn.human(r2.view))
        SEAT = f'{CELL}[data-seat="{pc["label"]}"]'
        st = await T.drag(await T.point(ACT), await T.corner(SEAT), during=snap, release=False)
        lit = await pg.evaluate("(sel) => document.querySelector(sel)?.classList.contains('target-ok') ?? null", SEAT)
        await pg.mouse.up()
        got = await T.wait_posts(1)
        p = T.posts[-1] if got else None
        check("arrows: a drop on a player's seat targets that player",
              st["live"] == 1 and lit is True and got and post_matches(p, a2["index"], "targets", [pc["value"]]) and p["status"] == 200
              and on_stack(p["view"], spell, f"player:{pc['label']}"),
              f"seat {pc['label']} target-ok {lit}; " + (post_detail(p) if p else "no POST"))
    await T.part("arrows: a drop on a seat", seat)


# ---------------------------------------------------------------- the list is still there
async def blk_list(T):
    pg = T.pg
    r = await T.at("target_burn")
    v = r.view
    a = scn.target_actions(v, "opp+player")[0]
    N, cands, opp = a["index"], scn.candidates(a), scn.opp_battlefield_ids(v)
    cand = next(c for c in cands if c["kind"] == "object" and c["id"] in opp)
    TGT = f'{CELL} .card[data-oid="{cand["id"]}"]'
    CHOSEN = f'.v3 svg.v3-arrows path.arrow.chosen[data-to-oid="{cand["id"]}"]'

    async def run():
        await T.click(await T.point(f'.v3 .decision-float .act[data-action-index="{N}"]'))
        opened = await T.wait_for(".v3 .target-picker", 1500)
        labels = await pg.locator(".v3 .target-picker button.candidate").all_inner_texts()
        check("list: a click on the action opens the target picker with every candidate",
              opened and sorted(t.strip() for t in labels) == sorted(c["label"] for c in cands) and not T.posts,
              f"picker {opened}, {len(labels)} listed / {len(cands)} in the payload: {[t.strip() for t in labels]}")
        lit = await pg.evaluate(JS_ARROW, {"tgt": TGT, "oth": None})
        check("list: the open target stage lights its candidates on the table", bool(lit["tgt"]) and all(lit["tgt"]), f"{cand['label']}: {lit['tgt']}; lit cards {lit['okCards']}, seats {lit['okSeats']}")
        xy = await T.point(TGT)
        cover = await T.covered(TGT, xy)
        await T.click(xy)
        sel = [t.strip() for t in await pg.locator(".v3 .target-picker button.candidate.selected").all_inner_texts()]
        check("list: a click on a highlighted card selects that candidate in the picker", sel == [cand["label"]] and not T.posts, f"selected {sel}, POSTs {len(T.posts)}" + (f"; the card is {cover}" if cover else ""))
        check("list: the chosen target gets an arrow", await T.n(CHOSEN) == 1, f"{await T.n(CHOSEN)} `path.arrow.chosen[data-to-oid={cand['id']}]`, {await T.n('.v3 svg.v3-arrows path.arrow.chosen')} chosen arrow(s) in all")
        await T.shot("arrow-chosen")
        await pg.locator(".v3 .target-picker .picker-actions .confirm").click(timeout=2000)
        got = await T.wait_posts(1)
        p = T.posts[-1] if got else None
        check("list: the picker's own Confirm posts the same body as the arrow",
              got and len(T.posts) == 1 and post_matches(p, N, "targets", [cand["value"]]) and p["status"] == 200,
              post_detail(p) if p else "no POST")
    await T.part("list", run)


# ---------------------------------------------------------------- arrow from a card on the table
async def blk_table(T):
    pg = T.pg

    async def from_card(tag, scenario, want, zone_sel, where):
        r = await T.at(scenario)
        v = r.view
        opp = scn.opp_battlefield_ids(v)
        place = {c["object_id"] for c in (scn.my_battlefield(v) if where == "battlefield" else scn.my_hand(v))}
        a = next((x for x in scn.target_actions(v, want) if x["object_id"] in place and any(c["id"] in opp for c in scn.candidates(x) if c["kind"] == "object")), None)
        if a is None:
            raise NotReached(f"scenario {scenario!r} has no targeted action whose source is in the human's {where}")
        cand = next(c for c in scn.candidates(a) if c["kind"] == "object" and c["id"] in opp)
        src_name = name_of(v, a["object_id"])
        SRC, TGT = f'{zone_sel} .card[data-oid="{a["object_id"]}"]', f'{CELL} .card[data-oid="{cand["id"]}"]'
        st = await T.drag(await T.point(SRC), await T.point(TGT), release=False, steps=24,
                          during=lambda: pg.evaluate(JS_ARROW, {"tgt": TGT, "oth": None}))
        await T.shot(f"arrow-from-{tag}")
        check(f"table: dragging {tag} off its row starts an arrow", st["live"] == 1 and st["dragging"] and bool(st["tgt"]) and all(st["tgt"]),
              f"{src_name} -> {cand['label']}: live {st['live']}, dragging {st['dragging']}, target-ok {st['tgt']}")
        await pg.mouse.up()
        got = await T.wait_posts(1)
        p = T.posts[-1] if got else None
        check(f"table: the drop from {tag} posts that card's action with the target's value",
              got and len(T.posts) == 1 and post_matches(p, a["index"], "targets", [cand["value"]]) and p["status"] == 200
              and on_stack(p["view"], src_name, f"object:{cand['label']}"),
              f"{a['label']} (action {a['index']}): " + (post_detail(p) if p else "no POST"))
        order = await pg.evaluate(JS_OIDS, f"{zone_sel} .card[data-oid]")
        check(f"table: the drag out of the row rearranged nothing ({tag})", await T.n(".v3 .card.reorder-dragging") == 0 and await T.n(".v3 .reorder-marker") == 0, f"{len(order)} cards left in the row")

    await T.part("table: from your battlefield", lambda: from_card("a permanent of yours", "target_own", "own", ".v3 .board", "battlefield"))
    await T.part("table: from your hand", lambda: from_card("a hand card", "target_table", "opp", ".v3 .hand", "hand"))


# ---------------------------------------------------------------- card context menu
async def blk_menu(T):
    pg = T.pg
    r = await T.at("target_own")
    v = r.view
    acts = scn.actions(v)
    a = scn.target_actions(v, "own")[0]
    N, src, opp = a["index"], a["object_id"], scn.opp_battlefield_ids(v)
    cand = next(c for c in scn.candidates(a) if c["kind"] == "object" and c["id"] in opp)
    used = {x["object_id"] for x in acts if x["object_id"] is not None}
    for x in acts:
        used |= {c["id"] for slot in x["target_slots"] + x["fused_target_slots"] + [s for m in x["modes"] for s in m["target_slots"]] for c in slot["candidates"] if c["kind"] == "object"}
    unused = next((oid for oid in opp if oid not in used), None)
    SRC, TGT = f'.v3 .board .card[data-oid="{src}"]', f'{CELL} .card[data-oid="{cand["id"]}"]'
    MENU = ".v3 .ctx.card-ctx"

    async def indexes(role):
        return [int(x) for x in await pg.evaluate("(sel) => [...document.querySelectorAll(sel)].map((e) => e.dataset.actionIndex)", f'{MENU} button[data-role="{role}"]')]

    async def source():
        await T.click(await T.point(SRC), button="right")
        got = await indexes("source")
        mine = source_actions(acts, src)
        check("menu: right-click on the source card opens its menu", await T.n(f'{MENU}[data-oid="{src}"]') == 1 and await T.n(".v3 .ctx.seat-ctx") == 0,
              f"{name_of(v, src)}: {await T.n(MENU)} card menu(s), {await T.n('.v3 .ctx.seat-ctx')} seat menu(s)")
        check("menu: it lists the actions the card is the source of, by action index", N in got and sorted(got) == mine, f"source entries {got}; the payload's answerable actions on oid {src}: {mine}; the targeted one is {N}")
        await T.shot("menu-source")
        await pg.keyboard.press("Escape")
        await pg.wait_for_timeout(150)
        check("menu: Escape closes it", await T.n(MENU) == 0)
    await T.part("menu: the source card", source)

    async def hand():
        hand_ids = {c["object_id"] for c in scn.my_hand(v)}
        h = next((x for x in acts if x["object_id"] in hand_ids and x["kind"] in ("CastSpell", "PlayLand")), None)
        if h is None:
            raise NotReached("no hand card with an action in this scenario")
        await T.click(await T.point(f'.v3 .hand .card[data-oid="{h["object_id"]}"]'), button="right")
        got = await indexes("source")
        check("menu: a hand card's menu lists its action", await T.n(f'{MENU}[data-oid="{h["object_id"]}"]') == 1 and sorted(got) == source_actions(acts, h["object_id"]), f"{h['label']} = action {h['index']}; source entries {got}, expected {source_actions(acts, h['object_id'])}")
        await pg.keyboard.press("Escape")
        await pg.wait_for_timeout(150)
    await T.part("menu: a hand card", hand)

    async def nothing():
        if unused is None:
            raise NotReached("every opposing permanent is used by some action in this scenario")
        await T.click(await T.point(f'{CELL} .card[data-oid="{unused}"]'), button="right")
        text = await T.text(MENU)
        check("menu: a card nothing uses says so", await T.n(f'{MENU}[data-oid="{unused}"]') == 1 and "no action uses this card" in text.lower() and await T.n(f"{MENU} button[data-role]") == 0,
              f"{name_of(v, unused)} (oid {unused}): {text[:120]!r}")
        await pg.keyboard.press("Escape")
        await pg.wait_for_timeout(150)
    await T.part("menu: an unused card", nothing)

    async def seat_menu():
        await T.click(await T.corner(f"{CELL}:not(.focused-cell)"), button="right")
        items = await pg.locator(".v3 .ctx.seat-ctx button").all_inner_texts()
        check("menu: a right-click on a cell's empty area still opens the seat menu", any("graveyard" in t.lower() for t in items) and any("across" in t.lower() for t in items) and await T.n(MENU) == 0, str(items))
        await pg.keyboard.press("Escape")
        await pg.wait_for_timeout(150)
    await T.part("menu: the seat menu", seat_menu)

    async def counters():
        def put(view):
            for perms in view["state"]["zones"]["battlefield"].values():
                for p in perms:
                    if p["object_id"] == cand["id"]:
                        p["counters"] = {"+1/+1": 2}
        T.mut = put
        try:
            await T.load()
            await T.click(await T.point(TGT), button="right")
            text = await T.text(f"{MENU} .ctx-counters")
            check("menu: a card with counters lists them (a +1/+1 counter written onto the server's real answer in flight)",
                  await T.n(f'{MENU}[data-oid="{cand["id"]}"] .ctx-counters') == 1 and "+1/+1" in text and "2" in text, f"{cand['label']}: {text!r}")
            await T.shot("menu-counters")
        finally:
            T.mut = None
            await T.load()
        await T.click(await T.point(f'.v3 .board .card[data-oid="{src}"]'), button="right")
        check("menu: a card without counters has no counters line", await T.n(MENU) == 1 and await T.n(f"{MENU} .ctx-counters") == 0)
        await pg.keyboard.press("Escape")
        await pg.wait_for_timeout(150)
    await T.part("menu: counters", counters)

    async def target():
        await T.click(await T.point(TGT), button="right")
        got = await indexes("target")
        check("menu: right-click on the target card (opponent's cell) offers the action that can target it", await T.n(f'{MENU}[data-oid="{cand["id"]}"]') == 1 and got == [N], f"{cand['label']}: target entries {got}, expected [{N}] ({a['label']})")
        await T.shot("menu-target")
        await pg.locator(f'{MENU} button[data-role="target"][data-action-index="{N}"]').click(timeout=2000)
        got_post = await T.wait_posts(1)
        p = T.posts[-1] if got_post else None
        check("menu: \"Target with\" ends in the same POST body as the arrow",
              got_post and len(T.posts) == 1 and post_matches(p, N, "targets", [cand["value"]]) and p["status"] == 200
              and on_stack(p["view"], name_of(v, src), f"object:{cand['label']}"),
              post_detail(p) if p else "no POST within 5 s" + (f"; the picker is open ({await T.n('.v3 .target-picker')})" if await T.n(".v3 .target-picker") else ""))
    await T.part("menu: target with", target)


# ---------------------------------------------------------------- blockers
async def blk_blockers(T):
    pg = T.pg
    r = await T.at("block")
    v = r.view
    act_ = scn.block_action(v)
    blk = act_["block"]
    legal = blk["legal_blocks"]
    bid, aid = legal[0]["blocker"]["id"], legal[0]["attackers"][0]["id"]
    bname, aname = legal[0]["blocker"]["label"], legal[0]["attackers"][0]["label"]
    BU = ".v3 .blockui"
    MINE, ATT = f'{BU} .act.mine[data-blocker="{bid}"]', f'{BU} .act.attacker[data-attacker="{aid}"]'
    PATH = f'{BU} svg.arrows path[data-blocker="{bid}"][data-attacker="{aid}"]'

    async def select_value():
        return await pg.evaluate("(i) => document.querySelectorAll('.v3 .blocker-picker .blocker-row')[i]?.querySelector('select.block-target')?.value ?? null", 0)

    async def state():
        await pg.wait_for_timeout(250)
        return {"path": await T.n(PATH), "paths": await T.n(f"{BU} svg.arrows path[data-blocker]"), "select": await select_value(),
                "cls": await pg.evaluate("(sel) => document.querySelector(sel)?.className ?? null", MINE), "hint": await T.text(f"{BU} .hint")}

    async def box():
        shown = await T.wait_for(BU, 2500)
        atts = sorted(await pg.evaluate("(sel) => [...document.querySelectorAll(sel)].map((e) => Number(e.dataset.attacker))", f"{BU} .act.attacker[data-attacker]"))
        mine = sorted(await pg.evaluate("(sel) => [...document.querySelectorAll(sel)].map((e) => Number(e.dataset.blocker))", f"{BU} .act.mine[data-blocker]"))
        want_mine = sorted(x["blocker"]["id"] for x in legal)
        blockable = {y["id"] for x in legal for y in x["attackers"]}
        check("blockers: the box is on screen for the real DeclareBlockers decision", shown and v["decision"]["kind"] == "DeclareBlockers", f"turn {v['state']['turn']['number']}: {aname} attacks, {bname} may block")
        check("blockers: its attackers and blockers are the payload's legal_blocks", shown and mine == want_mine and blockable <= set(atts) <= {x["id"] for x in blk["attackers"]},
              f"blockers {mine} / {want_mine}; attackers {atts} / blockable {sorted(blockable)}")
        check("blockers: the list picker is still there under it", await T.n(".v3 .blocker-picker .blocker-row") == len(legal), f"{await T.n('.v3 .blocker-picker .blocker-row')} rows / {len(legal)} legal blockers")
    await T.part("blockers: the box", box)

    async def escape():
        src, dst = await T.point(MINE), await T.point(ATT)
        mid = ((src[0] + dst[0]) / 2, (src[1] + dst[1]) / 2)
        await T.drag(src, mid, release=False)
        await pg.keyboard.press("Escape")
        await pg.wait_for_timeout(200)
        held = [await T.n(BU), await T.n(".v3 .blocker-picker"), await T.n(f"{BU} svg.arrows path[data-blocker]")]
        await pg.mouse.move(*dst, steps=6)
        await pg.mouse.up()
        s = await state()
        check("blockers: Escape during a blocker drag cancels the drag only",
              held == [1, 1, 0] and await T.n(BU) == 1 and await T.n(".v3 .blocker-picker") == 1 and s["paths"] == 0 and s["select"] != str(aid) and not T.posts,
              f"after Escape [.blockui, .blocker-picker, paths] = {held}; after the release over the attacker: box {await T.n(BU)}, picker {await T.n('.v3 .blocker-picker')}, paths {s['paths']}, select {s['select']!r}, POSTs {len(T.posts)}")
        await pg.mouse.move(1, 1)
    await T.part("blockers: Escape mid-drag", escape)

    async def clicks():
        await T.click(await T.point(MINE))
        s0 = await state()
        check("blockers: a click on a blocker selects it", "sel" in (s0["cls"] or "").split() and s0["paths"] == 0, f"class {s0['cls']!r}, hint {s0['hint']!r}")
        await T.click(await T.point(ATT))
        s1 = await state()
        check("blockers: then a click on its attacker draws the block", s1["path"] == 1 and "assigned" in (s1["cls"] or "").split(), f"{s1['path']} path(s) {bid}->{aid}, class {s1['cls']!r}")
        check("blockers: the picker's select for that blocker now holds the attacker", s1["select"] == str(aid), f"select value {s1['select']!r}, expected {str(aid)!r}")
        check("blockers: the sentence says it in words", bname.lower() in s1["hint"].lower() and aname.lower() in s1["hint"].lower() and "blocks" in s1["hint"].lower(), repr(s1["hint"][:160]))
        check("blockers: nothing is submitted by assigning", not T.posts, f"{len(T.posts)} POST(s)")
        await T.shot("blockers-assigned")
        await T.click(await T.point(MINE))
        s2 = await state()
        check("blockers: a click on the assigned blocker undoes it", s2["paths"] == 0 and s2["select"] != str(aid), f"{s2['paths']} path(s), select {s2['select']!r}")
    await T.part("blockers: click, click, undo", clicks)

    async def by_drag():
        await T.drag(await T.point(MINE), await T.point(ATT))
        s = await state()
        check("blockers: dragging a blocker onto its attacker assigns it too", s["path"] == 1 and s["select"] == str(aid) and not T.posts, f"{s['path']} path(s), select {s['select']!r}, POSTs {len(T.posts)}")
        await pg.locator(".v3 .blocker-picker .picker-actions .confirm").click(timeout=2000)
        got = await T.wait_posts(1)
        p = T.posts[-1] if got else None
        check("blockers: the picker's own button posts blockers = [[blocker, attacker]]",
              got and len(T.posts) == 1 and post_matches(p, act_["index"], "blockers", [[bid, aid]]) and p["status"] == 200,
              post_detail(p) if p else "no POST")
        after = (p or {}).get("view") or {}
        fought = any(x["object_id"] == aid and any(b["object_id"] == bid for b in x.get("blockers", [])) for x in ((after.get("state") or {}).get("combat") or {}).get("attackers", []))
        said = [e["text"] for e in after.get("events", []) if e["kind"] == "BlockersDeclared"]
        check("blockers: the next seat view shows that creature blocking", fought or any(bname in t and aname in t for t in said), f"combat shows it {fought}; events {said}")
    await T.part("blockers: drag and confirm", by_drag)


# ---------------------------------------------------------------- rearranging
async def blk_order(T):
    pg = T.pg
    HAND = ".v3 .hand .card[data-oid]"

    async def hand_dom():
        return await pg.evaluate(JS_OIDS, HAND)

    def same(dom, want):
        """The row shows `want` in order (a paged hand shows a prefix of it; three cards at least)."""
        return len(dom) >= min(3, len(want)) and dom == want[:len(dom)]

    kept = {}

    async def hand():
        r = await T.at("rearrange_hand")
        v = r.view
        view_order = [c["object_id"] for c in scn.my_hand(v)]
        d0 = await hand_dom()
        check("order: the hand starts in the seat view's order", same(d0, view_order) and len(d0) >= 4, f"{len(d0)} shown of {len(view_order)}: {d0}")
        src, dst = await T.point(HAND, nth=0), await T.point(HAND, fx=0.8, nth=2)
        mid = await T.drag(src, dst, release=False, during=lambda: pg.evaluate(
            "() => [document.querySelectorAll('.v3 .hand .card.reorder-dragging').length, document.querySelectorAll('.v3 .reorder-marker').length]"))
        await T.shot("order-hand-drag")
        await pg.mouse.up()
        await T.idle(400)
        want = [view_order[1], view_order[2], view_order[0]] + view_order[3:]
        d1 = await hand_dom()
        check("order: mid-drag the card is marked and an insertion marker shows", mid == [1, 1], f"[.reorder-dragging, .reorder-marker] = {mid}")
        check("order: dragging hand card 1 to after card 3 moves it there", same(d1, want), f"{d0[:5]} -> {d1[:5]}, expected {want[:5]}")
        check("order: the drop submits nothing and opens nothing", not T.posts and await T.n(".v3 .picker-area") == 0 and await T.n(".v3 .st.chooser") == 0 and await T.n(".v3 .reorder-marker") == 0,
              f"POSTs {len(T.posts)}, picker {await T.n('.v3 .picker-area')}, chooser {await T.n('.v3 .st.chooser')}, marker left {await T.n('.v3 .reorder-marker')}")
        await T.load()
        d2 = await hand_dom()
        check("order: the hand order survives a reload", same(d2, want), f"{d2[:5]}, expected {want[:5]}")

        # a plain click is still a click: on a land card it plays the land (if the click does not get
        # that far, the land is played over the API so the rest of the block still runs)
        now = scn.view(T.port)
        land = next(x for x in scn.actions(now) if x["kind"] == "PlayLand")
        await T.click(await T.point(f'{HAND}[data-oid="{land["object_id"]}"]'))
        await T.wait_posts(1, 1500)
        p = T.posts[-1] if T.posts else None
        played = bool(p) and p["status"] == 200 and (p["body"] or {}).get("action_index") == land["index"]
        check("order: a plain click on a hand card still opens its action", played or await T.n(".v3 .picker-area") > 0 or await T.n(".v3 .st.chooser") > 0,
              f"{land['label']} (action {land['index']}): " + (post_detail(p) if p else f"no POST; picker {await T.n('.v3 .picker-area')}, chooser {await T.n('.v3 .st.chooser')}"))
        nxt = p["view"] if played else scn.act(T.port, now["decision"]["seq"], land["index"])

        # the land has left the hand: the others keep their relative order (also after a reload)
        want = [x for x in want if x != land["object_id"]]
        live = await hand_dom()
        await T.load()
        d3 = await hand_dom()
        check("order: a land played from the hand drops out and the rest keep their order", same(d3, want) and land["object_id"] not in d3 and (not played or live == d3),
              f"played {land['label']} (oid {land['object_id']}) by {'the click' if played else 'the API'}: {d3[:6]}, expected {want[:6]}" + ("" if not played or live == d3 else f"; before the reload {live[:6]}"))

        # the next draw: the new card goes last
        known = set(want)
        drew = scn.advance_until(T.port, nxt, lambda x: scn._priority(x) and any(c["object_id"] not in known for c in scn.my_hand(x)), "pass", 120)
        if not drew.ok:
            raise NotReached(f"the next draw: {drew.why}")
        after = [c["object_id"] for c in scn.my_hand(drew.view)]
        new = [x for x in after if x not in known]
        want = [x for x in want if x in after] + new
        await T.load()
        d4 = await hand_dom()
        check("order: a drawn card goes to the end and the rest keep their order", d4 == want, f"drew {[name_of(drew.view, x) for x in new]} on turn {drew.view['state']['turn']['number']}: {d4}, expected {want}")
        kept["hand_game"] = True
    await T.part("order: the hand", hand)

    async def new_game():
        r = await T.at("rearrange_hand")
        old = [c["object_id"] for c in scn.my_hand(r.view)]
        await T.drag(await T.point(HAND, nth=0), await T.point(HAND, fx=0.8, nth=2))
        await T.idle(400)
        moved = await hand_dom()
        key = "() => { try { return JSON.parse(localStorage.getItem('v3-game') || 'null')?.key ?? null; } catch (e) { return null; } }"
        k0 = await pg.evaluate(key)
        # the table's own menu: a different seed (the scenario's is CHOSEN's; 2 is not it)
        seed = 2 if scn.CHOSEN["rearrange_hand"]["seed"] != 2 else 3
        await pg.locator(".v3 .top .v3-menu-wrap > button.btn").click(timeout=2000)
        await pg.locator(".v3 .v3-menu label", has_text="seed").locator("input").fill(str(seed), timeout=2000)
        await pg.locator(".v3 .v3-menu button", has_text="Deal a new table").click(timeout=2000)
        await T.idle(700)
        await pg.wait_for_timeout(300)
        now = scn.view(T.port)
        want = [c["object_id"] for c in scn.my_hand(now)]
        dom = await hand_dom()
        k1 = await pg.evaluate(key)
        shared = [x for x in want if x in old]
        check("order: a new table dealt from the menu does not inherit the hand order",
              moved != old and bool(want) and same(dom, want) and len(shared) >= 3,
              f"old game rearranged to {moved[:5]}; new game (seed {seed}, pregame {now['summary']['pregame']}) shows {dom[:8]}, seat view {want[:8]}; {len(shared)} object ids are in both hands")
        check("order: the new table has a new game key", bool(k0) and bool(k1) and k0 != k1, f"v3-game key {k0!r} -> {k1!r}")
    await T.part("order: a new game", new_game)

    async def board():
        r = await T.at("rearrange_board")
        v = r.view
        creatures = [p["object_id"] for p in scn.my_battlefield(v) if scn.is_creature(p)]
        lands = {p["object_id"] for p in scn.my_battlefield(v) if "Land" in p["card_types"]}
        dom = await pg.evaluate(JS_OIDS, ".v3 .board .card[data-oid]")
        if len(creatures) >= 2:
            what, group = "creatures", [x for x in dom if x in creatures]
        else:
            what, group = "land stacks", [x for x in dom if x in lands]
        if len(group) < 2:
            raise Missing(f"fewer than two {what} drawn on the board ({group}; the view has creatures {creatures}, lands {sorted(lands)})")
        sel = lambda oid: f'.v3 .board .card[data-oid="{oid}"]'
        mid = await T.drag(await T.point(sel(group[0])), await T.point(sel(group[1]), fx=0.8), release=False,
                           during=lambda: pg.evaluate("() => [document.querySelectorAll('.v3 .board .card.reorder-dragging').length, document.querySelectorAll('.v3 .reorder-marker').length]"))
        await T.shot("order-board-drag")
        await pg.mouse.up()
        await T.idle(400)
        want = [group[1], group[0]] + group[2:]
        now = [x for x in await pg.evaluate(JS_OIDS, ".v3 .board .card[data-oid]") if x in group]
        check(f"order: dragging the first of your {what} past the second swaps them", mid == [1, 1] and now == want and not T.posts,
              f"{[name_of(v, x) for x in group]} -> {[name_of(v, x) for x in now]}; mid-drag [dragging, marker] {mid}; POSTs {len(T.posts)}")
        await T.load()
        again = [x for x in await pg.evaluate(JS_OIDS, ".v3 .board .card[data-oid]") if x in group]
        check("order: the battlefield order survives a reload", again == want, f"{[name_of(v, x) for x in again]}")
    await T.part("order: the battlefield", board)


# ---------------------------------------------------------------- summoning sickness
JS_SICK = """(rows) => {
  const wrong = []; let seen = 0, sick = 0;
  for (const [oid, want, name] of rows) {
    const els = [...document.querySelectorAll('.v3 .board .card[data-oid="' + oid + '"], .v3 .cell .card[data-oid="' + oid + '"], .v3 .focused .card[data-oid="' + oid + '"]')];
    if (els.length) seen += 1;
    if (els.some((e) => e.classList.contains('sick'))) sick += 1;
    if (els.some((e) => e.classList.contains('sick') !== want)) wrong.push(name + ' (oid ' + oid + ') wants ' + want);
  }
  return { wrong, seen, sick };
}"""


def sick_rows(v):
    """[oid, should be outlined, name] for every permanent in the seat view. CR 302.6, as `CardFace` reads it:
    a creature the view marks summoning_sick and that has no haste. Anything else is never outlined."""
    rows = []
    for seat, perms in v["state"]["zones"]["battlefield"].items():
        for p in perms:
            haste = any(str(k).lower() == "haste" for k in p.get("keywords") or [])
            rows.append([p["object_id"], bool(scn.is_creature(p) and p["summoning_sick"] and not haste), f"{seat}'s {p['name']}"])
    return rows


async def blk_sick(T):
    pg = T.pg
    r = await T.at("sick")
    p = scn.sick_creatures(r.view)[0]
    oid, name = p["object_id"], p["name"]
    MINE = f'.v3 .board .card[data-oid="{oid}"]'

    async def table(tag):
        rows = sick_rows(T.last_view)
        res = await pg.evaluate(JS_SICK, rows)
        theirs = sum(1 for row in rows if row[1] and not row[2].startswith(scn.human(T.last_view)))
        check(f"sick: every permanent on screen is outlined exactly when the seat view says a creature is summoning sick ({tag})", res["seen"] > 0 and not res["wrong"],
              f"{res['seen']} permanents on screen, {res['sick']} outlined ({theirs} of them an opponent's), wrong: {res['wrong'][:4]}")

    async def fresh():
        cls = await pg.evaluate("(sel) => [...document.querySelectorAll(sel)].map((e) => e.classList.contains('sick'))", MINE)
        check("sick: the creature the human just cast is outlined on their board", bool(cls) and all(cls), f"{name} (oid {oid}), turn {r.view['state']['turn']['number']}: {cls}")
        await T.shot("sick")
        await table("the turn it was cast")
    await T.part("sick: just cast", fresh)

    async def later():
        _, _, then = scn.sick_followup(T.port, r)
        if not then.ok:
            raise NotReached(f"{name} never lost its sickness: {then.why}")
        await T.load()
        cls = await pg.evaluate("(sel) => [...document.querySelectorAll(sel)].map((e) => e.classList.contains('sick'))", MINE)
        check("sick: on the human's next turn the same creature is no longer outlined", bool(cls) and not any(cls), f"{name}, turn {then.view['state']['turn']['number']}: {cls}")
        await table("the human's next turn")
    await T.part("sick: the next turn", later)


# ---------------------------------------------------------------- auto-pass
async def blk_autopass(T):
    pg = T.pg
    AP = ".v3 .autopass"

    async def mode_on():
        return await pg.evaluate("(sel) => [...document.querySelectorAll(sel)].map((e) => e.dataset.mode)", f"{AP} .mode.on")

    async def stops(**want):
        for key in ("spell", "ability", "combat"):
            await pg.locator(f'{AP} input[data-stop="{key}"]').set_checked(bool(want.get(key)), timeout=2000)

    def turn_of(view):
        t = view["state"]["turn"]
        return t["number"], t["active_player"], t["step"]

    def passes_only(first):
        """Every POST of the run answered a Priority decision with PassPriority (read off the view before it).
        Since C3 a run is one POST that also carries `pass_until`; the answer it names is still the pass."""
        prev = first
        for p in T.posts:
            d = (prev or {}).get("decision") or {}
            act_ = next((x for x in d.get("actions", []) if x["index"] == (p["body"] or {}).get("action_index")), None)
            if d.get("kind") != "Priority" or not act_ or act_["kind"] != "PassPriority" or (p["body"] or {}).get("params") not in ({}, None):
                return False
            prev = p["view"]
        return True

    def server_passes():
        """Passes the server reports for the page's run(s) (C3: one request makes a whole run)."""
        return sum(((p["view"] or {}).get("pass_until") or {}).get("passes", 0) for p in T.posts)

    def first_where(pred):
        """Index of the first POST whose answer satisfies `pred`, or None."""
        return next((i for i, p in enumerate(T.posts) if p["view"] and pred(p["view"])), None)

    async def run(mode):
        first = T.last_view
        await pg.locator(f'{AP} .mode[data-mode="{mode}"]').click(timeout=2000)
        settled = await T.idle(900, 60000)
        return first, settled

    async def panel():
        r = await T.at("autopass_round")
        modes = await pg.evaluate("(sel) => [...document.querySelectorAll(sel)].map((e) => e.dataset.mode)", f"{AP} .mode[data-mode]")
        hooks = [await T.n(f"{AP} select.step"), await T.n(f'{AP} input[data-stop="spell"]'), await T.n(f'{AP} input[data-stop="ability"]'), await T.n(f'{AP} input[data-stop="combat"]'), await T.n(f"{AP} .status")]
        check("autopass: the panel is under the right column with its four modes and three stops", await T.n(".v3 .side.right .autopass") == 1 and sorted(modes) == ["eot", "mynext", "off", "step"] and hooks == [1, 1, 1, 1, 1],
              f"modes {modes}; [select.step, spell, ability, combat, .status] = {hooks}")
        await T.idle(1200)
        check("autopass: a freshly loaded page is Off and passes nothing", await mode_on() == ["off"] and not T.posts, f"on: {await mode_on()}, POSTs {len(T.posts)}")
    await T.part("autopass: the panel", panel)

    async def my_next():
        r = await T.at("autopass_round")
        t0, me = r.view["state"]["turn"]["number"], scn.human(r.view)
        await stops()
        before = await T.text(f"{AP} .status")
        first, settled = await run("mynext")
        v = T.last_view
        mine = lambda x: x["state"]["turn"]["number"] > t0 and x["state"]["turn"]["active_player"] == me
        check("autopass: \"until my next turn\" with no stop passes to the human's next turn and no further",
              settled and bool(T.posts) and mine(v) and first_where(mine) == len(T.posts) - 1 and passes_only(first),
              f"turn {t0} -> {turn_of(v)} in {len(T.posts)} request(s) ({server_passes()} server passes); first view of my next turn at request {None if first_where(mine) is None else first_where(mine) + 1}; passes only {passes_only(first)}")
        after = await T.text(f"{AP} .status")
        check("autopass: the mode is back to Off and the status says why it stopped", await mode_on() == ["off"] and bool(after) and after != before and await T.n(f"{AP} .resume") == 0, f"on: {await mode_on()}; status {after[:140]!r}")
        await T.shot("autopass-mynext")
    await T.part("autopass: until my next turn", my_next)

    async def eot():
        r = await T.at("autopass_round")
        t0 = r.view["state"]["turn"]["number"]
        await stops()
        first, settled = await run("eot")
        v = T.last_view
        nxt = lambda x: x["state"]["turn"]["number"] > t0
        check("autopass: \"until end of turn\" stops at the next turn", settled and bool(T.posts) and v["state"]["turn"]["number"] == t0 + 1 and first_where(nxt) == len(T.posts) - 1 and passes_only(first) and await mode_on() == ["off"],
              f"turn {t0} -> {turn_of(v)} in {len(T.posts)} request(s) ({server_passes()} server passes); on: {await mode_on()}")
    await T.part("autopass: until end of turn", eot)

    async def step():
        r = await T.at("autopass_round")
        await stops()
        await pg.locator(f"{AP} select.step").select_option("End", timeout=2000)
        first, settled = await run("step")
        v = T.last_view
        at_end = lambda x: x["state"]["turn"]["step"] == "End"
        check("autopass: \"until step\" with End chosen stops at an End step", settled and bool(T.posts) and at_end(v) and first_where(at_end) == len(T.posts) - 1 and passes_only(first) and await mode_on() == ["off"],
              f"{turn_of(first)} -> {turn_of(v)} in {len(T.posts)} request(s) ({server_passes()} server passes); on: {await mode_on()}")
    await T.part("autopass: until a step", step)

    async def spell():
        r = await T.at("autopass_spell")
        t0, me = r.view["state"]["turn"]["number"], scn.human(r.view)
        await stops(spell=True)
        first, settled = await run("mynext")
        v = T.last_view
        cast = lambda x: bool(scn.bot_spells(x.get("events", []), x))
        said = scn.bot_spells(v.get("events", []), v)
        text = said[0]["text"] if said else ""
        card = text.split(" casts ", 1)[-1] if text else ""
        check("autopass: with the spell stop on, the run stops on a real bot SpellCast", settled and bool(said) and first_where(cast) == len(T.posts) - 1 and passes_only(first) and v["state"]["turn"]["number"] < t0 + 4,
              f"{len(T.posts)} request(s) ({server_passes()} server passes), then {text!r} (turn {turn_of(v)})")
        status = await T.text(f"{AP} .status")
        check("autopass: the status carries the event and Resume is shown", bool(card) and (text in status or card in status) and await T.n(f"{AP} .resume") == 1, f"status {status[:160]!r}; resume {await T.n(f'{AP} .resume')}; on: {await mode_on()}")
        await T.shot("autopass-spell-stop")
        n0 = len(T.posts)
        await pg.locator(f"{AP} .resume").click(timeout=2000)
        settled = await T.idle(900, 60000)
        v2 = T.last_view
        ended = (await T.n(f"{AP} .resume") == 1) or (await mode_on() == ["off"] and v2["state"]["turn"]["active_player"] == me and v2["state"]["turn"]["number"] > t0)
        check("autopass: Resume continues the same run", settled and len(T.posts) > n0 and ended,
              f"{len(T.posts) - n0} more request(s) to {turn_of(v2)}; paused again {await T.n(f'{AP} .resume') == 1}, on: {await mode_on()}, status {(await T.text(f'{AP} .status'))[:120]!r}")
    await T.part("autopass: the spell stop", spell)

    async def reload():
        await T.at("autopass_spell")
        await stops(spell=True)
        _, settled = await run("mynext")
        paused = await T.n(f"{AP} .resume") == 1
        await T.load()
        await T.idle(1500)
        check("autopass: a reload of a paused run never starts passing by itself", settled and paused and not T.posts and await mode_on() == ["off"] and await T.n(f"{AP} .resume") == 0,
              f"paused before the reload {paused}; after: POSTs {len(T.posts)}, on: {await mode_on()}, resume {await T.n(f'{AP} .resume')}")
    await T.part("autopass: reload", reload)

    async def combat():
        r = await T.at("autopass_combat")
        t0 = r.view["state"]["turn"]["number"]
        await stops(combat=True)
        first, settled = await run("mynext")
        v = T.last_view
        hit = lambda x: bool(scn.attacks_on_human(x))
        check("autopass: with the combat stop on, the run stops when a bot declares attackers at the human",
              settled and hit(v) and first_where(hit) == len(T.posts) - 1 and passes_only(first) and await T.n(f"{AP} .resume") == 1,
              f"{len(T.posts)} request(s) ({server_passes()} server passes) from turn {t0}; {[x['name'] for x in scn.attacks_on_human(v)]} attacking at {turn_of(v)}; resume {await T.n(f'{AP} .resume')}; status {(await T.text(f'{AP} .status'))[:120]!r}")
        await T.shot("autopass-combat-stop")
    await T.part("autopass: the combat stop", combat)

    async def picker_open():
        r = await T.at("target_burn")
        a = scn.target_actions(r.view, "opp+player")[0]
        await T.click(await T.point(f'.v3 .decision-float .act[data-action-index="{a["index"]}"]'))
        if not await T.wait_for(".v3 .decision-host .picker-area .target-picker", 1500):
            raise Missing("the target picker did not open on a click on the action")
        seq0 = T.last_view["decision"]["seq"]
        dis = "(sel) => Object.fromEntries([...document.querySelectorAll(sel)].map((e) => [e.dataset.mode, e.disabled]))"
        d0 = await pg.evaluate(dis, f"{AP} .mode[data-mode]")
        status = await T.text(f"{AP} .status")
        check("autopass: while a picker is open every mode but Off is disabled and the status says why",
              d0 == {"off": False, "eot": True, "mynext": True, "step": True} and "picker" in status.lower(), f"disabled {d0}; status {status[:140]!r}")
        for mode in ("eot", "mynext", "step"):      # pressed anyway, three ways; none may start a run
            await pg.locator(f'{AP} .mode[data-mode="{mode}"]').click(force=True, timeout=2000)
            await pg.locator(f'{AP} .mode[data-mode="{mode}"]').dispatch_event("click")
            await pg.evaluate("(sel) => document.querySelector(sel)?.click()", f'{AP} .mode[data-mode="{mode}"]')
        await pg.wait_for_timeout(900)
        seq1 = scn.view(T.port)["decision"]["seq"]
        check("autopass: a mode pressed anyway while the picker is open sends nothing and leaves the picker on its decision",
              not T.posts and await T.n(".v3 .decision-host .picker-area .target-picker") == 1 and seq1 == seq0 and await mode_on() == ["off"],
              f"POSTs {len(T.posts)} in 900 ms; picker {await T.n('.v3 .target-picker')}; server decision seq {seq0} -> {seq1}; on: {await mode_on()}")
        await pg.locator(".v3 .target-picker .picker-actions .cancel").click(timeout=2000)
        await pg.wait_for_timeout(250)
        d1 = await pg.evaluate(dis, f"{AP} .mode[data-mode]")
        check("autopass: backing out of the picker enables the modes again", await T.n(".v3 .picker-area") == 0 and not any(d1.values()) and len(d1) == 4 and not T.posts, f"disabled {d1}; picker {await T.n('.v3 .picker-area')}")
    await T.part("autopass: a picker is open", picker_open)

    async def off():
        # C3 (scutemob-268) changed what this can assert. A run is now ONE request the server completes
        # (`pass_until`), so there is no "between passes" for Off to land in: C2's "Off cancels a run" is
        # asserted where it is still true — on the fallback client loop — by `v3c3.py` ("serverpass: on the
        # fallback loop Off is enabled and cancels the run"). Here: Off cannot interrupt the server run, and
        # the panel does not pretend it can.
        r = await T.at("autopass_round")
        t0, me = r.view["state"]["turn"]["number"], scn.human(r.view)
        await stops()
        T.delay_ms = 700            # hold the page's one action POST so Off is pressed mid-request (content untouched)
        try:
            await pg.locator(f'{AP} .mode[data-mode="mynext"]').click(timeout=2000)
            await pg.wait_for_timeout(300)
            off_disabled = await pg.evaluate("(sel) => document.querySelector(sel)?.disabled ?? null", f'{AP} .mode[data-mode="off"]')
            await pg.locator(f'{AP} .mode[data-mode="off"]').click(force=True, timeout=2000)
            await T.idle(1200, 60000)
        finally:
            T.delay_ms = 0
        v = T.last_view
        reached = v["state"]["turn"]["number"] > t0 and v["state"]["turn"]["active_player"] == me
        check("autopass: Off is disabled while the one server request is in flight, and the run completes (cancelling is the fallback loop's — see C3 serverpass)",
              off_disabled is True and len(T.posts) == 1 and reached and await mode_on() == ["off"],
              f"Off disabled mid-request: {off_disabled}; {len(T.posts)} request(s); at {turn_of(v)}; on: {await mode_on()}")
    await T.part("autopass: Off", off)


C2_RUN = {"arrows": blk_arrows, "list": blk_list, "table": blk_table, "menu": blk_menu, "blockers": blk_blockers,
          "order": blk_order, "sick": blk_sick, "autopass": blk_autopass}


async def c2(browser, a, base, w, h):
    """The C2 section. One fresh browser context per block (no stored order, no stored auto-pass settings)."""
    print("---- C2 (scutemob-267): each interaction on a real decision", flush=True)
    wanted = [x.strip() for x in a.blocks.split(",") if x.strip()] or list(C2_BLOCKS)
    unknown = [x for x in wanted if x not in C2_RUN]
    if unknown:
        print(f"COULD-NOT-RUN unknown C2 block(s) {unknown}"); sys.exit(2)
    errs = []
    for name in wanted:
        ctx = await browser.new_context(viewport={"width": w, "height": h})
        pg = await ctx.new_page()
        T = Table(pg, base, a.port, errs, a.out)
        await T.route()
        try:
            await C2_RUN[name](T)
        except NotReached as e:
            could_not(name, str(e))
        except Exception as e:  # noqa: BLE001
            check(f"{name}: the block ran to its end", False, f"{type(e).__name__}: {str(e).splitlines()[0][:260]}")
        if T.loaded:
            try:
                await pg.mouse.move(1, 1)
                await pg.keyboard.press("Escape")
                await pg.wait_for_timeout(200)
                m = await pg.evaluate(JS_NOSCROLL)
                check(f"{name}: no page scroll and no box scroll at the end of the block", m["boxes"] >= 5 and not m["bad"], f"{m['boxes']} boxes measured; {m['bad']}")
            except Exception as e:  # noqa: BLE001
                check(f"{name}: no page scroll and no box scroll at the end of the block", False, f"could not measure: {str(e).splitlines()[0][:200]}")
        await ctx.close()
    check("C2: zero page errors", not errs, str(errs[:5]))


MISS_NAME = "v3interact no such card"   # a name no store holds: the server's own 404 for it

JS_HOVER_STATE = """() => {
  const all = document.querySelectorAll('.v3 .hover-card');
  const e = all[0];
  if (!e) return null;
  const r = e.getBoundingClientRect(), img = e.querySelector('img'), chip = e.querySelector('.hover-chip');
  const shown = (n) => !!n && getComputedStyle(n).display !== 'none' && n.getBoundingClientRect().height > 50;
  return { count: all.length, display: getComputedStyle(e).display, box: [r.left, r.top, r.right, r.bottom],
           face: e.dataset.face ?? null, imgShown: shown(img), chipShown: shown(chip),
           chipName: (e.querySelector('.hover-chip-name')?.textContent ?? '').trim(),
           src: img?.getAttribute('src') ?? '', natural: img?.naturalWidth ?? 0 };
}"""

JS_CARDWIN_STATE = """() => {
  const root = document.querySelector('.v3-cardwin'), img = root?.querySelector('img'), chip = root?.querySelector('.chip');
  return { text: (root?.innerText ?? ''), img: !!img, src: img?.getAttribute('src') ?? '', natural: img?.naturalWidth ?? 0,
           chip: !!chip && chip.getBoundingClientRect().height > 50,
           chipName: (chip?.querySelector('.chip-name')?.textContent ?? '').trim() };
}"""


def watch_scryfall(ctx):
    """Every request of the context that goes to a Scryfall host (v3 must make none)."""
    hits = []
    ctx.on("request", lambda rq: hits.append(rq.url[:200]) if "scryfall" in (rq.url.split("/")[2] if rq.url.startswith("http") else "").lower() else None)
    return hits


async def hover_checks(ctx, pg, base, w, h):
    """C1: the one hover overlay and the card window. C3: the pictures are the local store's; a miss is a chip."""
    answers = []                                       # (url, status) of every image-route answer in this context
    ctx.on("response", lambda resp: answers.append((resp.url, resp.status)) if "/api/card-image?" in resp.url else None)

    async def answered(version, since, ms=3000):
        for _ in range(ms // 50):
            got = [st for (u, st) in answers[since:] if f"version={version}" in u]
            if got:
                return got[-1]
            await pg.wait_for_timeout(50)
        return None

    is_image = lambda url: "/api/card-image?" in url   # noqa: E731
    cw = await ctx.new_page()
    await cw.goto(f"{base}/?card-window")
    await cw.wait_for_selector(".v3-cardwin", timeout=5000)
    hand = pg.locator(".v3 .hand .card").first
    name = await hand.get_attribute("data-name")
    inside = lambda v: bool(v) and v["count"] == 1 and v["display"] != "none" and v["box"][0] >= 0 and v["box"][1] >= 0 and v["box"][2] <= w and v["box"][3] <= h  # noqa: E731

    # a store miss, for real: this hover's image requests go to the server under a name it cannot hold
    async def readdress(route):
        url = route.request.url
        head, _, tail = url.partition("name=")
        await route.continue_(url=head + "name=" + MISS_NAME.replace(" ", "%20") + ("&" + tail.split("&", 1)[1] if "&" in tail else ""))
    await ctx.route(is_image, readdress)
    mark = len(answers)
    await hand.hover()
    normal = await answered("normal", mark)
    await pg.wait_for_timeout(200)
    vis = await pg.evaluate(JS_HOVER_STATE)
    check("a store miss: the hover shows its chip, not nothing", normal == 404 and inside(vis) and vis["face"] == "chip" and vis["chipShown"]
          and not vis["imgShown"] and vis["chipName"] == name, f"{name!r} HTTP {normal} {vis}")
    large = await answered("large", mark)
    await cw.wait_for_timeout(200)
    win = await cw.evaluate(JS_CARDWIN_STATE)
    check("a store miss: the card window shows its chip", large == 404 and bool(name) and win["chip"] and win["chipName"] == name and not win["img"],
          f"{name!r} HTTP {large} {win}")
    await pg.screenshot(path=os.path.join(OUT_DIR, "images-miss-hover.png"))
    await cw.screenshot(path=os.path.join(OUT_DIR, "images-miss-cardwin.png"))
    await ctx.unroute(is_image, readdress)
    await pg.mouse.move(w // 2, 5)
    await pg.wait_for_timeout(150)

    # the natural request: whatever this server's store holds for the card
    mark = len(answers)
    await hand.hover()
    normal = await answered("normal", mark)
    await pg.wait_for_timeout(300)
    vis = await pg.evaluate(JS_HOVER_STATE)
    face_ok = bool(vis) and ((normal == 200 and vis["face"] == "image" and vis["imgShown"]) or (normal == 404 and vis["face"] == "chip" and vis["chipShown"] and vis["chipName"] == name))
    check("hover shows the one overlay, inside the viewport", inside(vis) and face_ok, f"{name!r} HTTP {normal} {vis}")
    large = await answered("large", mark)
    await cw.wait_for_timeout(300)
    win = await cw.evaluate(JS_CARDWIN_STATE)
    win_ok = (large == 200 and win["img"] and "version=large" in win["src"]) or (large == 404 and win["chip"] and win["chipName"] == name)
    check("the card window shows the hovered card", bool(name) and name in win["text"] and win_ok, f"{name!r} HTTP {large} {win}")
    if normal == 200 and large == 200:
        check("a store hit: the hover picture loaded from /api/card-image", vis["natural"] > 0 and vis["src"].startswith("/api/card-image?") and "version=normal" in vis["src"]
              and not vis["chipShown"], str(vis))
        check("a store hit: the card window picture loaded from /api/card-image", win["natural"] > 0 and win["src"].startswith("/api/card-image?") and not win["chip"], str(win))
        await pg.screenshot(path=os.path.join(OUT_DIR, "images-hit-hover.png"))
        await cw.screenshot(path=os.path.join(OUT_DIR, "images-hit-cardwin.png"))
    else:
        could_not("images: store hit", f"the server's store does not hold {name!r} (normal HTTP {normal}, large HTTP {large})")
    await cw.close()
    await pg.mouse.move(w // 2, 5)
    await pg.wait_for_timeout(100)
    check("leaving the card hides the overlay", await pg.evaluate("getComputedStyle(document.querySelector('.v3 .hover-card')).display") == "none")


OUT_DIR = "."


# ======================================================================================
# The `verify.py` flow, re-expressed for v3 (scutemob-268 criterion 2).
#
# The -260 usability pass (`memory/play-ui-usability-2026-10-09.md`) ended with a scratch `verify.py`
# that was never committed: seed 3, play "until a picker, the stack and combat each appear", and assert
# every usability criterion of that pass on the OLD surface. This block is that flow on `/?v3`: the same
# seed 3 game (policy `develop`, `v3scenario.CHOSEN` verify_stack / verify_picker / verify_combat —
# turn 3, turn 9, turn 22 of one game, driven over the API between the stops, each stop loaded in the
# page and ANSWERED through the page), plus one stop on the C2 `block` game for a blocker in the combat
# card. At every stop the criteria below are asserted. `--verify-only` runs this block alone.
#
# Old criterion (finding # of the -260 notes)       -> v3 assertion                                   -> check name
#  0 served dist/ was stale                          -> not applicable on v3: a build-process note, not a property of a surface
#  1 nothing marks which cards can be used           -> the `.card.lit` set on the table is exactly the decision's offered ids
#                                                       (object_id of every answerable non-mana action + declare `eligible`)   -> "the lit cards are exactly the cards the decision offers"
#  2 three identical "Play Plains" buttons           -> v3 options are cards (brief §1 rule 11), one item per card by design, so
#                                                       "folded by label" becomes: no two loose buttons share a label, every card
#                                                       item carries a distinct action index of this decision, and identical mana
#                                                       sources show once (one muted item per name+label)                         -> "no decision option is drawn twice"
#  3 raw enum names in the UI (PreCombatMain,        -> no engine identifier (step / decision-kind / action-kind names) anywhere in
#    DECLAREATTACKERS, "PreCombatMain — PreCombatMain")  the surface's visible text; the lit phase-track segment is the humanised step -> "no raw engine identifier on screen", "the phase track lights the current step"
#  4 a full-width empty COMBAT box every combat      -> v3 keeps a fixed combat card in the dock by design (rule 10); the criterion
#                                                       becomes: it draws defender boxes only for declared attackers and says
#                                                       "not in combat" / "no attackers declared" otherwise, matching `state.combat` -> "the combat card shows exactly the declared attackers"
#  5 declare-attackers needed an extra click         -> on a DeclareAttackers decision with one declare action the attacker picker
#                                                       is open with no click; Back closes it and it stays closed; its button re-opens -> "the attacker picker opens by itself", "Back closes it and it stays closed"
#  6 feed stopped following after pass-until         -> not applicable on v3: v3 draws no event feed (C1 notes "No log panel")
#  7 eight feed lines per priority round             -> not applicable on v3: no event feed
#  8 "STOPPED" strip stays after the human acted     -> the auto-pass panel's stop reason is shown after a finished run and is gone
#                                                       once the human answers the next decision                                  -> "a finished auto-pass run's stop line clears once the human acts"
#  9 names cut at 11 chars; a land stack's ×N fell   -> every card on the table carries its full name (name band / chip text ==
#    off; the combat row's attacker name cut            `data-name`, a band clipped by its two lines still has the hover); every
#                                                       land-stack count sits inside its card; stack / combat / phase / button text
#                                                       is not clipped (scrollWidth <= clientWidth)                               -> "names are whole: cards, counts, dock text"
# 10 "Lands: 1" read as a land count                 -> not applicable on v3: `land_plays_remaining` is not drawn anywhere on v3
#                                                       (asserted: no "Lands: N" text; v3's "LANDS n" row label IS a land count)   -> "no \"Lands: N\" counter that means land drops"
# v3's own metrics (docs/play-surface-v3-brief.md §6 and §1 rules 10, 11, 13), at every stop:
#    no page scroll, no auto-mode box scroll                                                                                       -> "nothing scrolls"
#    every face-up card has an image or a chip                                                                                     -> "every card has an image or a chip"
#    the decision is reachable and visible: the centred box inside the viewport, its answering control inside the
#    viewport and the topmost element at its own centre                                                                            -> "the decision box and its answer are on screen and on top"
#    the decision is answerable: answered through the page, the page's POST is a 200                                               -> "… answered through the page (200)"
#    stack: one row per stack object, top first, the top row lit and saying "resolves next"; text legible                          -> "the stack lists …", "the top item is lit …"
#    legibility (rule 13: stack text, panel titles; plus the sentence and the answer button): >= 9px and contrast >= 4.5:1        -> "dock text is legible"
#    combat: the card and its enlarged view                                                                                        -> "the combat card enlarges …", "a blocker is drawn beside its attacker"
# Not translated (listed, not dropped): 0, 6, 7 (above). Nothing else of the -260 list is missing.
# ======================================================================================
RAW_IDENTIFIERS = ["precombatmain", "postcombatmain", "beginningofcombat", "declareattackers", "declareblockers", "combatdamage",
                   "endofcombat", "firststrikedamage", "cleanupdiscard", "passpriority", "castspell", "playland", "activateability",
                   "tapformana", "effectchoice", "triggertargets", "commanderzonechoice"]
SHORT_STEP_WORD = {"Untap": "untap", "Upkeep": "upkeep", "Draw": "draw", "PreCombatMain": "main 1", "BeginningOfCombat": "begin combat",
                   "DeclareAttackers": "attackers", "DeclareBlockers": "blockers", "FirstStrikeDamage": "first", "CombatDamage": "damage",
                   "EndOfCombat": "end combat", "PostCombatMain": "main 2", "End": "end", "Cleanup": "cleanup"}

JS_USABILITY = """(raw) => {
  const txt = (e) => ((e && e.textContent) || '').replace(/\\s+/g, ' ').trim();
  const q = (s, root) => [...(root || document).querySelectorAll(s)];
  const lum = (c) => { const m = c.match(/[\\d.]+/g).map(Number); const f = (x) => { x /= 255; return x <= 0.03928 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4); };
                       return [0.2126 * f(m[0]) + 0.7152 * f(m[1]) + 0.0722 * f(m[2]), m.length > 3 ? m[3] : 1]; };
  const bg = (e) => { for (let n = e; n; n = n.parentElement) { const l = lum(getComputedStyle(n).backgroundColor); if (l[1] > 0.5) return l[0]; } return 0; };
  const zoom = (e) => { let z = 1; for (let n = e; n; n = n.parentElement) z *= parseFloat(getComputedStyle(n).zoom) || 1; return z; };
  const leg = (sel) => q(sel).map((e) => { const cs = getComputedStyle(e); const f = lum(cs.color)[0], b = bg(e);
    return { sel, text: txt(e).slice(0, 28), px: Math.round(parseFloat(cs.fontSize) * zoom(e) * 10) / 10, contrast: Math.round((Math.max(f, b) + 0.05) / (Math.min(f, b) + 0.05) * 10) / 10 }; });
  const table = (c) => !c.closest('.decision-float, .popover, .dock, .hover-card');
  const onTop = (e) => { if (!e) return null; const r = e.getBoundingClientRect(); const hit = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2);
    return { inside: r.width > 0 && r.left >= 0 && r.top >= 0 && r.right <= innerWidth + 0.5 && r.bottom <= innerHeight + 0.5, top: !!hit && (e === hit || e.contains(hit)), text: txt(e).slice(0, 40) }; };
  const fl = document.querySelector('.v3 .decision-float');
  const answer = fl ? (fl.querySelector('.picker-area .picker-actions .confirm') || fl.querySelector('.prompt .buttons .btn.primary')) : null;
  const text = document.querySelector('.v3').innerText.toLowerCase();
  const cards = q('.v3 .card:not(.back)');
  const bands = q('.v3 .card .band, .v3 .card .chip-name');
  const clipSel = '.v3 .stack-item .s-who, .v3 .stack-item .s-name, .v3 .stack-item .s-text, .v3 .combat-box .to, .v3 .phase .seg, .v3 .decision-float .prompt .buttons .btn, .v3 .decision-float .sentence, .v3 .picker-actions button';
  return {
    lit: [...new Set(q('.v3 .card.lit').filter(table).map((c) => Number(c.dataset.oid)))].sort((a, b) => a - b),
    present: [...new Set(q('.v3 .card[data-oid]').filter(table).map((c) => Number(c.dataset.oid)))],
    raw: raw.filter((w) => text.includes(w)), landsCounter: /\\blands:\\s*\\d/.test(text),
    segNow: q('.v3 .phase .seg.now').map(txt), segs: q('.v3 .phase .seg').length,
    buttons: fl ? q('.prompt .buttons .btn', fl).map(txt) : [], items: fl ? q('.act[data-action-index]', fl).map((e) => [Number(e.dataset.actionIndex), !!e.closest('.group.muted'), e.querySelector('.card')?.dataset.name || '', txt(e.querySelector('.cap'))]) : [],
    float: onTop(fl), answer: onTop(answer), picker: q('.v3 .picker-area').length, attackerPicker: q('.v3 .picker-area .attacker-picker').length,
    toggles: q('.v3 .picker-area .attacker-toggle').map(txt),
    stack: q('.v3 .stack-box .stack-item').map((e) => { const cs = getComputedStyle(e); return { top: e.classList.contains('top'), who: txt(e.querySelector('.s-who')), name: txt(e.querySelector('.s-name')), text: txt(e.querySelector('.s-text')),
                                                           edge: cs.borderLeftColor, other: cs.borderBottomColor, edgeW: parseFloat(cs.borderLeftWidth) }; }),
    stackEmpty: q('.v3 .stack-box .dim').map(txt),
    combat: { box: q('.v3 .combat-box').length, text: txt(document.querySelector('.v3 .combat-box')).slice(0, 160),
              defs: q('.v3 .combat-box .def').map((d) => ({ to: txt(d.querySelector('.to')), pairs: q('.pair', d).map((p) => ({ blocked: p.classList.contains('blocked'),
                attacker: Number(q('.card', p).find((c) => !c.closest('.blocker'))?.dataset.oid), blockers: q('.blocker .card', p).map((c) => Number(c.dataset.oid)) })) })) },
    legible: ['.v3 .stack-box .panel-title', '.v3 .stack-item .s-who', '.v3 .stack-item .s-name', '.v3 .stack-item .s-text', '.v3 .combat-box .panel-title', '.v3 .combat-box .to',
              '.v3 .decision-float .head .panel-title', '.v3 .decision-float .prompt .sentence', '.v3 .decision-float .prompt .buttons .btn.primary', '.v3 .picker-area .picker-actions .confirm'].flatMap(leg),
    faint: ['.v3 .decision-float .stage-step', '.v3 .autopass .status', '.v3 .decision-float .cap', '.v3 .phase .seg:not(.now)', '.v3 .combat-box .dim', '.v3 .stack-box .dim'].flatMap((s) => leg(s).slice(0, 1)),
    cards: cards.length, chipless: cards.filter((c) => !c.classList.contains('has-image') && !txt(c.querySelector('.chip'))).length,
    bands: bands.length, bandWrong: bands.filter((b) => txt(b) !== (b.closest('.card').dataset.name || '')).map((b) => txt(b)).slice(0, 4),
    bandClipped: bands.filter((b) => b.scrollHeight > b.clientHeight + 1 || b.scrollWidth > b.clientWidth + 1).length,
    counts: q('.v3 .card .count').filter(table).map((c) => { const a = c.getBoundingClientRect(), b = c.closest('.card').getBoundingClientRect();
      return a.width > 0 && a.left >= b.left - 1 && a.right <= b.right + 1 && a.top >= b.top - 1 && a.bottom <= b.bottom + 1; }),
    clipped: q(clipSel).filter((e) => e.scrollWidth > e.clientWidth + 1).map((e) => txt(e).slice(0, 40)), clipMeasured: q(clipSel).length,
  };
}"""


def offered_ids(v):
    """`TableV3`'s `offeredIds`, from the payload: every answerable non-mana action's object, plus declare `eligible`."""
    ids = set()
    for a in scn.actions(v):
        if a["kind"] == "TapForMana":
            continue
        if not (a["target_min"] <= 0 or not a["target_slots"] or any(s["candidates"] for s in a["target_slots"])):
            continue
        if a["object_id"] is not None:
            ids.add(a["object_id"])
        ids |= {e["id"] for e in ((a["attack"] or {}).get("eligible") or []) + ((a["block"] or {}).get("eligible") or [])}
    return ids


async def usability(T, v, tag):
    """The criteria every stop is held to (the mapping table above). Returns the page's measurements."""
    pg = T.pg
    await pg.mouse.move(1, 1)
    await pg.wait_for_timeout(250)
    u = await pg.evaluate(JS_USABILITY, RAW_IDENTIFIERS)
    m = await pg.evaluate(JS_NOSCROLL)
    V = f"verify[{tag}]"
    check(f"{V}: nothing scrolls (page, boards, cells, auto-pass panel)", m["boxes"] >= 5 and not m["bad"], f"{m['boxes']} boxes measured; {m['bad']}")
    check(f"{V}: every card has an image or a chip", u["cards"] > 0 and u["chipless"] == 0, f"{u['cards']} face-up cards, {u['chipless']} with neither")
    want = sorted(offered_ids(v) & set(u["present"]))
    check(f"{V}: the lit cards are exactly the cards the decision offers", u["lit"] == want, f"lit {u['lit']}, offered and on the table {want} ({len(u['present'])} cards on the table)")
    acts = {a["index"]: a for a in scn.actions(v)}
    idx = [i[0] for i in u["items"]]
    loose = [b for b in u["buttons"]]
    mana = [(i[2], i[3]) for i in u["items"] if i[1]]
    check(f"{V}: no decision option is drawn twice", len(set(loose)) == len(loose) and len(set(idx)) == len(idx) and all(i in acts for i in idx) and len(set(mana)) == len(mana),
          f"buttons {loose}; card items by action index {idx}; mana items {mana} for {sum(1 for a in acts.values() if a['kind'] == 'TapForMana')} mana actions")
    step = v["state"]["turn"]["step"]
    check(f"{V}: no raw engine identifier on screen", not u["raw"], f"found {u['raw']}" if u["raw"] else f"{len(RAW_IDENTIFIERS)} identifiers looked for in the surface's visible text")
    check(f"{V}: the phase track lights the current step, humanised", len(u["segNow"]) == 1 and SHORT_STEP_WORD[step] in u["segNow"][0].lower() and u["segs"] >= 12, f"step {step}: lit segment {u['segNow']} of {u['segs']}")
    combat = v["state"].get("combat")
    want_pairs = sorted((x["object_id"], tuple(sorted(b["object_id"] for b in x.get("blockers", [])))) for x in (combat or {}).get("attackers", []))
    got_pairs = sorted((p["attacker"], tuple(sorted(p["blockers"]))) for d in u["combat"]["defs"] for p in d["pairs"])
    words = u["combat"]["text"].lower()
    quiet = "not in combat" in words if not combat else ("no attackers declared" in words if not want_pairs else True)
    check(f"{V}: the combat card shows exactly the declared attackers", u["combat"]["box"] == 1 and got_pairs == want_pairs and quiet and (bool(want_pairs) or not u["combat"]["defs"]),
          f"card {got_pairs}, state.combat {want_pairs}; {u['combat']['text'][:80]!r}")
    check(f"{V}: no \"Lands: N\" counter that means land drops", not u["landsCounter"])
    check(f"{V}: the decision box and its answer are on screen and on top", bool(u["float"]) and u["float"]["inside"] and bool(u["answer"]) and u["answer"]["inside"] and u["answer"]["top"],
          f"box {u['float']}; answering control {u['answer']}")
    weak = [x for x in u["legible"] if x["px"] < 9 or x["contrast"] < 4.5]
    faint = "; fainter than 4.5:1 elsewhere (not asserted): " + ", ".join(f"{x['sel'].split(' ')[-1]} {x['contrast']}" for x in u["faint"] if x["contrast"] < 4.5) if any(x["contrast"] < 4.5 for x in u["faint"]) else ""
    check(f"{V}: dock text is legible (>= 9px, contrast >= 4.5:1)", len(u["legible"]) >= 4 and not weak,
          (f"weak: {weak}" if weak else f"{len(u['legible'])} texts, smallest {min(x['px'] for x in u['legible'])}px, lowest contrast {min(x['contrast'] for x in u['legible'])}") + faint)
    check(f"{V}: names are whole: cards, counts, dock text", u["bands"] > 0 and not u["bandWrong"] and all(u["counts"]) and u["clipMeasured"] > 0 and not u["clipped"],
          f"{u['bands']} name bands / chips all equal to the card's name ({u['bandClipped']} clipped by the two-line band; hover has them); {len(u['counts'])} land-stack counts inside their card; "
          f"{u['clipMeasured']} dock / track / button texts, clipped: {u['clipped']}; wrong: {u['bandWrong']}")
    return u


async def blk_verify(T):
    pg = T.pg
    FL = ".v3 .decision-float"

    async def pass_by_page(tag):
        await pg.locator(f"{FL} .prompt .buttons .btn.primary").click(timeout=2000)
        got = await T.wait_posts(1)
        p = T.posts[-1] if got else None
        check(f"verify[{tag}]: the decision is answered through the page (200)", got and len(T.posts) == 1 and p["status"] == 200 and (p["body"] or {}).get("params") in ({}, None),
              post_detail(p) if p else "no POST /api/game/action within 5 s")
        return p

    async def stack():
        r = await T.at("verify_stack")
        v = r.view
        wire = v["state"]["zones"]["stack"]
        u = await usability(T, v, "stack")
        await T.shot("verify-stack")
        top = wire[-1]
        rows = u["stack"]
        check("verify[stack]: the stack lists one row per stack object, top first", len(rows) == len(wire) and [x["name"] for x in rows] == [i["source_name"] for i in reversed(wire)],
              f"{[x['name'] for x in rows]} for wire (bottom first) {[i['source_name'] for i in wire]}")
        t = rows[0] if rows else {}
        check("verify[stack]: the top item is lit and says it resolves next", bool(rows) and [x["top"] for x in rows] == [True] + [False] * (len(rows) - 1) and "resolves next" in t["who"] and top["controller"] in t["who"]
              and t["edgeW"] >= 2 and t["edge"] != t["other"] and all("resolves next" not in x["who"] for x in rows[1:]),
              f"{t.get('who')!r} / {t.get('name')!r}; left edge {t.get('edgeW')}px {t.get('edge')} against {t.get('other')}")
        p = await pass_by_page("stack")
        after = ((p or {}).get("view") or {}).get("state", {}).get("zones", {}).get("stack")
        await pg.wait_for_timeout(300)
        shown = await pg.evaluate("() => [...document.querySelectorAll('.v3 .stack-box .stack-item .s-name')].map((e) => e.textContent.trim())")
        check("verify[stack]: after the pass the stack box follows the seat view", after is not None and shown == [i["source_name"] for i in reversed(after)], f"box {shown}, seat view {[i['source_name'] for i in (after or [])]}")
    await T.part("verify: the stack", stack)

    async def picker():
        r = await T.at("verify_picker")
        v = r.view
        a = next(x for x in scn.actions(v) if x["attack"] is not None)
        elig = a["attack"]["eligible"]
        opened = await T.wait_for(".v3 .picker-area .attacker-picker", 2500)
        u = await usability(T, v, "picker")
        await T.shot("verify-picker")
        check("verify[picker]: the attacker picker opens by itself, with every eligible attacker", opened and not T.posts and u["attackerPicker"] == 1 and sorted(u["toggles"]) == sorted(e["label"] for e in elig),
              f"open without a click {opened}; toggles {u['toggles']}, eligible {[e['label'] for e in elig]}")
        await pg.locator(".v3 .picker-area .picker-actions .cancel").click(timeout=2000)
        await pg.wait_for_timeout(900)
        closed = await T.n(".v3 .picker-area") == 0
        btns = await pg.locator(f"{FL} .prompt .buttons .btn").all_inner_texts()
        check("verify[picker]: Back closes it and it stays closed, with the decision's buttons in its place", closed and not T.posts and any(t.strip().startswith("Declare attackers") for t in btns), f"picker open after 900 ms: {not closed}; buttons {[t.strip() for t in btns]}")
        await pg.locator(f"{FL} .prompt .buttons .btn", has_text="Declare attackers").click(timeout=2000)
        again = await T.wait_for(".v3 .picker-area .attacker-picker", 1500)
        await pg.locator(".v3 .picker-area .attacker-toggle").first.click(timeout=2000)
        await pg.wait_for_timeout(150)
        label = await T.text(".v3 .picker-area .picker-actions .confirm")
        await pg.locator(".v3 .picker-area .picker-actions .confirm").click(timeout=2000)
        got = await T.wait_posts(1)
        p = T.posts[-1] if got else None
        sent = ((p or {}).get("body") or {}).get("params", {}).get("attackers") or []
        check("verify[picker]: the decision is answered through the page (200): a real attack", again and got and len(T.posts) == 1 and p["status"] == 200 and (p["body"] or {}).get("action_index") == a["index"]
              and [x[0] for x in sent] == [elig[0]["id"]], f"button {label!r}; " + (post_detail(p) if p else "no POST"))
        # the same game, one answer later: the human's attacker is in the combat card
        nxt = (p or {}).get("view")
        if nxt and nxt.get("decision") and (nxt["state"].get("combat") or {}).get("attackers"):
            await pg.wait_for_timeout(300)
            await usability(T, nxt, "picker, after the attack")
        else:
            could_not("verify[picker, after the attack]", "the answer to the attack carried no decision with attackers still declared")
    await T.part("verify: a picker", picker)

    async def combat():
        r = await T.at("verify_combat")
        v = r.view
        u = await usability(T, v, "combat")
        await T.shot("verify-combat")
        atts = v["state"]["combat"]["attackers"]
        tos = [d["to"] for d in u["combat"]["defs"]]
        want_to = {x["target"].split(":", 1)[1] for x in atts}
        me = scn.human(v)
        check("verify[combat]: one box per defending player, named", len(tos) == len(want_to) and all(any((("you" if n == me else n) in t) for t in tos) for n in want_to), f"{tos} for defenders {sorted(want_to)}")
        await T.click(await T.point(".v3 .combat-box .panel-title"))
        big = await T.wait_for(".v3 .combat-float", 1500)
        f = await pg.evaluate("""() => { const e = document.querySelector('.v3 .combat-float'); if (!e) return null; const r = e.getBoundingClientRect();
            return { inside: r.left >= 0 && r.top >= 0 && r.right <= innerWidth + 0.5 && r.bottom <= innerHeight + 0.5, lanes: e.querySelectorAll('.lane').length,
                     cards: [...e.querySelectorAll('.lane .card')].map((c) => Number(c.dataset.oid)), unblocked: e.querySelectorAll('.none').length }; }""")
        ids = sorted([x["object_id"] for x in atts] + [b["object_id"] for x in atts for b in x.get("blockers", [])])
        check("verify[combat]: the combat card enlarges to a lane per defender, inside the viewport", big and bool(f) and f["inside"] and f["lanes"] == len(want_to) and sorted(f["cards"]) == ids,
              f"{f}; attackers and blockers {ids}")
        await pg.keyboard.press("Escape")
        await pg.wait_for_timeout(200)
        check("verify[combat]: Escape closes the enlarged view and the decision is still there", await T.n(".v3 .combat-float") == 0 and await T.n(f"{FL} .prompt .buttons .btn.primary") == 1)
        await pass_by_page("combat")
    await T.part("verify: combat", combat)

    async def blocked():
        r = await T.at("block")
        blk = scn.block_action(r.view)
        b, t = blk["block"]["legal_blocks"][0]["blocker"]["id"], blk["block"]["legal_blocks"][0]["attackers"][0]["id"]
        try:
            nxt = scn.act(T.port, r.view["decision"]["seq"], blk["index"], {"blockers": [[b, t]]})     # the block itself is C2's `blockers` block; here it is set up
        except scn.Refused as e:
            raise NotReached(f"the block was refused: {e}")
        pairs = [(x["object_id"], [y["object_id"] for y in x.get("blockers", [])]) for x in (nxt["state"].get("combat") or {}).get("attackers", [])]
        if not nxt.get("decision") or (t, [b]) not in pairs:
            raise NotReached(f"no decision with the block still in combat after declaring it (combat {pairs})")
        await T.load()
        u = await usability(T, nxt, "combat, blocked")
        await T.shot("verify-combat-blocked")
        drawn = [p for d in u["combat"]["defs"] for p in d["pairs"] if p["attacker"] == t]
        check("verify[combat, blocked]: a blocker is drawn beside its attacker", len(drawn) == 1 and drawn[0]["blocked"] and drawn[0]["blockers"] == [b], f"{drawn} for {t} blocked by {b}")
    await T.part("verify: combat with a blocker", blocked)

    async def stop_line():
        await T.at("autopass_round")
        AP = ".v3 .autopass"
        idle = await T.text(f"{AP} .status")
        await pg.locator(f'{AP} input[data-stop="spell"]').set_checked(False, timeout=2000)
        await pg.locator(f'{AP} input[data-stop="ability"]').set_checked(False, timeout=2000)
        await pg.locator(f'{AP} input[data-stop="combat"]').set_checked(False, timeout=2000)
        await pg.locator(f'{AP} .mode[data-mode="eot"]').click(timeout=2000)
        settled = await T.idle(900, 60000)
        ran = len(T.posts)
        stopped = await T.text(f"{AP} .status")
        T.posts.clear()
        await pg.locator(f"{FL} .prompt .buttons .btn.primary").click(timeout=2000)
        await T.wait_posts(1)
        await pg.wait_for_timeout(300)
        after = await T.text(f"{AP} .status")
        check("verify: a finished auto-pass run's stop line clears once the human acts", settled and ran > 0 and stopped != idle and len(stopped) > len(idle) and after == idle and len(T.posts) == 1 and T.posts[0]["status"] == 200,
              f"idle {idle!r}; after {ran} passes {stopped[:120]!r}; after the human's own pass {after!r}")
    await T.part("verify: the auto-pass stop line", stop_line)


async def verify_flow(browser, a, base, w, h):
    """The verify block: one fresh browser context, the three stops of one seed-3 game and the two extras."""
    print("---- verify (scutemob-268): the -260 verify.py flow on v3 — a picker, the stack, combat", flush=True)
    errs = []
    ctx = await browser.new_context(viewport={"width": w, "height": h})
    pg = await ctx.new_page()
    T = Table(pg, base, a.port, errs, a.out)
    await T.route()
    try:
        await blk_verify(T)
    except NotReached as e:
        could_not("verify", str(e))
    except Exception as e:  # noqa: BLE001
        check("verify: the block ran to its end", False, f"{type(e).__name__}: {str(e).splitlines()[0][:260]}")
    await ctx.close()
    check("verify: zero page errors", not errs, str(errs[:5]))


async def main():
    global OUT_DIR
    ap = argparse.ArgumentParser()
    ap.add_argument("--images-only", action="store_true", help="run only the hover / card-window / image-store checks")
    ap.add_argument("--port", type=int, default=3041)
    ap.add_argument("--seed", type=int, default=3)
    ap.add_argument("--viewport", default="2560x1340")
    ap.add_argument("--out", default=".")
    ap.add_argument("--c2-only", action="store_true", help="skip the C1 checks; run only the C2 blocks")
    ap.add_argument("--verify-only", action="store_true", help="run only the verify flow (the -260 verify.py re-expressed for v3)")
    ap.add_argument("--blocks", default="", help="comma list of C2 blocks to run (default: all): " + ",".join(C2_BLOCKS))
    ap.add_argument("--c3-only", action="store_true", help="run only the C3 blocks (v3c3.py: the new seat-view fields, server-side pass-until)")
    ap.add_argument("--c3-blocks", default="", help="comma list of C3 blocks to run (default: all): " + ",".join(v3c3.C3_BLOCKS))
    a = ap.parse_args()
    if a.port == 3040:
        print("COULD-NOT-RUN 3040 is the owner's server"); sys.exit(2)
    w, h = (int(x) for x in a.viewport.split("x"))
    os.makedirs(a.out, exist_ok=True)
    OUT_DIR = a.out
    base = f"http://127.0.0.1:{a.port}"
    async with async_playwright() as p:
        b = await p.chromium.launch(executable_path="/usr/bin/chromium")
        if a.verify_only:
            await verify_flow(b, a, base, w, h)
            await b.close()
            sys.exit(0 if all(RESULTS) and RESULTS and not COULD_NOT else 1 if not all(RESULTS) else 2)
        if a.c3_only:
            await v3c3.run(sys.modules[__name__], b, a, base, w, h)
            await b.close()
            sys.exit(0 if all(RESULTS) and RESULTS and not COULD_NOT else 1 if not all(RESULTS) else 2)
        if a.c2_only:
            await c2(b, a, base, w, h)
            await b.close()
            sys.exit(0 if all(RESULTS) and RESULTS and not COULD_NOT else 1 if not all(RESULTS) else 2)
        ctx = await b.new_context(viewport={"width": w, "height": h})
        pg = await ctx.new_page()
        errs = []
        pg.on("pageerror", lambda e: errs.append(str(e)))
        scryfall = watch_scryfall(ctx)
        try:
            await pg.goto(f"{base}/?v3")
            await pg.wait_for_selector(".v3 .app", timeout=10000)
        except Exception as e:  # noqa: BLE001
            print(f"COULD-NOT-RUN {e}"); sys.exit(2)
        await pg.evaluate("(seed) => fetch('/api/game', {method:'POST', headers:{'content-type':'application/json'}, body: JSON.stringify({players: 4, seed})})", a.seed)
        await pg.reload()
        await pg.wait_for_selector(".v3 .cell", timeout=10000)
        await pg.click(".v3 .decision-float .btn.primary")  # Keep this hand
        await pg.wait_for_timeout(600)
        if a.images_only:
            await hover_checks(ctx, pg, base, w, h)
            check("no request to a Scryfall host", not scryfall, str(scryfall[:3]))
            check("zero page errors", not errs, str(errs))
            await b.close()
            sys.exit(1 if not all(RESULTS) else 2 if COULD_NOT else 0)

        # decision float: minimise sends it to the dock, the dock button brings it back
        check("decision opens in the centre", await pg.locator(".v3 .decision-float").count() == 1)
        await pg.click(".v3 .decision-float .mini-btn")
        await pg.wait_for_timeout(150)
        check("minimise sends it to the dock", await pg.locator(".v3 .decision-float").count() == 0
              and await pg.locator(".v3 .dock .prompt .btn.primary").count() == 1)

        cells = pg.locator(".v3 .cell:not(.cell-pop)")
        n = await cells.count()
        check("three opponents keep a fixed cell", n == 3, f"{n} cells")
        focused0 = await pg.evaluate("document.querySelector('.v3 .focused')?.dataset.seat ?? null")
        grey = pg.locator(".v3 .cell.focused-cell")
        check("exactly one cell is greyed out", await grey.count() == 1)
        op = await grey.evaluate("e => [getComputedStyle(e).opacity, getComputedStyle(e).filter, getComputedStyle(e.querySelector('.who'), '::after').content]")
        check("greyed cell is dimmed, desaturated and tagged", float(op[0]) < 0.6 and "grayscale" in op[1] and "across" in op[2], str(op))

        # click another cell: it comes across; the top copy is in addition to the cell
        other = pg.locator(".v3 .cell:not(.focused-cell):not(.cell-pop)").first
        other_name = await other.get_attribute("data-seat")
        box = await other.bounding_box()
        await pg.mouse.click(box["x"] + box["width"] - 6, box["y"] + box["height"] - 6)
        await pg.wait_for_timeout(200)
        focused1 = await pg.evaluate("document.querySelector('.v3 .focused')?.dataset.seat ?? null")
        check("clicking a cell brings that player across", focused1 == other_name and focused1 != focused0, f"{focused0} -> {focused1}")
        check("cells stay put", await cells.count() == 3)

        # pile popover from the 2x2 box; Escape closes
        await pg.click(".v3 .board .zonebox .pile >> nth=1")
        await pg.wait_for_timeout(150)
        title = await pg.locator(".v3 .popover .panel-title").first.inner_text() if await pg.locator(".v3 .popover").count() else ""
        check("a pile opens a popover", "library" in title.lower(), title.replace("\n", " "))
        await pg.keyboard.press("Escape")
        await pg.wait_for_timeout(100)
        check("Escape closes the popover", await pg.locator(".v3 .popover").count() == 0)
        await pg.click(".v3 .cell:not(.cell-pop) .zones5 .pile >> nth=1")
        await pg.wait_for_timeout(150)
        backs = await pg.locator(".v3 .popover .card.back").count()
        faces = await pg.locator(".v3 .popover .card:not(.back)").count()
        check("an opponent's hand pile lists backs only", backs > 0 and faces == 0, f"{backs} backs, {faces} face up")
        await pg.mouse.click(w // 2, 5)
        await pg.wait_for_timeout(100)
        check("an outside click closes the popover", await pg.locator(".v3 .popover").count() == 0)

        # seat right-click menu
        tgt = pg.locator(".v3 .cell:not(.focused-cell):not(.cell-pop)").first
        tgt_name = await tgt.get_attribute("data-seat")
        box = await tgt.bounding_box()
        await pg.mouse.click(box["x"] + box["width"] - 6, box["y"] + box["height"] - 6, button="right")
        await pg.wait_for_timeout(150)
        items = await pg.locator(".v3 .ctx button").all_inner_texts()
        check("right-click opens the seat menu", any("graveyard" in t.lower() for t in items) and any("across" in t.lower() for t in items), str(items))
        await pg.locator(".v3 .ctx button", has_text="across").click()
        await pg.wait_for_timeout(200)
        check("the menu brings the seat across", await pg.evaluate("document.querySelector('.v3 .focused')?.dataset.seat ?? null") == tgt_name)

        # hover overlay on a hand card, the card window fed over a storage event, and the image store (C3)
        await hover_checks(ctx, pg, base, w, h)

        # peek: one second on the cell itself grows it; resting on a card never arms it
        cell = pg.locator(".v3 .cell:not(.cell-pop)").first
        card = cell.locator(".zones5 .card").first
        await card.hover()
        await pg.wait_for_timeout(1300)
        check("peek does not arm while a card is hovered", await pg.locator(".v3 .cell-pop").count() == 0)
        box = await cell.bounding_box()
        await pg.mouse.move(box["x"] + box["width"] - 8, box["y"] + box["height"] - 8)
        await pg.mouse.move(box["x"] + box["width"] - 10, box["y"] + box["height"] - 10)
        await pg.wait_for_timeout(600)
        check("peek waits a full second", await pg.locator(".v3 .cell-pop").count() == 0)
        await pg.wait_for_timeout(700)
        pop = pg.locator(".v3 .cell-pop")
        opened = await pop.count() == 1
        pb = await pop.bounding_box() if opened else None
        anchored = opened and (abs(pb["x"] - box["x"]) < 3 or abs(pb["x"] + pb["width"] - box["x"] - box["width"]) < 3)
        check("peek grows the cell to double size in place", opened and abs(pb["width"] - min(box["width"] * 2, w * 0.5)) < 3 and anchored, f"cell {box}, pop {pb}")
        if opened:
            sc = await pop.evaluate("e => [e.scrollHeight, e.clientHeight]")
            check("the peeked cell's cards fit its box", sc[0] <= sc[1] + 1, str(sc))
            await pg.screenshot(path=os.path.join(a.out, "v3i-peek.png"))
        await pg.mouse.move(w // 2, 5)
        await pg.wait_for_timeout(150)
        check("peek closes when the pointer leaves", await pg.locator(".v3 .cell-pop").count() == 0)

        # sizing: auto fills without scrolling; the slider is authoritative and turns auto off
        fit = "(() => [...document.querySelectorAll('.v3 .board, .v3 .focused, .v3 .cell:not(.cell-pop)')].map(e => [e.className.split(' ')[0], parseFloat(e.style.getPropertyValue('--card')), e.scrollHeight > e.clientHeight + 1]))()"
        auto = await pg.evaluate(fit)
        centre = [x[1] for x in auto if x[0] in ("board", "focused")]
        side = [x[1] for x in auto if x[0] == "cell"]
        check("the two centre boards share one card size", len(centre) == 2 and centre[0] == centre[1], str(centre))
        check("all side cells share one card size", len(set(side)) == 1, str(side))
        check("auto mode: no box scrolls", not any(x[2] for x in auto), str(auto))
        await pg.locator(".v3 .top input[type=range]").fill("140")
        await pg.wait_for_timeout(200)
        big = await pg.evaluate(fit)
        auto_on = await pg.locator(".v3 .top input[type=checkbox]").first.is_checked()
        check("moving the slider turns auto-fit off and sets the size", (not auto_on) and all(x[1] == 140 for x in big if x[0] in ("board", "focused")), str(big))
        check("the page still does not scroll", await pg.evaluate("document.documentElement.scrollHeight === innerHeight && document.documentElement.scrollWidth === innerWidth"))
        await pg.locator(".v3 .top input[type=checkbox]").first.check()
        await pg.wait_for_timeout(200)
        back = await pg.evaluate(fit)
        check("auto-fit on again restores the measured fit", not any(x[2] for x in back) and all(x[1] <= 140 for x in back), str(back))
        pile = await pg.evaluate("[...document.querySelectorAll('.v3 .zonebox .pile .card, .v3 .zonebox .pile .empty')].map(e => Math.round(e.getBoundingClientRect().width))")
        handw = await pg.evaluate("[...document.querySelectorAll('.v3 .hand .card')].map(e => Math.round(e.getBoundingClientRect().width))")
        check("zone-box piles are a fixed 55px", bool(pile) and set(pile) == {55}, str(sorted(set(pile))))
        check("hand cards are a fixed 70px", bool(handw) and set(handw) == {70}, str(sorted(set(handw))))

        # re-open the decision from the dock
        await pg.locator(".v3 .dock .mini-btn", has_text="open").click()
        await pg.wait_for_timeout(150)
        check("the dock button re-opens the decision", await pg.locator(".v3 .decision-float").count() == 1)
        await pg.mouse.move(1, 1)
        await pg.screenshot(path=os.path.join(a.out, "v3i-end.png"))

        # ---- states a short real game does not reach: the server's own answer, rewritten on the way
        # in (GET /api/game and the reply to an action), so the page sees a seat view of the real shape.
        mut = {"fn": None}

        async def rewrite(route):
            resp = await route.fetch()
            if mut["fn"] is None or resp.status != 200:
                return await route.fulfill(response=resp)
            v = await resp.json()
            mut["fn"](v)
            await route.fulfill(status=200, content_type="application/json", body=json.dumps(v))
        await pg.route("**/api/game", rewrite)
        await pg.route("**/api/game/action", rewrite)

        async def seat(sel):
            return await pg.evaluate(f"document.querySelector('{sel}')?.dataset.seat ?? null")

        # a new decision re-opens a minimised box
        await pg.click(".v3 .decision-float .mini-btn")
        await pg.wait_for_timeout(120)
        was_min = await pg.locator(".v3 .decision-float").count() == 0
        await pg.keyboard.press("Space")  # pass priority: the reply carries the next decision
        await pg.wait_for_timeout(900)
        check("a new decision re-opens the minimised box", was_min and await pg.locator(".v3 .decision-float").count() == 1)

        # a turn change brings the new active player across, over a clicked focus
        across = await seat(".v3 .focused")
        others = [n for n in await pg.evaluate("[...document.querySelectorAll('.v3 .cell:not(.cell-pop)')].map(e => e.dataset.seat)") if n != across]
        real_active = await pg.evaluate("fetch('/api/game').then(r => r.json()).then(v => v.state.turn.active_player)")
        newcomer = next(n for n in others if n != real_active)

        def turn_to(v, who=newcomer):
            v["state"]["turn"]["active_player"] = who
        mut["fn"] = turn_to
        await pg.reload()
        await pg.wait_for_selector(".v3 .cell", timeout=10000)
        await pg.wait_for_timeout(500)
        check("a turn change brings the active player across", await seat(".v3 .focused") == newcomer, f"{across} -> {await seat('.v3 .focused')} (active {newcomer})")

        # the focused seat's grey-out lifts while blockers are being declared
        def blockers(v, who=newcomer):
            turn_to(v, who)
            if v.get("decision"):
                v["decision"]["kind"] = "DeclareBlockers"
                v["decision"]["actions"] = [x for x in v["decision"]["actions"] if x["kind"] in ("PassPriority", "Concede")]
        grey = await pg.evaluate("getComputedStyle(document.querySelector('.v3 .cell.focused-cell')).opacity")
        mut["fn"] = blockers
        await pg.reload()
        await pg.wait_for_selector(".v3 .cell", timeout=10000)
        await pg.wait_for_timeout(500)
        lifted = await pg.evaluate("(() => { const e = document.querySelector('.v3 .cell.focused-cell'); return e ? [e.classList.contains('lift'), getComputedStyle(e).opacity, getComputedStyle(e).filter] : null; })()")
        check("the grey-out lifts while blockers are declared", float(grey) < 0.6 and bool(lifted) and lifted[0] and float(lifted[1]) == 1 and lifted[2] == "none", f"before {grey}, during {lifted}")

        # a commander-zone choice: both answers carry the id of a card that is in a pile, not on the table
        def commander_choice(v):
            d = v.get("decision")
            if not d:
                return
            tmpl = dict(d["actions"][0])
            v["state"]["zones"]["graveyard"].setdefault("Human-1", []).insert(0, {"object_id": 9001, "name": "Isamaru, Hound of Konda", "card_types": ["Creature"], "hidden": False})
            acts = []
            for i, (k, l) in enumerate([("ReturnCommanderToCommandZone", "Return Isamaru, Hound of Konda to the command zone"), ("LeaveCommanderInZone", "Leave Isamaru, Hound of Konda where it is")]):
                x = dict(tmpl); x.update(index=i, kind=k, label=l, object_id=9001); acts.append(x)
            d["kind"], d["actions"] = "CommanderZoneChoice", acts
        mut["fn"] = commander_choice
        await pg.reload()
        await pg.wait_for_selector(".v3 .cell", timeout=10000)
        await pg.wait_for_timeout(500)
        btns = await pg.locator(".v3 .decision-float .prompt .buttons .btn").all_inner_texts()
        check("a commander-zone choice shows both answers as buttons", sum("command zone" in t for t in btns) == 1 and sum("where it is" in t for t in btns) == 1, str(btns))
        mut["fn"] = None
        check("no request to a Scryfall host", not scryfall, str(scryfall[:3]))
        check("zero page errors", not errs, str(errs))
        await ctx.close()

        await c2(b, a, base, w, h)
        await v3c3.run(sys.modules[__name__], b, a, base, w, h)
        await verify_flow(b, a, base, w, h)
        await b.close()
    print(f"---- {sum(RESULTS)} ok, {len(RESULTS) - sum(RESULTS)} BAD, {len(COULD_NOT)} could not run", flush=True)
    sys.exit(1 if not all(RESULTS) else 2 if COULD_NOT else 0)


if __name__ == "__main__":
    asyncio.run(main())
