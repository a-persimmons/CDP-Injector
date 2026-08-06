# CDP Injector UI Design QA

- Source visual truth: `docs/plans/assets/2026-08-06-cdp-injector-codex-ui-option-2.png`
- Normalized source: `docs/plans/assets/2026-08-06-cdp-injector-ui-source-normalized.png`
- Implementation screenshot: `docs/plans/assets/2026-08-06-cdp-injector-ui-implementation.png`
- Diagnostics screenshot: `docs/plans/assets/2026-08-06-cdp-injector-ui-diagnostics.png`
- Side-by-side evidence: `docs/plans/assets/2026-08-06-cdp-injector-ui-comparison.png`
- Viewport: 980 x 680 CSS px, device scale factor 1
- Pixels: source 1506 x 1045; normalized source 980 x 680; implementation 980 x 680; comparison 1960 x 680
- Normalization: the source and implementation have the same aspect ratio; the source was resized to 980 x 680 without cropping.
- State: Codex selected, Modules active, Codex theme enabled, Product Session not running.

## Full-view comparison evidence

The final side-by-side comparison confirms the selected three-column proportions, dark surface hierarchy, 184 px navigation rail, module table alignment, compact 68 px row rhythm, 286 px runtime inspector, typography hierarchy, semantic status colors, Phosphor icon family, button treatments, and Chinese product copy. The browser capture omits the macOS traffic-light chrome from the source; this is expected because the evidence captures the app content viewport. The source includes a Taskboard row and warning that are intentionally absent until Taskboard migration is authorized.

## Focused region comparison

No separate crop was needed: at 980 x 680, the module table text, status dot, switch, disclosure control, navigation, and runtime steps are readable at 1:1 density in the full-view comparison.

## Findings

- No actionable P0, P1, or P2 visual differences remain.
- P3: the browser-rendered system font rasterization is slightly sharper than the generated reference. This is expected and should be rechecked in the packaged macOS WebView.

Required fidelity surfaces:

- Fonts and typography: system font stack, weights, sizes, line heights, and hierarchy match the reference intent; no wrapping or truncation remains in the module row.
- Spacing and layout rhythm: column boundaries, header/table placement, row density, padding, radii, dividers, and bottom launch area align with the source.
- Colors and visual tokens: dark neutrals, low-contrast borders, disabled controls, green ready state, and light primary action match the selected direction with accessible contrast.
- Image quality and asset fidelity: no raster product imagery is required; visible interface symbols use the installed Phosphor icon set rather than placeholder drawings.
- Copy and content: Chinese labels are coherent and standalone; unavailable Taskboard content is not fabricated.

## Interaction evidence

- Modules navigation returns to the module management view.
- Diagnostics navigation opens the Product Session diagnostic list.
- Theme switch changes `已就绪` to `未启用` and back to `已就绪`.
- Import, Settings, disclosure, and Launch remain disabled until their real backend commands exist.
- Browser console warnings/errors checked: none.

## Comparison history

1. Initial 980 x 680 comparison: P1 controls were clipped because the module grid minimum tracks exceeded the center column. Fixed by reducing the grid track minimums and control gaps. Post-fix evidence shows status, switch, and disclosure fully visible.
2. Second comparison: P2 table density was too tall and the description wrapped. Fixed by matching the 37 px header, 68 px row, source-aligned header spacing, and compact column allocation. Post-fix evidence is the final side-by-side comparison listed above.

## Implementation checklist

- [x] Match the selected Codex-style three-column layout.
- [x] Preserve only backend-supported module data.
- [x] Exercise core module, diagnostics, and toggle interactions.
- [x] Check disabled capability states and browser console.
- [x] Recompare at the same viewport after fixes.

final result: passed
