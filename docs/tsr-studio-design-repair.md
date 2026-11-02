# Studio interaction and visual repair — 2026-10-30

The previous approval modal was a dead end: it explained the trust boundary but supplied no usable reply, and clipped badly in the actual Safari window. The redesign keeps that genuine boundary and makes the next action explicit.

## Delivered

- Conversation-first workspace, clear selected project, restrained blue/neutral typography and spacing, secondary Activity/files and TSR context.
- Inline recovery panel with three steps, precise selectable approval reply including disclosure digest, Copy/repeat-copy feedback and Back. Copying/back do not call any mutation endpoint, approve, or spend. No fake browser authentication added.
- While blocked, composer explicitly says Save message instead of implying a model response. Redundant blocked Continue button removed. Existing messages/drafts retained.
- Navigation is disabled during a pending Send to prevent response completion from clearing a different project's/new-project draft. New project hides unrelated preview/context tabs.
- Play preview exposes only fixed delivered catalog artifacts, byte-identical, in an opaque sandbox without same-origin access. Explicit asset-prefix CSP permits their stylesheet/script, blocks connect/forms; no protected source or current private native state is served. Open full preview is available. This host preview is not the networkless execution container or a new gameplay-quality certification.
- Prior/watch response excerpts are labeled earlier input, not answers to later saved messages. Full cross-session conversation aggregation is not implemented; journals remain preserved.

## Actual verification

Safari actions: project/new-project/back; authored unsent draft retained across switching and recovery/back, then only that QA text removed; approval panel opened, Copy clicked twice, Back; TSR history inspected; real delivered Pond preview loaded with correct stylesheet and drawn game canvas. No production Send, approval, Start-game or provider action was triggered. Old tabs/drafts and game artifacts were not refreshed or modified. One accidental browser Home shortcut navigated only the QA tab to Apple; Back restored Studio immediately, and the screenshot was replaced with the correct Studio capture.

Independent visual/source review found missing preview styles under opaque-origin `self` CSP. Fixed using exact local catalog asset-prefix sources, keeping sandbox restrictions. Final review found no blocking issue within this scope.

21 existing authored HTTP/native tests passed (65.283s), plus the new preview allowlist/byte-equality/CSP/no-effects regression passed (0.533s). JS syntax, Python compile and diff whitespace checks pass. Browser fixture selectors updated but container browser suite not rerun. Narrow/mobile CSS is implemented; narrow viewport rendering remains unverified in this pass. Actual desktop screenshots were inspected.

Screenshots (local QA artifacts):
- `target/studio-before-redesign.png`
- `target/studio-conversation-after.png`
- `target/studio-approval-after.png`
- `target/studio-preview-after.png`

## Remaining real boundary

No authenticated financial-risk approval has arrived. The user must send the exact copied disclosure in the existing ChatGPT conversation; a local click cannot establish that identity. The old tool-free output must remain abandoned, uncertain liability retained ($0.05 is an estimate, not a verified maximum), and fresh paid work requires a reviewed financial disposition and remaining-budget admission. No acceptance, quarantine application, provider-setting change or paid request occurred. Pond's visual redesign remains queued.
