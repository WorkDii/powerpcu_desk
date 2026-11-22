# PowerPCU Remote branding

- `logo.svg` (24 px and up), `logo-small.svg` (16–24 px) and `wordmark.svg` are the sources. Logo C: `#004c28` tile, PowerPCU ECG line, `#10b981` cursor badge.
- `out/` is generated and committed, so CI needs no Node. To regenerate: `npm --prefix <dir> i @resvg/resvg-js@2.6.2`, then `POWERPCU_ICON_TOOLS=<dir> node res/powerpcu/gen-icons.mjs`. The repo ignores `*png`/`*svg`, so add new outputs with `git add -f`.
- `apply-branding.ps1` runs before the build in `.github/workflows/powerpcu-remote.yml`. It copies `out/` over the app, tray and portable icons, adds `flutter/assets/{icon.ico,icon.png,icon.svg,logo.png,logo_dark.png}`, and rewrites the version strings in `flutter/windows/runner/Runner.rc` and `libs/portable/Cargo.toml`. It refuses to run twice.
- The checked-in RustDesk files stay untouched; branding exists only in a build that ran this script.
