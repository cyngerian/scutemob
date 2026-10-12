#!/usr/bin/env python3
"""Fetch the sample cards' images into img/ (gitignored) so the mockups never
lean on Scryfall at view time. Polite: one request every 150 ms, retries on 429."""
import json, re, sys, time, urllib.request, urllib.error, pathlib
here = pathlib.Path(__file__).parent
names = sorted(set(re.findall(r"n: '((?:[^'\\]|\\.)*)'|n: \"([^\"]*)\"", (here / 'common.js').read_text())))
names = [a or b for a, b in names]
names = [n.replace("\\'", "'") for n in names]
out = here / 'img'; out.mkdir(exist_ok=True)
def slug(n): return re.sub(r'[^a-z0-9]+', '-', n.lower()).strip('-')
for n in names:
    for v in ('small', 'normal', 'large'):
        dest = out / f'{slug(n)}-{v}.jpg'
        if dest.exists(): continue
        url = f'https://api.scryfall.com/cards/named?exact={urllib.parse.quote(n)}&format=image&version={v}'
        for attempt in range(6):
            try:
                req = urllib.request.Request(url, headers={'User-Agent': 'scutemob-mockups/1', 'Accept': 'image/*'})
                with urllib.request.urlopen(req, timeout=20) as r: dest.write_bytes(r.read())
                print('ok', dest.name); break
            except urllib.error.HTTPError as e:
                if e.code == 429: time.sleep(3 * (attempt + 1)); continue
                print('FAIL', n, v, e.code); break
            except Exception as e:
                print('FAIL', n, v, e); break
        time.sleep(0.15)
