# Headless interaction check for the v3 play surface (`/?v3`): the things the greedy sweep
# (v3sweep.py) never touches. One game, one viewport; each check prints ok / BAD:
#   focus by click, focused cell greyed + "across", pile popover (open, Escape closes),
#   seat right-click menu (+ "Bring across the table"), one-second peek (arms on the cell,
#   never on a card; closes on leave), hover overlay, size slider turns auto-fit off and
#   boards may then scroll, auto toggle restores the fit, decision minimise / re-open,
#   the ?card-window route receiving a hover over a storage event.
# Needs a play-server on 127.0.0.1 (never the owner's 3040):
#   cargo run -p play-server -- --port 3041 --players 4 --bot heuristic --seed 0
# Run:  python3 tools/play-server/probes/v3interact.py --port 3041 --seed 3 --out /tmp/v3i
# Exit 0 = every check ok, 1 = a check is BAD, 2 = COULD-NOT-RUN (no server / no table).
import argparse, asyncio, os, sys
from playwright.async_api import async_playwright

RESULTS = []


def check(name, ok, detail=""):
    RESULTS.append(ok)
    print(("ok   " if ok else "BAD  ") + name + (f" — {detail}" if detail else ""))


async def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, default=3041)
    ap.add_argument("--seed", type=int, default=3)
    ap.add_argument("--viewport", default="2560x1340")
    ap.add_argument("--out", default=".")
    a = ap.parse_args()
    if a.port == 3040:
        print("COULD-NOT-RUN 3040 is the owner's server"); sys.exit(2)
    w, h = (int(x) for x in a.viewport.split("x"))
    os.makedirs(a.out, exist_ok=True)
    base = f"http://127.0.0.1:{a.port}"
    async with async_playwright() as p:
        b = await p.chromium.launch(executable_path="/usr/bin/chromium")
        ctx = await b.new_context(viewport={"width": w, "height": h})
        pg = await ctx.new_page()
        errs = []
        pg.on("pageerror", lambda e: errs.append(str(e)))
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

        # hover overlay on a hand card, and the card window fed over a storage event
        cw = await ctx.new_page()
        await cw.goto(f"{base}/?card-window")
        await cw.wait_for_selector(".v3-cardwin", timeout=5000)
        hand = pg.locator(".v3 .hand .card").first
        name = await hand.get_attribute("data-name")
        await hand.hover()
        await pg.wait_for_timeout(300)
        vis = await pg.evaluate("(() => { const e = document.querySelector('.v3 .hover-card'); if (!e) return null; const r = e.getBoundingClientRect(); return [getComputedStyle(e).display, r.left, r.top, r.right, r.bottom, e.querySelector('img')?.src ?? '']; })()")
        check("hover shows the one overlay, inside the viewport", bool(vis) and vis[0] != "none" and vis[1] >= 0 and vis[2] >= 0 and vis[3] <= w and vis[4] <= h and "version=normal" in vis[5], str(vis))
        await cw.wait_for_timeout(300)
        cw_text = await cw.locator(".v3-cardwin").inner_text()
        cw_src = await cw.evaluate("document.querySelector('.v3-cardwin img')?.src ?? ''")
        check("the card window shows the hovered card", bool(name) and name in cw_text and "version=large" in cw_src, f"{name!r}")
        await cw.close()
        await pg.mouse.move(w // 2, 5)
        await pg.wait_for_timeout(100)
        check("leaving the card hides the overlay", await pg.evaluate("getComputedStyle(document.querySelector('.v3 .hover-card')).display") == "none")

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
        check("peek grows the cell to double size in place", opened and abs(pb["width"] - min(box["width"] * 2, w * 0.5)) < 3 and abs(pb["x"] - box["x"]) < 3 or (opened and abs(pb["x"] + pb["width"] - box["x"] - box["width"]) < 3), f"cell {box}, pop {pb}")
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
        check("zero page errors", not errs, str(errs))
        await b.close()
    sys.exit(0 if all(RESULTS) else 1)


asyncio.run(main())
