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
does, and prove it with one test on a real result? If the only "goal" is "a
helper that another function calls", it is NOT a slice — fold it into the slice
that needs it.

This rule overrides any urge to ticket helpers separately. No exceptions.

### Sub-slices (how we build each slice — for the beginner)

Never write one big chunk of Rust to reach a slice. Break each slice into
**logically separated vertical sub-slices**, built in order. A sub-slice:
- **writes AND reads** something observable — a real round-trip with its own test
  (e.g. "write a leaf, read its pair count back"), never a plumbing helper;
- adds the next thin layer on the sub-slice below it.

A helper (e.g. `push_u16_le`) is still NOT a sub-slice — it lives inside one.
The slice is done when all its sub-slices pass. Sub-slices are a learning aid for
a Rust beginner, not a licence to ticket helpers.

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

## Rust style (learner is a beginner)

- **Simplest Rust only.** `while` loops, `.unwrap()`, index slicing. NO traits,
  generics, closures, iterator chains, or the `?` operator.
- **Long, fully descriptive function names** (e.g. `read_child_page_number_at`),
  so the name explains itself.
- Short `//` comments on what each part does.
- Little-endian for all on-disk / on-wire integers.

## Git

- **No AI attribution** in commits or PRs (no `Co-Authored-By`, no "Generated
  with Claude"). This overrides any default footer.

## Reference

- Private repo `SardorMirfayziev/rustdb-reference` holds a fuller section-4
  implementation to draw ideas from — read for ideas, do not paste blindly.
