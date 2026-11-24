# Plugin architecture

> **Last reviewed:** 2026-11-24 · **Last updated:** 2026-11-24 · **Change:** major (first plugin architecture doc) · **Target:** Autodesk AutoCAD 2027

How domain plugins extend CADCraft without touching its codebase. A plugin is an external Rust
crate that **implements two traits** (`Domain` for the engine and `DomainUi` for the shell) and is
linked by the host app at compile time. CADCraft itself never names a plugin; the plugin's own code
is the only place that knows its domain.

## Why plugins live outside cadcraft

The goal is **minimal interference with upstream**. The cadcraft repository is the shared CAD shell
that tracks `main` on storytold/cadcraft and receives updates from the upstream team. Plugins add
domain-specific behaviour (architecture, electrical, logical) **without modifying any file in the
cadcraft workspace**. This way:

- Merging `main` into a plugin branch never conflicts with engine or shell code.
- Each plugin team can iterate independently.
- The bare CAD builds and runs without any plugin.

## Layering

Plugins sit **above L5** in the cadcraft dependency graph. They depend downwards on cadcraft crates;
cadcraft never depends upwards on them.

```
  Plugin crate (arqelog-cadcraft, anacad-cadcraft)
    │
    ├─► cadcraft-engine        (L4)  — via cadcraft_engine::plugin (the public contract)
    ├─► cadcraft-ui-egui       (L5)  — via cadcraft_ui_egui::domain (the UI contract)
    ├─► cadcraft-doc           (L1)  — drawing types (Entity, Handle, Drawing)
    ├─► cadcraft-geom          (L0)  — geometry (Vec2, Vec3, Bounds2)
    ├─► cadcraft-color         (L0)  — colour (Color, Rgb)
    └─► egui                   (L5)  — the same version cadcraft uses (workspace patch)

  Host app (apps/cadcraft)       — Exempt from layer rules
    ├─► plugin crate (optional, feature-gated)
    ├─► cadcraft-engine
    └─► cadcraft-ui-egui
```

`cargo xtask layers` checks the cadcraft workspace; plugin crates are outside the workspace and are
the app's optional dependencies (feature-gated, `Class::Exempt`), so they don't appear in the layer
table and don't break the check.

## The two contracts

### Engine contract: `cadcraft_engine::plugin`

Everything a plugin needs from the engine is re-exported in
[`crates/engine/src/plugin.rs`](../crates/engine/src/plugin.rs). A plugin depends **only** on
this module (plus `cadcraft_doc`/`cadcraft_geom` for drawing types) and **never** on the engine's
internals (`pub(crate)` items). Changing `plugin.rs` is changing the plugins' API.

Provided re-exports:

| Item | Purpose |
|---|---|
| `CommandSpec`, helpers (`always`, `bad`, `f64_or`, …) | Defining commands |
| `Session`, `EngineError`, `Result` | Running commands |
| `Interactive`, `Prompt`, `Step`, `Input`, `Accept` | Interactive prompt machines |
| `Domain`, `DomainStates`, `registrar`, `registrados` | Registration and per-session state |
| `blocks::insert`, `blocks::insert_with` | Inserting blocks |
| `SelOutcome`, `SelectPhase` | Selection |
| `campo_xdata`, `marcar` | XData helpers (read/write domain data on entities) |

### UI contract: `cadcraft_ui_egui::domain`

The shell counterpart lives in
[`crates/ui-egui/src/domain.rs`](../crates/ui-egui/src/domain.rs). A plugin's UI module implements
the `DomainUi` trait and is registered with `cadcraft_ui_egui::domain::registrar`.

What `DomainUi` provides:

| Method | Purpose |
|---|---|
| `ui_commands()` | UI-only commands (menu items reachable as `ui.*`) |
| `ui_aliases()` | Short names that open UI commands |
| `tabs()`, `toolset(tab)` | Tool Sets palette tabs and groups |
| `toolbar()`, `toolbars()` | Tool Bar rows |
| `panels()`, `panel(panel, app, ui)` | Docking panels owned by the plugin |
| `dialogs(app, ctx)` | Per-frame dialog drawing |
| `begin_frame(app, ctx)` | Per-frame refresh (read drawing, update state) |
| `command_enabled(app, id)` | Grey out buttons when the plugin is not ready |
| `route(app, id, params)` | Run a `ui.*` command |
| `state()` | Per-app screen state (type-erased, like `DomainStates`) |

## Registration

The host app (`apps/cadcraft/src/main.rs`) registers plugins **once, before the first session**.
Both the engine module and the UI module are registered:

```rust
#[cfg(feature = "arqelog")]
cadcraft_engine::domain::registrar(&arqelog_cadcraft::ARQUITETURA);
#[cfg(feature = "arqelog")]
cadcraft_ui_egui::domain::registrar(&arqelog_cadcraft::ARQUITETURA_UI);
```

Registration is idempotent (by `Domain::id`). Feature gates (`anacad`, `arqelog`) make each plugin
fully optional: `cargo build --no-default-features` builds the bare CAD.

## Activation

Registered plugins are **inactive by default**. They become active through:

1. **Drawing header:** `CADCRAFT_PLUGINS` (comma-separated ids).
2. **XData probe:** if the header is absent, the engine scans entities for XData whose application
   name starts with the plugin's uppercased id (backward compatibility).
3. **User command:** `PLUGIN ATIVAR arqelog` or the short form `/arqelog`.

The active set lives on `Session::plugins_ativos` and is persisted on the drawing.

## Plugin repository layout

A plugin lives in a **sibling directory** (e.g. `../arqelog`, `../anacad`) with the same
conventions as cadcraft:

```
arqelog/
  crates/
    core/           ← domain logic, no cadcraft dependency
    data/           ← data model
    app/            ← application rules (connects core + data)
    cadcraft/       ← the integration crate (depends on cadcraft)
      src/
        lib.rs      ← re-exports the statics (ARQUITETURA, ARQUITETURA_UI, …)
        comum/      ← shared code for all three domains
        arq/        ← architecture domain config (Domain + DomainUi impls)
        ele/        ← electrical domain config
        log/        ← logical domain config
```

**Key rule:** `crates/core`, `crates/data` and `crates/app` know **nothing about cadcraft**. Only
`crates/cadcraft` depends on the cadcraft crates, and only through the public contracts.

## Rules for plugin authors

> [!IMPORTANT]
> These rules prevent the conflicts that arise when a plugin touches cadcraft internals.

1. **Depend only on the public contracts.** `cadcraft_engine::plugin` and
   `cadcraft_ui_egui::domain` — never import from `pub(crate)` modules or reach into `cmd/`
   submodules directly.

2. **Use the same dependency versions.** The cadcraft workspace patches `egui`, `eframe`, `winit`
   and `wgpu` to reviewed forks. Your plugin must use the same versions (match the workspace
   `Cargo.toml`'s `[patch.crates-io]` section and `[workspace.dependencies]`), or Cargo will pull
   two copies and types won't match.

3. **Never add files to the cadcraft workspace.** No new crates in `crates/`, no new modules in
   `cmd/`, no new `*.rs` in `ui-egui/src/`. Your code lives in your own repository.

4. **Respect the layer graph.** Your integration crate is above L5; it may depend on L0–L5 cadcraft
   crates but nothing in cadcraft depends on it. The host app links it as an optional feature.

5. **Commands are prefixed.** A plugin's command ids use its own prefix (`anacad.*`, `arq.*`,
   `ele.*`, `log.*`). Never add commands with bare ids that could collide with the shell's.

6. **State is type-erased.** Use `DomainStates::with::<YourType>(id, |state| …)` for engine state
   and `UiStates::state_mut::<YourType>(id)` for screen state. Never add fields to `Session` or
   `CadApp`.

7. **Icons are drawn in code** (the cadcraft convention). If your plugin needs icons, draw them
   with the `Icon` helpers, don't ship raster images.

8. **Panels go through commands.** A panel changes engine state only through `app.run(id, params)`,
   never by reaching into `Session` directly (AGENTS.md rule 14).

9. **Follow the cadcraft AGENTS.md.** Clean room, no panic, no unsafe, translations in English
   with the key system, accessibility rules — all apply to plugin code that runs inside cadcraft.

10. **Keep domain logic outside the integration crate.** Business rules, calculations, data models
    go in `crates/core` and `crates/app` (which don't depend on cadcraft). Only the glue that
    implements `Domain`/`DomainUi` goes in `crates/cadcraft`.

11. **Always use `..Default::default()` on entity and engine structs.** When instantiating `Insert`,
    `View`, `Common`, or other cadcraft types, always terminate the struct literal with
    `..Default::default()`. When upstream cadcraft introduces new fields (such as `clip` on `Insert`
    or `twist` on `View`), struct initializers without `..Default::default()` fail with `E0063`.

## Common mistakes that cause merge conflicts

| Mistake | Why it conflicts | Fix |
|---|---|---|
| Adding a module inside `crates/engine/src/cmd/` | Multiple PRs edit `cmd/mod.rs`; upstream refactors move commands | Put commands in your plugin crate, register them via `Domain::commands()` |
| Adding a file in `crates/ui-egui/src/` | Same: `lib.rs` and `menus.rs` change on every merge | Put UI in your plugin crate, register via `DomainUi` |
| Importing `pub(crate)` items from the engine | A refactor renames or moves them | Use only `cadcraft_engine::plugin::*` |
| Using a different `egui` version | Types from two `egui` versions are incompatible | Match the cadcraft workspace's patched version exactly |
| Initializing structs without `..Default::default()` | Upstream adding fields to `Insert`, `View`, etc. breaks compilation (`E0063`) | Always add `..Default::default()` in struct literals |
| Editing `AGENTS.md`, `Cargo.toml` or `docs/` in the cadcraft repo | These files change on every upstream merge | Edit your own repo's `AGENTS.md`; keep cadcraft's untouched |

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-11-24 | major | First committed plugin architecture doc |
