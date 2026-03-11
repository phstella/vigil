# Technical Debt & Known Issues

## Active

### Vite/Svelte circular chunk warning during build
- **Severity**: Low (non-blocking)
- **Introduced**: Epic 0 (SvelteKit scaffold)
- **Description**: `npm run build` emits a Rollup circular dependency warning related to Svelte's internal `tick` re-export between chunks. This is a known upstream Svelte/Rollup issue.
- **Impact**: No runtime impact. Build succeeds. May cause noise in CI logs.
- **Track before**: Epic 2/3 complexity growth could amplify chunk issues.
- **Upstream**: Monitor Svelte/Vite releases for a fix.

### Placeholder icons
- **Severity**: Low
- **Introduced**: Task 0.4
- **Description**: `src-tauri/icons/` contains solid dark square placeholders. Replace with actual Vigil branding before any release build.

### Fonts not yet bundled
- **Severity**: Low
- **Introduced**: Task 0.6
- **Description**: Design tokens reference `Inter` and `JetBrains Mono` fonts but they are not loaded. System font fallbacks work in the meantime. Add `@fontsource` packages or CDN link in a future task.

## Resolved

(none yet)
