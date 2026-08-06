# Codex-Native UI Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the temporary Codex product card with the selected dark three-column module-management interface while preserving the existing Tauri state flow.

**Architecture:** React keeps one local navigation state and renders the Product View returned by Rust. Existing Tauri commands remain the only persistence boundary; unavailable Import, Settings, and Launch behaviors stay disabled until their planned backend tasks.

**Tech Stack:** React 19, TypeScript, CSS, Tauri 2, `@phosphor-icons/react`, Rust tests, Vite.

---

### Task 1: Implement the selected shell and live module controls

**Files:**
- Modify: `package.json`
- Modify: `pnpm-lock.yaml`
- Modify: `src/App.tsx`
- Modify: `src/styles.css`

**Step 1: Preserve the current behavior check**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml module_selection_persists
```

Expected: PASS before the UI change.

**Step 2: Add the one required icon dependency**

Run:

```bash
pnpm add @phosphor-icons/react
```

Use Phosphor's regular-weight monochrome icons. Do not draw icons with CSS,
text glyphs, inline SVG, or a second icon package.

**Step 3: Implement the three-column interface**

Replace the temporary card with:

- the product/navigation rail;
- Modules and Diagnostics center views;
- a persistent Product Session inspector;
- real module switches;
- truthful disabled Import, Settings, and Launch controls.

Use only the backend-provided module list. Keep one error message region and one
loading state. Do not add routing or another state library.

**Step 4: Implement the selected visual tokens**

Recreate the selected 980 × 680 reference in `src/styles.css`. Add visible
focus styles and a narrow-window fallback that preserves all controls.

**Step 5: Verify behavior and compilation**

Run:

```bash
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: TypeScript build succeeds and both Rust tests pass.

### Task 2: Run browser interaction checks and visual QA

**Files:**
- Create: `design-qa.md`
- Modify if required: `src/App.tsx`
- Modify if required: `src/styles.css`

**Step 1: Start the existing Vite app**

Run `pnpm dev -- --host 127.0.0.1` and open the app in the Codex in-app
browser at 980 × 680.

**Step 2: Verify the primary interactions**

Check Modules, Diagnostics, keyboard focus, and module switches. Because plain
Vite lacks Tauri commands, visual QA may use a browser-only preview state, but
the production Tauri code path must remain unchanged.

**Step 3: Compare against the selected reference**

Open the selected reference and the rendered screenshot together. Record P0–P3
differences in `design-qa.md`, fix P0–P2, and repeat until it states:

```text
final result: passed
```

**Step 4: Commit**

```bash
git add package.json pnpm-lock.yaml src docs/plans design-qa.md
git commit -m "feat: add Codex-native module workspace"
```
