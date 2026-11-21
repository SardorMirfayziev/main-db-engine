# rustdb — project rules

Learning project: build a database from scratch in Rust, following the book
"Build Your Own Database From Scratch". Learner is a Rust beginner.

## HARD RULE (not bypassable): a slice = ONE meaningful goal

A slice must deliver **one meaningful, observable capability** — a real thing a
user or test can hold and check (e.g. "build an empty leaf", "build a leaf with
one pair"). A slice is **NEVER** a plumbing helper function with no standalone
meaning (e.g. `push_u16_le`, `push_bytes`). Helpers may exist **inside** a slice,
but they do not get their own slice or ticket.

Test of a valid slice: can you state its goal as a behaviour the whole thing now
does, and prove it on a real result? Proof is NOT a test (D16 — no tests). Claude
checks the behaviour — by reading the code and, if useful, a one-off `cargo run` —
and confirms it in chat. If the only "goal" is "a helper that another function
calls", it is NOT a slice — fold it into the slice that needs it.

This rule overrides any urge to ticket helpers separately. No exceptions.

### Sub-slices (how we build each slice — for the beginner)

Never write one big chunk of Rust to reach a slice. Break each slice into
**logically separated vertical sub-slices**, built in order. A sub-slice:
- **writes AND reads** something observable — a real round-trip (e.g. "write a
  leaf, read its pair count back"), never a plumbing helper. No test code; Claude
  confirms the round-trip works (D16 — no tests);
- adds the next thin layer on the sub-slice below it.

A helper (e.g. `push_u16_le`) is still NOT a sub-slice — it lives inside one.
The slice is done when all its sub-slices pass. Sub-slices are a learning aid for
a Rust beginner, not a licence to ticket helpers.

### Minimum only — NO generalizing ahead (hard guard)

A slice builds **only** the behaviour it needs. NO loop over many items, NO sorting,
NO extra case borrowed from a future slice. If a later slice will need a loop, the
later slice adds it — not this one.

Why this guard exists: the first slice ("a leaf remembers one key") was built with a
loop over all pairs. So the next slice ("a leaf remembers many keys") had no new code
left to write at all. That defeats the point of slicing. Every subagent prompt
MUST repeat this rule: implement THIS slice's behaviour and nothing more.

### A sub-slice must add NEW code — never re-do already-built behaviour (hard rule)

Before writing or implementing any sub-slice, CHECK the current code (e.g.
`src/leaf.rs`): does an existing function or method already do this? If yes, it is
**NOT a sub-slice**. Do not create it, do not spawn a subagent for it. Fold its
check into the sub-slice that adds the new code, as one confirmation of that
slice's result.

A sub-slice earns its own file/ticket ONLY if it writes new code. A step that is
"call a function that already works and confirm the result" is a check line, not a
sub-slice.

Concrete: "read the value back with `get`" is never its own sub-slice once `get`
exists — it is one confirmation inside the sub-slice that wrote the new data. Check
first, then slice.

## Core build rule: minimum vertical slice, spec by spec

We do NOT follow the book's bottom-up order. We build in **vertical slices**.

- A **vertical slice** = the smallest end-to-end path that delivers ONE real,
  demoable behaviour (store a value, read it back, …).
- Each slice has **one purpose**. It contains **only** the layout, values,
  structs, constants, and functions that THIS slice actually uses. If the slice
  does not use it, it does not exist yet. No "we will need it later" code.
- **Minimum layout, minimum values, minimum code.** Strip every field, byte,
  type, or helper the current behaviour does not touch.
- Each new spec **builds on top of what is already built** — never rewrites the
  foundation, just adds the next thin layer on the working code below it.
- Flow: one **spec** (the design) → one **ticket** (the slice) → implement →
  next spec builds on it.

## Tickets / specs

- Source of truth = the **Obsidian vault** at
  `~/Developer/Books Vault/Db-engine/` (`Roadmap.md` + `slices/` + `decisions.md`).
  GitHub Issues are NOT used.
- Keep it tiny: ideally **one phase in focus + one slice** open at a time.
- Each slice is one folder under `slices/` with an `overview.md` plus its
  vertical sub-slice files.

## Rust style (we are learning real Rust — D14)

- **No Rust-feature restriction (D14).** Use any Rust that fits: structs,
  `impl`/methods, traits, generics, closures, iterator chains, `?`, `&mut`. We are
  here to learn real Rust. Write it the way a real engineer would (D15); run the
  `improve-codebase-architecture` skill per slice if useful. No "simplest Rust
  only" rule anymore.
- **Long, fully descriptive names** (e.g. `read_child_page_number_at`), so the name
  explains itself. KEEP THIS — it is a learning aid, not a feature restriction.
- Short `//` comments on what each part does.
- Little-endian for all on-disk / on-wire integers.

## No tests (D16)

- **No test code.** No `#[test]`, no `#[cfg(test)] mod tests`, no `cargo test` as
  the gate. A slice is proven by Claude reading the code and confirming the
  round-trip works, with a one-off `cargo run` if useful.
- Accepted risk: no automatic regression net, so a later slice can silently break
  an earlier one. Claude must re-check earlier behaviour when a slice touches
  shared code.

## Code structure (D15)

- **No fixed file/module rule.** Organize the growing code the way a real engineer
  would — split by real part when it helps, keep together when it does not.
- Optionally run the `improve-codebase-architecture` skill after a slice to find
  module-design improvements.

## Done = update the vault (D17)

- A slice is DONE only when: its box on the `Roadmap.md` ladder is ticked AND one
  dated line is added to the `decisions.md` changelog. The vault is the sole source
  of truth (D4), so it must always show how far the DB is built.

## Git

- **No AI attribution** in commits or PRs (no `Co-Authored-By`, no "Generated
  with Claude"). This overrides any default footer.

## Reference

- Private repo `SardorMirfayziev/rustdb-reference` holds a fuller section-4
  implementation to draw ideas from — read for ideas, do not paste blindly.
