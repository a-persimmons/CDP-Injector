# CDP Injector Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a standalone macOS `CDP Injector.app` that launches Codex through one Rust-owned CDP session and injects an enabled theme plus the packaged Taskboard module.

**Architecture:** A Tauri 2 application uses React for the launcher UI and Rust for Product discovery, process lifecycle, CDP transport, package installation, and session orchestration. Renderer modules are prebuilt `.cdpmod` packages; modules with HTTP APIs run through a Node binary bundled inside the application. The existing Taskboard remains in this repository and becomes the first service-backed module.

**Tech Stack:** Tauri 2, Rust 2021, Tokio, reqwest, tokio-tungstenite, React, TypeScript, Vite, pnpm, bundled Node.js 22+, macOS application bundles.

**Repositories:** Create a standalone sibling repository named `cdp-injector`. Keep Taskboard-specific module source in the existing `dashi-taskboard` repository.

**Project rule:** Follow the current repository rule: implement and demonstrate the direct operation path first. Do not add broad regression suites, compatibility layers, marketplace behavior, or speculative safeguards before the user confirms the feature works. Boundary validation for imported packages and loopback HTTP remains required.

---

## Direct operation path to prove before implementation

Present this concrete path to the user before changing application code:

```text
React Product card
-> Tauri launch_product command
-> Rust ProductSession allocates one CDP port
-> Rust launches Codex with remote-debugging flags
-> Rust reads /json/list and connects matched renderer targets
-> Rust installs one composite document-start source
-> theme CSS visibly changes Codex
```

For Taskboard, extend the same path:

```text
launch_product
-> ModuleService starts packaged service/index.mjs
-> GET /health succeeds
-> module context receives opaque serviceUrl
-> injected Taskboard entry opens its iframe
-> task mutation reaches packaged HTTP API and module data directory
-> changed card is visible in Codex
```

The first path is the initial success checkpoint. Do not begin Taskboard migration until the user has seen the theme path operate in Codex.

### Task 1: Bootstrap the standalone Tauri repository

**Files:**
- Create repository: `../cdp-injector/`
- Create: `../cdp-injector/package.json`
- Create: `../cdp-injector/src/main.tsx`
- Create: `../cdp-injector/src/App.tsx`
- Create: `../cdp-injector/src/styles.css`
- Create: `../cdp-injector/src-tauri/Cargo.toml`
- Create: `../cdp-injector/src-tauri/tauri.conf.json`
- Create: `../cdp-injector/src-tauri/src/main.rs`
- Create: `../cdp-injector/src-tauri/src/lib.rs`
- Create: `../cdp-injector/README.md`

**Step 1: Create the repository**

Run from the parent directory:

```bash
pnpm create tauri-app cdp-injector
```

Choose React, TypeScript, pnpm, and Tauri 2. Initialize Git with branch `main`.

**Step 2: Reduce the generated application to one screen**

`src/App.tsx` should render only:

```tsx
export function App() {
  return (
    <main>
      <h1>CDP Injector</h1>
      <p>CDP 注入器</p>
    </main>
  );
}
```

Remove generated counters, demo commands, logos, and unused styles.

**Step 3: Configure the macOS application**

Set the Tauri product name and bundle identifier:

```json
{
  "productName": "CDP Injector",
  "identifier": "dev.cdp-injector.desktop"
}
```

Use one 980 x 680 main window. Add tray support but no tray actions yet.

**Step 4: Verify the shell**

Run:

```bash
pnpm tauri dev
```

Expected: a macOS window displays `CDP Injector` and `CDP 注入器`.

**Step 5: Commit**

```bash
git add .
git commit -m "feat: bootstrap CDP Injector app"
```

### Task 2: Define Product, Module, and Session state

**Files:**
- Create: `../cdp-injector/src-tauri/src/model.rs`
- Create: `../cdp-injector/src-tauri/src/state.rs`
- Create: `../cdp-injector/src-tauri/resources/products/codex.json`
- Modify: `../cdp-injector/src-tauri/src/lib.rs`
- Modify: `../cdp-injector/src-tauri/tauri.conf.json`

**Step 1: Add the minimum serializable models**

Define:

```rust
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductProfile {
    pub id: String,
    pub name: String,
    pub application_paths: Vec<String>,
    pub process_names: Vec<String>,
    pub contexts: Vec<TargetContext>,
    pub preview: PreviewCapability,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled_for: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductStatus {
    pub product_id: String,
    pub phase: String,
    pub module_errors: std::collections::BTreeMap<String, String>,
}
```

Keep session maps inside a single `AppState` guarded by `tokio::sync::Mutex`.
Do not introduce repositories, service traits, or dependency injection.

**Step 2: Add the built-in Codex profile**

Translate the approved Codex Product Profile from the design document into
`resources/products/codex.json` and include it as a Tauri resource.

**Step 3: Persist only enabled-module selections**

Use one JSON file under Tauri's application config directory:

```json
{
  "enabledModules": {
    "codex": ["dev.cdp-injector.codex-theme"]
  }
}
```

Use write-to-temporary-file then rename. Do not add SQLite to CDP Injector.

**Step 4: Add a direct model check**

Add one Rust test that parses `codex.json` and asserts `id == "codex"` and
`preview.supported == false`.

Run:

```bash
cd src-tauri && cargo test product_profile_parses
```

Expected: one passing test.

**Step 5: Commit**

```bash
git add src-tauri
git commit -m "feat: add CDP Injector domain state"
```

### Task 3: Expose the real Product list to React

**Files:**
- Create: `../cdp-injector/src-tauri/src/commands.rs`
- Modify: `../cdp-injector/src-tauri/src/lib.rs`
- Create: `../cdp-injector/src/api.ts`
- Modify: `../cdp-injector/src/App.tsx`
- Modify: `../cdp-injector/src/styles.css`

**Step 1: Add Tauri commands**

Expose only:

```rust
#[tauri::command]
pub async fn list_products(state: tauri::State<'_, AppState>) -> Result<Vec<ProductView>, String>;

#[tauri::command]
pub async fn set_module_enabled(
    product_id: String,
    module_id: String,
    enabled: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String>;
```

`ProductView` combines the Product Profile, persisted selection, and current
status. Do not create a generic command bus.

**Step 2: Add the frontend wrapper**

`src/api.ts` calls Tauri `invoke` directly and exports TypeScript types matching
the serialized Rust structures.

**Step 3: Render one Product card**

Render Codex with:

- icon and name;
- `不支持预览` badge;
- built-in theme switch;
- launch button placeholder;
- current status text.

Do not add navigation, marketplace tabs, accounts, or search.

**Step 4: Verify selection persistence**

Run `pnpm tauri dev`, toggle the theme, quit CDP Injector, and reopen it.

Expected: the theme remains enabled.

**Step 5: Commit**

```bash
git add src src-tauri
git commit -m "feat: show configurable Codex product"
```

### Task 4: Launch and terminate Codex on macOS

**Files:**
- Create: `../cdp-injector/src-tauri/src/product.rs`
- Modify: `../cdp-injector/src-tauri/src/commands.rs`
- Modify: `../cdp-injector/src-tauri/src/lib.rs`
- Modify: `../cdp-injector/src/api.ts`
- Modify: `../cdp-injector/src/App.tsx`

**Step 1: Resolve the installed application**

Check configured absolute `.app` paths in order. Return a specific error if no
candidate exists.

**Step 2: Detect an existing process**

Use `/usr/bin/pgrep -x` with each configured process name. Keep this macOS-only
implementation direct; do not add a cross-platform process abstraction.

**Step 3: Implement normal launch**

When no module is enabled, run:

```text
/usr/bin/open -a <resolved-app-path>
```

**Step 4: Implement injected relaunch**

When at least one module is enabled:

1. return `restartRequired` if Codex is already running;
2. after UI confirmation, request normal termination with `/usr/bin/osascript`
   using the resolved application name;
3. poll `pgrep` until the process exits;
4. fail with `请手动退出 Codex 后重试` rather than force-killing it;
5. allocate one loopback CDP port;
6. launch with `open -n -a ... --args --remote-debugging-port=<port>` and the
   matching `--remote-allow-origins` argument.

**Step 5: Wire the UI confirmation**

The launch button calls `prepare_launch`. If it returns `restartRequired`, show
exactly the Product Profile message and call `launch_product` only after user
confirmation.

**Step 6: Verify the process path**

With no module enabled, launch Codex and confirm no injection UI appears. Then
enable the theme, accept the restart notice, and confirm Codex starts with a
reachable `http://127.0.0.1:<port>/json/version` endpoint shown in Injector's
status details.

**Step 7: Commit**

```bash
git add src src-tauri
git commit -m "feat: launch Codex with an application CDP session"
```

### Task 5: Implement the Rust CDP client and target matcher

**Files:**
- Create: `../cdp-injector/src-tauri/src/cdp.rs`
- Create: `../cdp-injector/src-tauri/src/session.rs`
- Modify: `../cdp-injector/src-tauri/Cargo.toml`
- Modify: `../cdp-injector/src-tauri/src/lib.rs`

**Step 1: Add only required crates**

Add `reqwest`, `tokio-tungstenite`, `futures-util`, `serde`, `serde_json`,
`thiserror`, and `uuid`. Reuse Tokio from Tauri. Do not add a CDP framework.

**Step 2: Implement command-response correlation**

`CdpConnection` owns one WebSocket, an incrementing request ID, pending response
senders, and an event broadcast channel. It exposes:

```rust
pub async fn send(
    &self,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, CdpError>;
```

**Step 3: Discover and filter targets**

Fetch `/json/list`, require `type == "page"`, and apply the Product context's
URL prefixes and exclusions. Keep the current two-second target polling model.

Add this deliberate simplification in code:

```rust
// ponytail: polling matches the proven Taskboard launcher; use Target.setDiscoverTargets if latency matters.
```

**Step 4: Retain target connections in ProductSession**

Store connections by target ID. Remove closed targets and connect new matching
targets. Do not let modules create their own connections.

**Step 5: Run a direct CDP probe**

Launch Codex through CDP Injector and invoke `Runtime.evaluate` with `1 + 1` on
the matched main target.

Expected result: `2` appears in the CDP Injector log.

**Step 6: Commit**

```bash
git add src-tauri
git commit -m "feat: connect Codex renderer targets over CDP"
```

### Task 6: Add the renderer runtime and built-in theme

**Files:**
- Create: `../cdp-injector/src-tauri/resources/runtime/cdp-injector.js`
- Create: `../cdp-injector/builtin-modules/codex-theme/manifest.json`
- Create: `../cdp-injector/builtin-modules/codex-theme/inject/index.css`
- Create: `../cdp-injector/src-tauri/src/injection.rs`
- Modify: `../cdp-injector/src-tauri/src/session.rs`
- Modify: `../cdp-injector/src-tauri/tauri.conf.json`

**Step 1: Implement the minimal page runtime**

The runtime stores registered definitions and cleanup functions:

```js
(() => {
  if (globalThis.cdpHub?.apiVersion === 1) return;

  const definitions = new Map();
  const cleanups = new Map();

  globalThis.cdpHub = {
    apiVersion: 1,
    register(definition) {
      definitions.set(definition.id, definition);
    },
    async activate(id, context) {
      await cleanups.get(id)?.();
      const cleanup = await definitions.get(id)?.activate(context);
      cleanups.set(id, typeof cleanup === "function" ? cleanup : () => {});
    },
    async deactivate(id) {
      await cleanups.get(id)?.();
      cleanups.delete(id);
      definitions.delete(id);
      document.querySelectorAll(`[data-cdp-hub-owner="${CSS.escape(id)}"]`)
        .forEach((node) => node.remove());
    }
  };
})();
```

**Step 2: Build one composite injection source**

For the enabled module set, concatenate:

1. the runtime;
2. guarded CSS style insertion;
3. each module entry in its own `try/catch`;
4. one `cdpHub.activate` call per script module.

Use manifest data to build context. Do not add a bundler inside CDP Injector.

**Step 3: Install and execute the source**

For every target:

1. call `Page.enable` and `Runtime.enable`;
2. call `Page.setBypassCSP` only if an enabled module declares it;
3. call `Page.addScriptToEvaluateOnNewDocument` once with the composite source;
4. call `Runtime.evaluate` with the same source for the current document.

**Step 4: Create an obvious built-in theme**

Use a small CSS change that makes success visible without rewriting the Codex
layout. Keep the theme intentionally simple and removable.

**Step 5: Demonstrate the first complete operation path**

Launch Codex with the theme enabled.

Verify:

- one CDP port belongs to the Codex Product Session;
- the theme is visible;
- reloading or replacing the main renderer reapplies it;
- launching without modules leaves Codex unchanged.

Stop here and ask the user to confirm this path works before adding package
installation or Taskboard migration.

**Step 6: Commit**

```bash
git add builtin-modules src-tauri
git commit -m "feat: inject the built-in Codex theme"
```

### Task 7: Import and validate `.cdpmod` packages

**Prerequisite:** The user has confirmed Task 6 works.

**Files:**
- Create: `../cdp-injector/src-tauri/src/module_package.rs`
- Modify: `../cdp-injector/src-tauri/src/model.rs`
- Modify: `../cdp-injector/src-tauri/src/commands.rs`
- Modify: `../cdp-injector/src-tauri/Cargo.toml`
- Modify: `../cdp-injector/src/api.ts`
- Modify: `../cdp-injector/src/App.tsx`

**Step 1: Parse the approved manifest schema**

Use concrete Rust structs with `#[serde(deny_unknown_fields)]`. Validate ID,
SemVer, `hubApi == 1`, known capabilities, contained relative paths, and at
least one injection entry.

**Step 2: Extract safely**

Use the `zip` crate. Reject absolute paths, `..`, NUL bytes, symlinks, and files
outside the temporary install directory. This is required trust-boundary
validation, not speculative compatibility code.

**Step 3: Install atomically**

Extract to an application-support temporary directory, validate the complete
package, then rename it to:

```text
Modules/<module-id>/<version>/
```

Preserve `Module Data/<module-id>/` across updates.

**Step 4: Add GUI import**

Use Tauri's file dialog for `.cdpmod`. Show name, version, targets, and declared
capabilities before the confirmation action.

**Step 5: Verify the direct package boundary**

Import a zipped copy of the built-in theme and enable it instead of the bundled
copy. Then import one ZIP containing `../escape` and verify the installer rejects
it without creating files outside the temporary directory.

**Step 6: Commit**

```bash
git add src src-tauri
git commit -m "feat: install local CDP modules"
```

### Task 8: Bundle Node and supervise module services

**Files:**
- Create: `../cdp-injector/src-tauri/resources/node/LICENSE`
- Add per build architecture: `../cdp-injector/src-tauri/resources/node/node`
- Create: `../cdp-injector/src-tauri/src/module_service.rs`
- Modify: `../cdp-injector/src-tauri/src/session.rs`
- Modify: `../cdp-injector/src-tauri/tauri.conf.json`
- Modify: `../cdp-injector/.github/workflows/release.yml`

**Step 1: Bundle a pinned Node 22+ binary**

Produce separate macOS arm64 and x86_64 builds, each containing the matching
official Node executable and license. Do not attempt a universal binary in the
first release.

**Step 2: Start a service directly**

Spawn:

```text
<bundled-node> <module-dir>/service/index.mjs
```

Set the approved environment variables, allocate a loopback port, create a
random session token, set the module's data directory, and capture stdout and
stderr. Never invoke a shell.

**Step 3: Wait for health**

Poll the manifest health path until success or `readyTimeoutMs`. On failure,
terminate the process and report the error only against that module.

**Step 4: Reference-count the service**

Start one process on the first consuming Product Session and stop it after the
last consumer closes. Send `SIGTERM`, wait briefly, then terminate a stuck child
because the process belongs to CDP Injector and is not a user document app.

**Step 5: Pass an opaque service URL**

Include the token in the generated module service URL and pass only that URL to
the renderer context. Never expose the Product CDP endpoint.

**Step 6: Verify service isolation**

Use a tiny fixture service that returns `{ "status": "ok" }` from `/health`.
Verify its URL reaches the injected module. Stop the fixture unexpectedly and
confirm the theme remains active while that module becomes `service failed`.

**Step 7: Commit**

```bash
git add .github src-tauri
git commit -m "feat: run packaged Node module services"
```

### Task 9: Package Taskboard as the first service-backed module

**Files in `dashi-taskboard`:**
- Create: `modules/cdp/manifest.json`
- Create: `scripts/build-cdp-module.mjs`
- Modify: `inject/codex-taskboard.user.js`
- Modify: `scripts/codex-injector.mjs`
- Modify: `server/app.mjs`
- Modify: `server/index.mjs`
- Modify: `package.json`
- Output: `dist/cdp/dev.dashi.taskboard.cdpmod`

**Step 1: Make service configuration injectable**

Read Hub variables before existing Taskboard defaults:

```js
const host = process.env.CDP_HUB_HOST ?? process.env.CODEX_TASKBOARD_HOST ?? "0.0.0.0";
const port = process.env.CDP_HUB_PORT ?? process.env.CODEX_TASKBOARD_PORT ?? "47823";
const dataDir = process.env.CDP_HUB_DATA_DIR ?? process.env.CODEX_TASKBOARD_DATA_DIR;
```

Keep the existing standalone commands working; do not duplicate the server.

**Step 2: Adapt the injection lifecycle**

Wrap the existing script in `cdpHub.register` for the module build. Use
`context.serviceUrl` instead of the fixed URL and return the existing `destroy`
behavior as cleanup. Keep standalone injection source generation for existing
Taskboard users until the module path is confirmed.

**Step 3: Move Taskboard-specific host behavior**

Keep iframe management and Codex DOM behavior in the injected module. Keep API,
SQLite, Codex CLI automation, and Taskboard business behavior in the packaged
service. Remove nothing from the standalone launcher before parity is observed.

**Step 4: Build the package with existing tools**

`scripts/build-cdp-module.mjs` uses Node standard library and the existing Vite
build. It stages only manifest, injection output, runtime server/shared files,
compiled `web/`, and icon, then creates the ZIP using an already available
system or project packaging mechanism. Do not add a packaging dependency unless
the standard library path is insufficient on supported macOS.

Add:

```json
"build:cdp": "npm run build:web && node scripts/build-cdp-module.mjs"
```

**Step 5: Verify standalone Taskboard still runs**

Run:

```bash
npm start
```

Expected: the existing browser Taskboard opens against its existing default
port and database behavior.

**Step 6: Verify packaged Taskboard through CDP Injector**

Import the `.cdpmod`, enable theme and Taskboard for Codex, then launch.

Verify only the direct path:

- theme is visible;
- Taskboard sidebar entry is visible;
- opening it loads the packaged web UI;
- creating or moving one task writes the module database;
- restarting Codex shows the changed task.

Ask the user to confirm before removing any old standalone injector behavior or
adding migration regression coverage.

**Step 7: Commit Taskboard changes**

```bash
git add inject modules package.json scripts server
git commit -m "feat: package Taskboard as a CDP Injector module"
```

### Task 10: Finish resident UI, tray, and module-specific diagnostics

**Prerequisite:** The user has confirmed the packaged Taskboard path.

**Files:**
- Modify: `../cdp-injector/src/App.tsx`
- Modify: `../cdp-injector/src/styles.css`
- Modify: `../cdp-injector/src-tauri/src/lib.rs`
- Modify: `../cdp-injector/src-tauri/src/state.rs`
- Modify: `../cdp-injector/src-tauri/src/session.rs`

**Step 1: Add the tray menu**

Include only `Show CDP Injector` and `Quit`. Closing the main window hides it;
Quit cleans module renderer state where reachable, stops module services, and
then exits.

**Step 2: Emit state changes**

Emit one typed Tauri event when Product or Module status changes. React reloads
the Product list from the existing command. Do not introduce Redux or another
state framework.

**Step 3: Display scoped failures**

Show Product phase and per-module error text. Include a small diagnostics view
with timestamp, Product, target, module, phase, and error. Store rolling text
logs under the application log directory.

**Step 4: Verify resident behavior**

Close the window while Codex and Taskboard run, reopen from the tray, and verify
their statuses remain accurate. Quit CDP Injector and confirm its module service
process exits.

**Step 5: Commit**

```bash
git add src src-tauri
git commit -m "feat: add resident session status and diagnostics"
```

### Task 11: Build the signed macOS artifacts

**Files:**
- Modify: `../cdp-injector/src-tauri/tauri.conf.json`
- Modify: `../cdp-injector/.github/workflows/release.yml`
- Create: `../cdp-injector/docs/releasing.md`
- Modify: `../cdp-injector/README.md`

**Step 1: Configure bundles**

Build `.app` and `.dmg` for macOS 12+, separately for arm64 and x86_64. Include
the matching Node runtime as a Tauri resource.

**Step 2: Configure signing inputs**

Document the Apple Developer certificate and notarization environment variables
without committing credentials. Use Tauri's standard signing path.

**Step 3: Build locally**

Run:

```bash
pnpm build
pnpm tauri build
```

Expected: the architecture-specific `.app` and `.dmg` contain the Product
Profile, renderer runtime, built-in theme, and Node executable.

**Step 4: Verify the packaged direct path**

Install the DMG into `/Applications`, import the Taskboard `.cdpmod`, and repeat
the confirmed theme + Taskboard operation path without a development terminal.

**Step 5: Commit**

```bash
git add .github README.md docs src-tauri
git commit -m "build: package CDP Injector for macOS"
```

### Task 12: Add only explicitly requested post-confirmation protection

Do not execute this task automatically. After the user confirms the installed
application works, ask which observed risks deserve regression protection.

Likely targeted checks, only when requested:

- manifest and ZIP traversal validation;
- CDP response correlation;
- Product target matching;
- renderer runtime cleanup;
- service health timeout and reference count;
- Taskboard environment mapping.

Run the smallest relevant commands rather than inventing a broad suite:

```bash
cd src-tauri && cargo test <specific-test-name>
pnpm test --run <specific-test-file>
node --test test/<specific-taskboard-test>.test.mjs
```

Commit each requested protection with the implementation it protects.

## Final release verification

After all user-approved work:

```bash
pnpm typecheck
cd src-tauri && cargo check
pnpm tauri build
```

Then perform the packaged macOS path manually:

```text
open CDP Injector.app
-> enable theme and Taskboard
-> launch Codex
-> observe both modules
-> mutate one Taskboard issue
-> restart through Injector
-> observe persisted result
-> disable all modules
-> launch Codex normally
```

Do not claim general Electron/Chromium compatibility until a second Product is
implemented and demonstrated.
