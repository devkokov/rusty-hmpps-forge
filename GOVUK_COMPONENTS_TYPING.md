# GOV.UK components: do we get IntelliSense for params?

## Short answer

**No.** Component parameters are plain dynamic `Value` objects assembled with the `obj! { "key": ... }`
macro. `rustc`/rust-analyzer check that `obj!`'s *syntax* is valid, but they know nothing about which
keys a given component accepts, what type each value should be, or which keys are required. A typo'd
key, a wrong type, or a missing required field all compile fine and silently do nothing (or panic at
render time).

## How it actually works today

1. **Authoring side** (`examples-app`, any consumer): a component is invoked as

   ```rust
   component(&govuk::TEXT_INPUT, obj! {
       "code": "postcode",
       "label": obj! { "text": "Postcode" },
       "hint": obj! { "text": "Try SW1A 1AA" },
   })
   ```

   `obj!` (`forge-core/src/authoring/mod.rs`) is a `macro_rules!` that turns `"key": value` pairs into
   an ordered `Map<Arc<str>, Value>` — it's a JSON object literal, not a struct literal. `component()`
   (`examples-app/src/guide/dsl.rs:512`) accepts `impl IntoValue`, so the macro's only job is producing
   a `Value`.

2. **Component side** (`forge-govuk-components`): each variant is a `fn(&Value) -> Value` that reads
   fields back out by string key, e.g.

   ```rust
   fn back_link(p: &Value) -> Value {
       params! { "href" => p.get("href"), "text" => text_unless_html(p, "text", "html"), ... }
   }
   ```

   `params!` (`forge-govuk-components/src/lib.rs:27`) is the macro the user was asking about — it's a
   declarative macro (not a proc macro) that builds the *output* `Value` passed to the Nunjucks
   template. It has the same properties as `obj!`: no connection to a schema, no compile-time field
   list.

3. **`Component`** (`forge-core/src/authoring/mod.rs:696`) is just `{ variant: &'static str, field: bool,
   prepare: Option<fn(Value) -> Value> }`. `TEXT_INPUT`, `BUTTON`, `HEADING`, etc. are `const` values of
   this one type — there is no `TextInput` / `Button` / `Heading` type, so there's nothing for
   rust-analyzer to hang per-component fields off of.

4. **`Schema`** (`forge-core/src/schema.rs`) exists and *some* field components are registered with
   `.with_input(schema)` (`field_entry`, `lib.rs:120`), but this is a runtime (Zod-like) validator used
   for journey input validation — it has no effect on authoring-time type checking or editor
   completion.

Net effect: writing `"lable"` instead of `"label"`, or passing a `String` where an object with
`{ text, html }` is expected, is invisible until you run the journey (or notice the rendered HTML is
wrong). There is no autocomplete for valid keys, no "go to definition" from a key to its meaning, and no
compiler error for missing required fields.

## Why it's built this way

- `Value` is the lingua franca across the whole engine (JSON-equivalent — same type system used for
  condition expressions, journey answers, and Nunjucks template context), so every builder in
  `forge-core::authoring` produces/consumes `Value`. Components follow that convention for consistency.
- The params functions in `forge-govuk-components` are deliberately 1:1 ports of the upstream
  `govuk-frontend` Nunjucks macros' JS normalisation logic (see module doc comment). Keeping params as
  loosely-typed `Value` mirrors the dynamic JS objects the TS version passed around, which made porting
  behaviour mechanically verifiable against the JS source.

## A more "Rust-y" alternative: typed prop structs

Since `component()` already accepts `impl IntoValue` (`examples-app/src/guide/dsl.rs:512`) and
`IntoValue` is a public trait (`forge-core/src/authoring/mod.rs`), nothing about the plumbing needs to
change. We can add one concrete struct per component, implement `IntoValue` for it, and keep the dynamic
`Value` core as the wire format underneath. Authors get real IntelliSense and compiler-checked calls;
the renderer keeps working with `Value` exactly as it does today.

```rust
#[derive(Default)]
pub struct TextInputProps {
    pub code: String,                 // `impl Into<String>` at construction, not `&'static str` — some
    pub label: Label,                 // call sites build codes/labels at runtime (loops, formatted ids)
    pub hint: Option<Hint>,
    pub classes: Option<String>,
    pub autocomplete: Option<String>,
    pub formatters: Vec<Value>,       // escape hatch for the parts that stay dynamic (conditions, transformers)
    pub valid_when: Vec<Value>,
}

impl IntoValue for TextInputProps {
    fn into_value(self) -> Value {
        // Key order here is fixed by this function, not by the caller's struct-literal field order
        // (Rust struct literals are positional-irrelevant) — see "Does this preserve JSON
        // serialisation?" below for why that order still matters.
        obj! {
            "code": self.code,
            "label": self.label,
            "hint": self.hint,
            "classes": self.classes,
            "autocomplete": self.autocomplete,
            "formatters": self.formatters,
            "validWhen": self.valid_when,
        }
    }
}

// call site:
component(&govuk::TEXT_INPUT, TextInputProps {
    code: "postcode".into(),
    label: Label::text("Postcode"),
    hint: Some(Hint::text("Try SW1A 1AA")),
    ..Default::default()
})
```

What this buys:

- **Autocomplete** on `TextInputProps { ... }` — rust-analyzer lists every valid field.
- **Typos become compiler errors** (unknown field / `E0560`) instead of silently-ignored keys.
- **Required vs optional is explicit** in the type (`code: &str` vs `hint: Option<Hint>`), so a missing
  required field is a compiler error, not a runtime surprise.
- **Defaults** come for free via `#[derive(Default)]` + struct update syntax (`..Default::default()`),
  which is arguably nicer than the current `obj!` of only the keys you want to override.
- **No breaking change to the renderer**: `params!`/`entry`/`field_entry`/`ComponentEntry` all keep
  operating on `Value`; only the authoring-facing surface changes.
- **Incremental adoption**: `IntoValue` is implemented per-type, so typed structs and the existing
  `obj!` literal can coexist component-by-component while the rest of the codebase migrates.

### Does this preserve JSON serialisation of the forge config?

**Yes — by construction, with one migration caveat.** This was worth checking explicitly because the
whole Forge config (`Journey`/`Step`/`Block`/`Field` definitions) is JSON on the wire: `Value` has a
hand-written `Serialize`/`Deserialize` impl (`forge-core/src/value.rs:444`), `Map` is an `IndexMap`
built with `serde_json`'s `preserve_order` feature (`forge-core/Cargo.toml:13,17`), and multiple test
suites (`forge-core/tests/contracts.rs`, `forge-govuk-components/tests/recorded.rs`,
`forge-govuk-components/tests/jsx.rs`) replay or diff recorded JSON fixtures byte-for-byte.

Why typed structs don't threaten this:

- Every `IntoValue` impl — existing or new — ends by producing a plain `Value`. `component()` only ever
  calls `v(props)` → `IntoValue::into_value()` (`examples-app/src/guide/dsl.rs:512`), so no matter how
  props were authored (a `Value`, an `obj!` literal, or a typed struct), what gets stored in the
  Journey tree, traced by devtools (`forge-devtools/src/channel.rs:41`), and fed into `Schema`/Zod
  generation (`forge-core/src/engine/dsl/zod.rs`) is the same kind of `Value` as today. Typed structs
  are an authoring-time convenience layer in front of serialisation, not a replacement for it.
- `Option<T>`'s existing blanket impl (`None` → `Value::Undefined`) already matches the serializer's
  behaviour of dropping undefined object keys (`value.rs:460-465`), so `Option` fields round-trip
  exactly like an omitted `obj!` key does today — nothing to change there.
- An enum only needs `impl IntoValue for MyEnum { fn into_value(self) -> Value { Value::from(self.as_str()) } }`,
  which produces the identical `Value::String` the raw string field produces today — the JSON shape
  (and whatever consumes it) can't tell the difference between `"type": "submit"` written by hand and
  `"type": ButtonType::Submit.into_value()`.
- Typed props structs never need their own `Serialize`/`Deserialize`: authored definitions are built
  once in Rust and converted to `Value` outbound; nothing in the codebase deserializes JSON back into
  an authored `Component`/props type (only plain data — answers, fixtures, trace events — round-trips
  through `Value`/`serde_json::Value`). So this stays a one-way `IntoValue` conversion, no `serde`
  derive needed on the structs themselves.

The one real subtlety: **key order inside an object is semantically meaningful**, not just cosmetic.
`forge-core/src/authoring/mod.rs:2` notes block definitions are ported "key order included (block keys
decide AST node IDs)" — `NodeFactory`/`compile_ast` (`forge-core/src/engine/ast.rs:256`) walks each
authored object's keys in `IndexMap` insertion order and hands out sequential `compile_ast:N` ids to
whatever it finds, including nested blocks (e.g. a checkbox item's `conditional` block, an accordion
item's nested content). Change the order of keys in an object that contains nested blocks and you
change which descendant gets which id — which is exactly what the recorded fixtures snapshot.

What this means in practice:

- Converting a **leaf-only** component's props (no nested blocks among its fields — e.g. `BUTTON`,
  `TEXT_INPUT` outside its `validWhen` conditions) to a typed struct is safe regardless of field order,
  because there are no descendant node ids to perturb.
- Converting a component whose props **embed other components** (radio/checkbox `items[].block`,
  accordion/tabs items, summary list row `actions`) means the typed struct's `into_value()` should
  reproduce the existing call sites' key order, at least for the keys that hold nested blocks, or
  expect to regenerate fixtures deliberately (`UPDATE_FIXTURES=1`) and review the id diff rather than
  being surprised by it.
- This isn't unique to typed structs — hand-written `obj!` has exactly the same hazard today (reorder
  two keys in a hand-written `obj!` call and descendant ids already shift). If anything, typed structs
  **improve** on this: `into_value()` fixes the key order once per component, so every call site using
  `TextInputProps { .. }` produces identical key order no matter what order the author lists the
  struct's fields in (Rust struct-literal field order has no effect on the type's layout or behaviour)
  — today, two hand-written `obj!` calls for the "same" component can disagree on key order and
  silently produce different ids for equivalent-looking definitions.
- IDs (`compile_ast:N`) are recomputed fresh on every compile from the authored definition
  (`forge-core/src/engine/ast.rs:250`) and aren't persisted across deployments or used as externally
  stable keys for stored answers (those are keyed by field `code`, not node id) — so this is a
  **test-fixture stability concern**, not a production data-compatibility risk.

### `nullish`/`or` become `Option` combinators — mostly

The current code has two JS fallback helpers operating on `Value`:

```rust
fn or(a: Value, b: Value) -> Value {        // JS `a || b` (truthy fallback)
    if a.truthy() { a } else { b }
}
fn nullish(a: Value, b: Value) -> Value {   // JS `a ?? b` (nullish fallback)
    if a.is_nullish() { b } else { a }
}
```

In the typed world, `nullish` maps directly onto `Option`, because `Option::None` *is* "nullish" and
`Option::Some(x)` *is* "defined" — there's no information loss:

```rust
// before: nullish(p.get("id"), p.get("code"))
id: Option<&'static str>,
...
let id = self.id.or(self.code);           // Option::or
// or, if `code` is always required and non-optional:
let id = self.id.unwrap_or(self.code);
```

`or` (`||`, truthy fallback) is **not** a safe `Option` replacement, though. JS truthiness treats `""`,
`0`, `false`, and `NaN` as "fall through to the default" even though they're explicitly-provided values,
not absent ones. `Option` only has "absent" (`None`) vs "present" (`Some(_)`), so `self.rows.or(Some(5))`
would keep an explicit empty string instead of defaulting it, where JS's `rows || 5` would default it.
Anywhere the upstream macro uses `||` on a value that could legitimately be falsy-but-set (e.g.
`rows || '5'` in the character count macro), the typed struct needs either a plain default (no
`Option` at all — just `rows: u32` with a `Default` impl) or an explicit truthiness check, not a bare
`Option::or`.

Trade-offs to weigh before committing to this:

- It's **one struct per component variant** (~30+ here), each needing to track the upstream
  `govuk-frontend` macro's param shape — more code to keep in sync, though it would catch drift that
  currently only a test suite notices.
- Nested shapes (`label: { text, html }`, `items: [{ value, text, checked, conditional }]`) need their
  own small typed builders too, or you lose the benefit for anything but top-level fields.
- Conditions/transformers/validations are expression trees (`Value`) by design and would stay as `Value`
  fields on the struct (an intentional escape hatch, not a gap).
- `Schema`-driven runtime validation (`with_input`) and the typed struct would describe the same shape
  twice unless one is generated from the other.

## Which fields are good enum candidates?

Several component params are drawn from a small, known, upstream-documented set of strings rather than
being free text. These fall into two groups.

**Group A — already string-matched in our own Rust code.** Today these use `.as_str() == "..."` /
`match ... { "xl" | "l" => ... }` with a fallback arm for anything else, which silently swallows typos.
An enum makes the match exhaustive (compiler forces every variant to be handled) and removes the
stringly-typed comparison entirely:

| Field | Component(s) | Current code | Proposed enum |
|---|---|---|---|
| `size` | JSX `heading` (`jsx.rs:95-104`) | `match size.as_str() { "xl" \| "l" => ..., "m" => ..., "s" => ..., _ => ... }` | `HeadingSize { Xl, L, M, S }` |
| `style` | JSX `list` (`jsx.rs:133`) | `if style.as_str() == Some("number") { "ol" } else { "ul" }` | `ListStyle { Bullet, Number }` |
| `size` | JSX `section_break` (`jsx.rs:153-158`) | raw string interpolated into `govuk-section-break--{size}` | `SectionBreakSize { Xl, L, M, S }` (`Option<...>`, no modifier when `None`) |
| `width` | JSX `grid_row` columns (`jsx.rs:170-176`) | raw string interpolated into `govuk-grid-column-{width}` | `GridColumnWidth { OneQuarter, OneThird, OneHalf, TwoThirds, ThreeQuarters, Full, OneSixth, FiveSixths, ... }` |

**Group B — pure pass-through today**, forwarded verbatim to the upstream Nunjucks macro without any
Rust-side interpretation, but still documented as a closed set by `govuk-frontend`:

| Field | Component(s) | Upstream values |
|---|---|---|
| `buttonType` | `BUTTON` (`lib.rs:175`) | `"submit"` \| `"button"` \| `"reset"` |
| `bannerType` | `NOTIFICATION_BANNER` (`lib.rs:282`) | `"success"` (anything else = default banner) |
| `role` | `NOTIFICATION_BANNER` (`lib.rs:282`) | `"region"` \| `"alert"` |
| `inputType` | `TEXT_INPUT` (`lib.rs:406`) | `"text"` \| `"email"` \| `"tel"` \| `"number"` \| `"password"` \| `"search"` \| `"url"` |
| `inputMode` | `TEXT_INPUT` (`lib.rs:407`) | `"none"` \| `"text"` \| `"decimal"` \| `"numeric"` \| `"tel"` \| `"search"` \| `"email"` \| `"url"` |
| `headingLevel` / `titleHeadingLevel` | `ACCORDION`, `PANEL`, `NOTIFICATION_BANNER` (`lib.rs:148,282,296`) | `1`–`6` |

For Group B, an enum doesn't change any runtime behaviour — the renderer still just needs the same
string — but it moves the "is this a legal value" question from "read the GOV.UK Design System docs and
hope" to "the type only has these variants". Example:

```rust
#[derive(Clone, Copy, Default)]
pub enum ButtonType {
    #[default]
    Submit,
    Button,
    Reset,
}

impl ButtonType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Submit => "submit",
            Self::Button => "button",
            Self::Reset => "reset",
        }
    }
}

impl IntoValue for ButtonType {
    fn into_value(self) -> Value {
        Value::from(self.as_str())
    }
}
```

`HeadingLevel` is a good case for a plain `enum H1..H6` rather than a raw `u8`/`NonZeroU8` + range
check, since GOV.UK only ever accepts 1–6 and an enum makes out-of-range values a compile error instead
of a runtime validation failure (today caught, if at all, by the `Schema` for field components, but not
for `headingLevel` specifically).

Fields that **don't** fit an enum well: `classes` (free-form, user-composed CSS class list — see below),
`attributes` (arbitrary HTML attribute map), `autocomplete` (HTML autocomplete tokens — a large,
evolving spec-defined set, not worth hard-coding), and anything that's actually prose (`text`/`html`,
error messages, labels).

One nuance: `classes::tag::GREEN` and friends (`classes.rs`) *look* like enum candidates (GOV.UK tag
colours, label/fieldset sizes, etc.) but they're currently just `&str` constants spliced into a
free-form `classes` string field alongside arbitrary user classes (e.g. `classes::input::WIDTH_10`
combined with a custom class). Turning tag colour into a real `TagColour` enum would mean giving `TAG`
its own dedicated `colour: Option<TagColour>` field (separate from `classes`), which is a bigger,
more deliberate restructuring than adding an enum to an existing closed-set field — doable, but a
separate decision from the Group A/B changes above.

### Recommendation

Given the number of components and the goal of mechanical parity with the upstream Nunjucks macros, a
full rewrite isn't obviously worth it. A pragmatic middle ground: introduce typed prop structs (as above)
for the **most commonly hand-written / error-prone** components first (`TEXT_INPUT`, `RADIO_INPUT`,
`CHECKBOX_INPUT`, `SELECT_INPUT`, `BUTTON`, `SUMMARY_LIST`), keep `obj!`/`Value` for the rest, and let
`IntoValue` bridge both styles at `component()` call sites without touching `forge-govuk-components`'
internals.

## Further idiomatic-Rust suggestions

A few smaller refinements worth folding in if/when this gets built:

- **Share one `HeadingLevel` enum**, not one per component. `headingLevel` (accordion, panel) and
  `titleHeadingLevel` (notification banner) are the same closed set (1–6) — a single
  `forge_govuk_components::HeadingLevel` used across all three avoids three near-identical enums.
- **Prefer `Display` over a hand-rolled `as_str()`** for the string-valued enums (`ButtonType`,
  `InputType`, `ListStyle`, …), then write `IntoValue` in terms of `to_string()`. It's one trait instead
  of an ad hoc method name per enum, and it's what `{}`/`format!`/logging already expect.
- **Consider the `strum` crate** (not currently a dependency) for `#[derive(Display, EnumString)]` on
  these enums — it removes the boilerplate `match` in each `as_str`/`Display` impl across ~10 enums.
  Small enough to hand-write too; only worth adding if the enum count grows past the first few
  components.
- **Avoid `&'static str` in prop structs** (the initial sketch used it for brevity) — some props are
  built dynamically (formatted ids, loop-generated field codes, data pulled from a CMS/config at
  runtime), so `String` (or `impl Into<String>` at construction, e.g. via a `new()`/builder) is the
  right default; reserve `&'static str` for genuinely compile-time-constant things like the enum
  variants' wire names.
- **Weigh `#[non_exhaustive]` on the enums** against GOV.UK Design System's own stability: these mirror
  a third-party spec (HTML `autocomplete`/`inputmode` tokens, GOV.UK's size/colour scales) that can grow
  new values upstream without a breaking change on our side. `#[non_exhaustive]` plus a documented
  "unmapped values fall back to `Value::from(raw string)`" escape hatch avoids a hard compile break the
  next time `govuk-frontend` adds e.g. a new tag colour, at the cost of one wildcard arm wherever the
  enum is matched exhaustively today (there are none yet, since matching currently happens in
  `forge-govuk-components`' own `match size.as_str() { ... }`, which already has a fallback arm).
- **Small typed sub-builders, not just top-level prop structs.** `Label`/`Hint`/`Fieldset` style
  `{ text, html, classes }` shapes repeat across almost every field component; one small shared struct
  (with its own `IntoValue`) used from every prop struct avoids re-deriving the same `text`/`html`
  mutual-exclusivity logic (`text_unless_html`, `text_param`) per component.
- **Optional ergonomic builder methods** (`TextInputProps::new(code, label).hint(..).classes(..)`) on
  top of the plain struct, for the common case of setting 2–3 fields — struct-update syntax
  (`..Default::default()`) works today, but a `with_*`/builder API reads better at call sites that only
  ever set a handful of the ~15 fields a component can have.

## Findings from the button proof of concept

Steps 1–3b are implemented. What we learned:

- **`Component::build` is now generic** (`build(&self, props: impl IntoValue)`), so typed props and
  `Value`/`obj!` both work. This is backward-compatible because `Value` implements `IntoValue`.
- **`ButtonProps` covers `BUTTON` and `LINK_BUTTON`.** `ButtonKind` is `Link { href }` or
  `Action { name, button_type, value, disabled, prevent_double_click }`, mirroring the
  `href`-based branch in `button()` (`forge-govuk-components/src/lib.rs`).
- **Typed fields hold concrete `String`/`bool`/`Option` only.** Values that are expressions (`Ex`, such
  as `format(...)`, `post(...)`, `params`) cannot go in typed fields, so those call sites stay `obj!`.
  This is the main limitation of the typed-struct approach. The fix is to keep an explicit escape hatch
  or accept `obj!` for dynamic cases.
- **Framework keys are not button props.** `visibleWhen`, `validate`, `onSubmission`, `effects`,
  `next`, `goto`, `onValid`, and `onAccess` are engine block/hook keys
  (`forge-core/src/engine/runtime/pipeline.rs`, `forge-core/src/engine/semantic.rs`). So there is no
  need for a typed field per framework key. Step 4 does not require them.
- **Call-site conversion:** 91 `BUTTON`/`LINK_BUTTON` sites across about 78 `examples-app` `.rs`
  files were converted. `cargo test -p forge-examples-app` passes with no fixture changes.
- **Guides:** `GOVUK_COMPONENTS_TYPING.md` examples now use the typed API. The developer guides under
  `examples-app/content` were updated, and static-text buttons were converted. Dynamic buttons
  (`format`, `post`, `params`) and `visibleWhen` buttons keep `obj!`.
- **Open items:** a render-level check that `button()` output is unchanged for typed props, and
  whether `moj::BUTTON_MENU` is in scope (assumed out of scope because it is not a GOV.UK button).

## Summary

- No IntelliSense today; typed prop structs (implementing the existing `IntoValue`) would add it
  without touching the renderer or breaking JSON serialisation.
- `Option` is a safe, drop-in replacement for `nullish` (`??`); it is **not** a safe replacement for
  `or` (`||`), which needs an explicit default/truthiness check instead.
- Several fields (button/input/list/grid-width/heading-size/level) are good enum candidates; a few
  (`classes`, `attributes`, `autocomplete`) are not.
- The typed-struct approach is JSON-serialisation-safe by construction; the only thing to be careful
  about when migrating an *existing* call site is preserving key order for props that embed nested
  component blocks, since that order currently drives AST node-id allocation (a test-fixture concern,
  not a production-data-compatibility one).

## Migration plan: small iterative steps

Each step below is independently shippable, keeps `obj!`/`Value` call sites that haven't been touched
yet working exactly as today, and should leave `cargo test -p forge-core -p forge-govuk-components
-p examples-app` green with **no unexpected fixture diffs** (`UPDATE_FIXTURES=1` should produce an
empty or fully-explained diff — if it doesn't, stop and investigate before continuing). New code lives
in `forge-govuk-components` (a `props` module) unless noted otherwise.

1. **[DONE] Add shared sub-shapes, additive only.**
   Add `forge_govuk_components::props::{Text, Label, Hint, Fieldset, ErrorMessage}` (or similar), each
   implementing `IntoValue`, mirroring the existing `text_param`/`text_unless_html`/`fieldset`/
   `error_message` render-side helpers so authored output matches what those helpers already expect.
   No call sites change. Verify: new unit tests only (`assert_eq!(Label::text("x").into_value(), obj!
   { "text": "x" })`); existing test suites untouched and still pass.

2. **[DONE] Add the Group A/B enums, additive only.**
   Add `ButtonType`, `InputType`, `InputMode`, `ListStyle`, `HeadingSize`, `SectionBreakSize`,
   `GridColumnWidth`, `HeadingLevel` (shared across accordion/panel/notification banner),
   `NotificationBannerType`, `NotificationBannerRole`, each with `Display`/`IntoValue`. No call sites or
   render functions change yet. Verify: unit test per enum pinning its wire string(s) (e.g.
   `ButtonType::Submit.to_string() == "submit"`) so a later rename is caught immediately.

3. **[DONE] Proof of concept: one leaf component, `BUTTON`.**
   Add `ButtonProps` (uses `ButtonType` from step 2), implement `IntoValue` reproducing
   `Component::build(&govuk::BUTTON, ...)`'s current shape/key order exactly. Migrate the handful of
   `govuk::BUTTON` call sites in `examples-app` to it. `BUTTON` has no nested blocks, so this step
   carries no AST-id risk — good first real migration to validate the pattern end-to-end (including
   that rust-analyzer autocomplete actually shows up as expected). Verify: full test suite, no fixture
   changes expected at all.

3b. **[DONE] Convert the existing `BUTTON` call sites to `ButtonProps`.**
   Step 3 only adds the type and proves it works. This step does the bulk conversion: `grep` finds
   about 68 `govuk::BUTTON.build(obj! { ... })` call sites across about 57 files in `examples-app`
   (plus any `component(&govuk::BUTTON, obj! { ... })` forms). Convert them mechanically, keeping
   each `obj!` key set and value identical so the authored JSON is unchanged. Split the work into
   a few commits by directory if the diff is large. Verify: `cargo test -p examples-app` passes and
   `UPDATE_FIXTURES=1` produces no diff.

4. **Roll out to the other leaf field components, one per step.**
   Repeat step 3's pattern for `TEXT_INPUT`, `TEXTAREA`, `SELECT_INPUT`, `PASSWORD_INPUT`,
   `CHARACTER_COUNT` — each as its own small change (its own prop struct, its own call-site migration,
   its own test run). These have no nested blocks in their own fields (aside from `validWhen`/
   `formatters`, which stay as `Vec<Value>` escape hatches per the design above), so each step is
   low-risk and reviewable independently. Verify per step: no fixture changes expected.

5. **Roll out to simple non-field leaf components.**
   Same pattern for `HEADING`, `BODY`, `LIST`(JSX), `TAG`, `INSET_TEXT`, `DETAILS`, `WARNING_TEXT`,
   `BACK_LINK`, `BREADCRUMBS`, `PANEL` — bringing in `HeadingSize`/`HeadingLevel`/`ListStyle` from step
   2 where relevant. Verify per step: no fixture changes expected.

6. **Compound components with nested blocks — one at a time, deliberately.**
   `RADIO_INPUT`, `CHECKBOX_INPUT` (items can carry a `conditional` block), `ACCORDION`, `TABS`
   (item panels), `SUMMARY_LIST` (row actions). For each: write the prop struct's `into_value()` to
   reproduce the *existing* call sites' key order for any field holding nested blocks, migrate one
   component's call sites, then run fixtures with `UPDATE_FIXTURES=1` and **read the diff** — it should
   be empty; if `compile_ast:N` ids shift anywhere, that's a signal the key order doesn't match and
   needs fixing before merging, not a diff to accept blindly.

7. **(Optional/stretch) Teach the render side to parse the enums back, for Group A fields only.**
   In `jsx::heading`/`jsx::list`/`jsx::section_break`/`jsx::grid_row` (and `lib.rs` wherever a param is
   both authored via an enum *and* string-matched in our own Rust), parse the incoming `Value::String`
   back into the enum (`HeadingSize::from_str`, with a documented fallback for unrecognised/legacy
   strings, since render input is still dynamic `Value` and can't be guaranteed to come from the typed
   authoring path). This turns today's `match ... { "xl" | "l" => ..., _ => ... }` into an exhaustive
   match over enum variants plus one explicit "unrecognised" fallback arm — a readability win, not a
   correctness requirement, so it's fine to defer or skip.

8. **(Optional/stretch) Dedicated `colour`/size fields instead of `classes` string-splicing.**
   E.g. give `TAG` a `colour: Option<TagColour>` field (`classes.rs`'s tag colour constants already
   enumerate the valid set) instead of requiring `classes: "govuk-tag--blue"`. This is a genuine
   behavioural/API change (not just an additive typed wrapper), so treat it as its own proposal with
   its own call-site migration and changelog entry, separate from steps 1–7.

9. **Wrap up.**
   Once the "most commonly hand-written" components (per the Recommendation above) are migrated,
   update any developer-facing docs/examples that still show the raw `obj!` form for those components,
   and record in `CHANGELOG.md` that typed prop structs are now the recommended authoring style
   (while `obj!`/`Value` remains fully supported for anything not yet migrated, or for the dynamic
   escape-hatch fields like `validWhen`/`formatters`/`attributes`).

Ordering rationale: steps 1–2 are pure additions (cannot break anything that exists), steps 3–5 migrate
the lowest-risk call sites to validate the approach and build confidence before touching anything with
nested blocks, step 6 is where the one real hazard (AST id drift via key order) lives and gets handled
with explicit fixture review, and steps 7–9 are polish/optional follow-ups that don't block getting
value from steps 1–6.
