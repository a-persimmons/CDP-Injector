# CDP Injector Product Workspace Navigation Design

## Decision

CDP Injector uses one application workspace plus one settings page. The current
three-column screen is the workspace for the selected Electron application,
not a global dashboard.

The first usable release keeps the built-in Codex Product Profile and proves
the real launch, Product Session, CDP, and theme-injection path. Generic
Electron application import remains gated until that path is accepted.

## Information architecture

### Application workspace

The left rail and right Product Session inspector remain visible while the
center area switches between:

- Modules: modules installed for the selected application;
- Module detail: version, targets, declared capabilities, errors, and removal;
- Diagnostics: Product Session, CDP target, module phase, and error evidence.

The right inspector always describes the selected application. Import dialogs
and restart confirmation are modal flows rather than pages.

### Settings

Settings is the only separate first-level page. It manages CDP Injector itself,
not Electron applications. The Product Session inspector is hidden on this
page.

### Application management

Application management is a dedicated page reached from the compact icon in
the Product selector. It lists built-in and user-added Product Profiles and
supports add, inspect, re-detect, and remove. Built-in Codex cannot be removed.

## Product selector

The selector is one visual row with three independent controls:

```text
[application icon] [current application name] [manage applications] [switch]
```

- Clicking the icon, name, or chevron opens only the imported-application list.
- Clicking an application switches the active Product and reloads Modules,
  Diagnostics, and the Product Session inspector for it.
- Clicking the compact management icon opens Application Management directly.
- The management icon and switch control have separate hover, focus, tooltip,
  accessible name, and event boundaries.
- The switch list contains no add or management commands.

Before generic application import exists, Codex is the only switch-list item.

## Application management behavior

- The page header contains one `+ Add application` primary action.
- Clicking an application row enters its management detail; it does not switch
  the active workspace application.
- Switching applications remains exclusive to the Product selector.
- Re-detect refreshes compatibility evidence without replacing the saved
  profile until the user confirms.
- Removing a user-added application requires confirmation and does not delete
  the Electron application itself.

## Add application dialog

The same modal flow is used wherever application addition is invoked:

```text
choose .app
-> detect Electron and CDP compatibility
-> show name, path, version, process, and target evidence
-> confirm
-> persist Product Profile
-> refresh application lists
-> close dialog
```

Application selection alone is insufficient. A saved Product Profile requires
validated launch, process, preview/restart, and CDP target-matching data. The
flow rejects unsupported applications instead of creating a profile that only
appears usable.

## Scope order

1. Connect the existing workspace to real Codex launch and restart handling.
2. Create the Rust-owned Product Session and CDP connection.
3. Inject and remove the built-in theme in the real Codex renderer.
4. Stop for user verification of the direct operation path.
5. Only after acceptance, implement Application Management and generic Product
   Profile import using the interaction contract above.

Taskboard migration, marketplace, cross-platform profiles, and compatibility
layers remain outside this checkpoint.

## State and errors

- React reads Product views from Tauri commands and holds only selected view,
  dialog, and busy state locally.
- Rust owns Product Profiles, selected Product persistence, sessions, and
  compatibility evidence.
- Long-running launch phases remain visible in the right inspector.
- Restart consequences are stated before confirmation.
- Product and module failures remain scoped and never imply that unaffected
  modules failed.
- Failed application detection leaves no partial Product Profile on disk.

## Verification

- Product selector click targets do not trigger each other.
- Switching a Product refreshes all three workspace regions.
- Settings hides the Product Session inspector and returns to the previous
  Product workspace.
- Add Application success is atomic; failure creates no list entry.
- Codex can launch normally with no modules and through CDP with the theme.
- Generic import work does not begin until the real Codex theme path is
  accepted.
