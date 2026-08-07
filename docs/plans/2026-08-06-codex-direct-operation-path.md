# Codex Direct Operation Path Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the selected workspace launch Codex through one Rust-owned CDP Product Session and visibly inject the built-in theme.

**Architecture:** React calls small Tauri commands and renders the Product View; Rust owns application detection, restart, CDP port allocation, target connections, phase changes, and injection. The implementation remains Codex-only and stops after the theme path is ready for user acceptance.

**Tech Stack:** Tauri 2, React 19, TypeScript, Rust, Tokio, reqwest, tokio-tungstenite, serde_json.

---

### Task 1: Decide and prepare Codex launch

**Files:**
- Create: `src-tauri/src/product.rs`
- Modify: `src-tauri/src/model.rs`
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/product.rs`

**Step 1: Write the failing launch-decision test**

Add a pure test proving that no enabled modules selects normal launch and an
enabled module selects injected launch with restart required only when Codex is
already running.

**Step 2: Run the focused test and verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml product::tests::launch_decision`

Expected: FAIL because `product` and the decision function do not exist.

**Step 3: Implement the minimum product lifecycle**

Add serializable `LaunchPreparation { mode, restart_required }`. Resolve the
first existing configured `.app`, check configured process names with
`/usr/bin/pgrep -x`, and expose a pure decision helper. Do not add a generic
platform abstraction.

**Step 4: Implement safe process operations**

- Normal launch: `/usr/bin/open -a <absolute app path>`.
- Injected launch: request normal quit with `/usr/bin/osascript`, poll process
  exit, return `请手动退出 Codex 后重试` on timeout, reserve a loopback port,
  then run `/usr/bin/open -n -a <path> --args --remote-debugging-port=<port>
  --remote-allow-origins=*`.
- Never force-kill Codex.

**Step 5: Run tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all tests pass.

**Step 6: Commit**

```bash
git add src-tauri/src
git commit -m "feat: prepare Codex product launches"
```

### Task 2: Wire launch commands and workspace states

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/api.ts`
- Modify: `src/App.tsx`
- Modify: `src/styles.css`

**Step 1: Add state-transition coverage**

Extend the existing state test to set and read a Product phase. Verify the
Product View returns the new phase without changing module settings.

**Step 2: Verify the new assertion fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml state::tests::product_phase_updates`

Expected: FAIL because the phase setter does not exist.

**Step 3: Add commands**

- `prepare_launch(product_id) -> LaunchPreparation`
- `launch_product(product_id) -> ()`

Validate the Product ID against the loaded built-in profile. Update phase text
through the existing shared Product status.

**Step 4: Connect React**

Enable the launch button. Call `prepare_launch`; when restart is required, show
the exact Product Profile restart message in a native dialog-shaped React
confirmation; call `launch_product` only after confirmation. Disable launch
while busy and show scoped errors. Keep Import, Settings, and module disclosure
disabled.

**Step 5: Verify**

Run:

```bash
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: build succeeds and all Rust tests pass.

**Step 6: Commit**

```bash
git add src src-tauri
git commit -m "feat: launch Codex from the workspace"
```

### Task 3: Connect matching Codex renderer targets

**Files:**
- Create: `src-tauri/src/cdp.rs`
- Create: `src-tauri/src/session.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/cdp.rs`

**Step 1: Write target-matching tests**

Cover page type, required `app://` prefix, and the `global-dictation`
exclusion using small JSON target fixtures.

**Step 2: Verify the tests fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml cdp::tests::matches_codex_target`

Expected: FAIL because the matcher does not exist.

**Step 3: Add only required dependencies**

Add `reqwest` with JSON support, `tokio-tungstenite`, `futures-util`,
`thiserror`, and Tokio runtime/time features. Do not add a CDP framework.

**Step 4: Implement the minimal CDP connection**

Fetch `/json/list`, deserialize page targets, match the Product context, open
the target WebSocket, and correlate command responses by incrementing ID. One
connection belongs to the Rust Product Session; modules receive no socket.

**Step 5: Poll targets**

Poll every two seconds, connect new matching targets, and discard closed ones.

```rust
// ponytail: polling is sufficient for the first Codex target; use CDP target
// discovery events if measured replacement latency becomes a problem.
```

**Step 6: Verify**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all tests pass.

**Step 7: Commit**

```bash
git add src-tauri
git commit -m "feat: connect Codex renderer targets over CDP"
```

### Task 4: Inject the built-in theme

**Files:**
- Create: `src-tauri/resources/runtime/cdp-injector.js`
- Create: `builtin-modules/codex-theme/manifest.json`
- Create: `builtin-modules/codex-theme/inject/index.css`
- Create: `src-tauri/src/injection.rs`
- Modify: `src-tauri/src/session.rs`
- Modify: `src-tauri/tauri.conf.json`
- Test: `src-tauri/src/injection.rs`

**Step 1: Write a failing composite-source test**

Assert that the generated source contains the runtime guard, a stable style ID,
the built-in module ID, and the theme CSS exactly once.

**Step 2: Verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml injection::tests::builds_theme_source`

Expected: FAIL because the injection module does not exist.

**Step 3: Add the minimal renderer runtime and visible theme**

The runtime exposes API version 1 and cleanup storage. The theme only changes a
small set of root color variables and adds an unmistakable outline marked by
`data-cdp-hub-owner`; it must not restructure Codex.

**Step 4: Inject current and future documents**

For each matching target call `Page.enable`, `Runtime.enable`,
`Page.addScriptToEvaluateOnNewDocument`, then `Runtime.evaluate`. A single
composite source contains the runtime and all enabled built-in module content.

**Step 5: Bundle resources and verify**

Run:

```bash
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
```

Expected: build and tests pass with no whitespace errors.

**Step 6: Commit**

```bash
git add builtin-modules src-tauri
git commit -m "feat: inject the built-in Codex theme"
```

### Task 5: User acceptance checkpoint

**Files:**
- Modify only if verification finds a defect: `src/**`, `src-tauri/**`

**Step 1: Launch a detached Tauri application bundle**

Build the frontend, create an unsigned debug `.app`, and launch it through
macOS LaunchServices:

```bash
pnpm build
pnpm tauri build --debug --bundles app --no-sign
open -n "src-tauri/target/debug/bundle/macos/CDP Injector.app"
```

Expected: the 980 x 680 CDP Injector window opens with real Tauri commands and
remains running when Codex exits. Do not use `tauri dev` for this checkpoint;
its process can inherit the active Codex development session and be terminated
with it.

**Step 2: Verify normal launch**

Disable every module and click Launch Codex. Codex opens normally and no CDP
session is created.

**Step 3: Verify injected launch**

Enable Codex Theme, click Launch Codex, accept the restart notice, and verify
the visible theme marker appears.

**Step 4: Verify persistence and replacement**

Reload or replace the Codex renderer. Verify the theme returns and the right
inspector reaches `已注入` without module errors.

**Step 5: Stop at the checkpoint**

Ask the user to confirm the real path. Do not implement `.cdpmod`, Taskboard,
generic Electron application import, marketplace, or settings before that
confirmation.
