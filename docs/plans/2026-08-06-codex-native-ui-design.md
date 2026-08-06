# CDP Injector Codex-Native UI Design

## Selected reference

The selected visual target is
[`assets/2026-08-06-cdp-injector-codex-ui-option-2.png`](assets/2026-08-06-cdp-injector-codex-ui-option-2.png).

It adapts the current Codex desktop visual language without claiming that a
public Codex desktop design system exists. The implementation uses dark layered
surfaces, low-contrast separators, compact navigation, system typography,
monochrome outline icons, restrained radii, and one white primary action.

## Information architecture

The 980 × 680 window has three columns:

1. A 184 px product rail with CDP Injector identity, Codex selection, Modules,
   Diagnostics, and Settings.
2. A flexible module workspace with the module list and local package import.
3. A 286 px Product Session inspector showing Codex, CDP, target, and injection
   phases plus the primary launch action.

Module management is the default view. The user sees enabled modules and their
readiness before starting Codex. Diagnostics is a lightweight alternate center
view over the same live status data; it is not a separate route.

## Interaction rules

- Module switches call the existing `set_module_enabled` Tauri command and
  refresh the real product view.
- Modules and Diagnostics navigation changes only the center workspace.
- The right inspector remains visible so operational state never disappears.
- Import and Launch are shown in their intended locations but remain disabled
  until their planned backend commands exist. The UI must not fake success.
- Settings is visible but disabled until resident-app settings are implemented.
- Errors render inline and use red only for the scoped error message.
- Keyboard focus is always visible; icon-only controls have accessible labels.

## Current-scope fidelity

The reference contains Theme and Taskboard rows. The current implementation
renders only modules returned by Rust. It does not fabricate Taskboard before
the approved migration checkpoint. Empty and one-module states retain the same
layout without placeholder modules.

## Visual tokens

- Canvas: `#0b0b0c`
- Rail: `#101011`
- Workspace surface: `#121213`
- Inspector: `#171718`
- Raised row: `#1a1a1c`
- Primary text: `#f2f2f3`
- Secondary text: `#9a9aa0`
- Divider: `rgba(255, 255, 255, 0.08)`
- Healthy/waiting/error: `#35c46a`, `#f5b82e`, `#ef5a5a`
- Radius: 8–10 px
- Font: macOS system stack

No gradients, glass effects, bright tile icons, large shadows, or custom fonts.

## Verification

At 980 × 680, compare the selected reference and the running UI in the same
state. Modules navigation, Diagnostics navigation, and the real theme switch
must work. Browser console output must contain no errors. Visual QA passes only
after all P0–P2 mismatches are fixed.
