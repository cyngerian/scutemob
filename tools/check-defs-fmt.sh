#!/usr/bin/env bash
#
# SR-35: the formatting gate for the card-definition corpus.
#
# `cargo fmt --all -- --check` reports success while checking ZERO of the 1,748
# files in `crates/card-defs/src/defs/`. This script is what actually checks them.
# Run it with no arguments to check; run it with `--fix` to reformat in place.
#
#
# WHY cargo fmt cannot see these files
# ------------------------------------
# rustfmt discovers files by walking `mod` declarations from each target's crate
# root. It does that walk *textually* — it never expands a macro and never runs a
# build script. `crates/card-defs/src/defs/mod.rs` is a single line:
#
#     include!(concat!(env!("OUT_DIR"), "/card_defs_generated.rs"));
#
# and the file that `include!` pulls in is written by `build.rs` at compile time,
# out in `target/`, as 1,748 `#[path = "<abs>"] pub mod <card>;` pairs. Both halves
# defeat the walk: the `include!` is a macro rustfmt will not expand, and the
# `#[path]` mods it would have found live in a generated file rustfmt is never
# pointed at. So the corpus is not "formatted and clean" — it is unvisited, and a
# green `cargo fmt --check` says nothing about it.
#
# That arrangement is deliberate (SR-6) and worth keeping: adding a card is adding
# one file, with no shared registry to collide on during parallel authoring. The
# fix is therefore to hand rustfmt the file list explicitly, which is all this
# script does.
#
#
# WHY the two --config flags (do not drop either)
# -----------------------------------------------
# Pointing rustfmt at the files is necessary but NOT sufficient. rustfmt has two
# failure modes here that both look exactly like success — it prints nothing and
# exits 0 while leaving a file completely unformatted:
#
#   format_strings=true
#     When rustfmt cannot fit an expression within `max_width`, it gives up on
#     that expression and emits the original source verbatim. That fallback
#     propagates to the *enclosing* expression. Nearly every def carries a long
#     `oracle_text: "…".to_string(),` line, and a string literal is something
#     rustfmt will not break up by default — so the fallback swallows the whole
#     `CardDefinition { … }` literal, which is the whole file. Measured with a
#     canary (inject a misindented `card_id` line, then ask rustfmt directly):
#     1,380 of 1,748 defs were INERT — rustfmt reported them clean while blind to
#     a real formatting error. `format_strings=true` lets rustfmt split long
#     strings across lines with `\` continuations, which fits them, which stops
#     the fallback. With it on, all 1,748 canaries are caught and 0 are inert.
#     The `\` continuation preserves the string's value (Rust strips the newline
#     and the following line's leading whitespace); this was verified card-by-card
#     by diffing the full `Debug` of `all_cards()` across the reformat — 1,719
#     files changed, byte-identical output.
#
#   error_on_line_overflow=true
#     The residual case: rustfmt formats a line, the result still exceeds
#     `max_width`, and it silently keeps the file's original text. This is the
#     skip named in SR-35 — it left 51 files unformatted during SR-33 until a long
#     `AddMana` line was split by hand. This flag turns that skip into exit 1
#     instead of silence. The corpus currently has ZERO such lines, so the gate is
#     a hard failure with no allowlist: if you add a def whose formatted output
#     overflows 100 columns, this fails and you split the line yourself.
#
# Both options are honored by the pinned stable toolchain (rust-toolchain.toml,
# rustfmt 1.9.0). They are passed on the command line rather than written into a
# `rustfmt.toml` on purpose: a `rustfmt.toml` at the workspace root would apply to
# the engine crates too, silently restyling ~300 files that are not this task.
#
# Non-vacuity is not asserted here, it is demonstrated:
# `crates/engine/tests/sr35_adversarial_demo.sh`.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
defs_dir="$repo_root/crates/card-defs/src/defs"

if [[ ! -d "$defs_dir" ]]; then
    echo "error: card-defs corpus not found at $defs_dir" >&2
    exit 2
fi

# `mod.rs` is the include! shim, not a card. rustfmt handles it fine either way,
# but keeping it out of the list keeps the count honest.
mapfile -t defs < <(find "$defs_dir" -maxdepth 1 -name '*.rs' ! -name 'mod.rs' | sort)

if [[ ${#defs[@]} -eq 0 ]]; then
    echo "error: no card defs found under $defs_dir — refusing to pass vacuously" >&2
    exit 2
fi

# The edition is not inferable from a bare file list (there is no Cargo.toml in
# play), so it is passed explicitly. It must track `[workspace.package] edition`.
args=(
    --edition 2021
    --config format_strings=true
    --config error_on_line_overflow=true
)

mode="${1:-check}"
case "$mode" in
    --fix | fix)
        echo "Reformatting ${#defs[@]} card defs..."
        rustfmt "${args[@]}" "${defs[@]}"
        echo "Done. Re-run without --fix to verify."
        ;;
    check | "")
        # Emitted before the verdict, and on both the pass and fail paths, so the
        # count is always observable. `the_fmt_gate_is_not_vacuous` parses it and
        # compares it against the directory listing: a gate that reports success
        # after checking nothing is the precise bug this script exists to fix.
        echo "card-defs fmt gate: ${#defs[@]} defs checked"
        if rustfmt "${args[@]}" --check "${defs[@]}"; then
            echo "clean"
        else
            cat >&2 <<EOF

--------------------------------------------------------------------------------
card-defs formatting gate FAILED (${#defs[@]} defs checked).

'cargo fmt --all -- --check' does NOT cover these files and will keep reporting
success — see the comment at the top of tools/check-defs-fmt.sh for why.

To fix:  tools/check-defs-fmt.sh --fix

'line formatted, but exceeded maximum width' means rustfmt could not fit a line
within 100 columns and would otherwise have silently left the file unformatted.
Split the line by hand; there is no allowlist.
--------------------------------------------------------------------------------
EOF
            exit 1
        fi
        ;;
    *)
        echo "usage: $(basename "$0") [check|--fix]" >&2
        exit 2
        ;;
esac
