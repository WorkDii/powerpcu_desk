# PowerPCU Remote (phase 1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Windows x64 "PowerPCU Remote" from this RustDesk fork. It is incoming-only, consent on every connection, server baked in, and isolated from a user's own RustDesk. It ships as `powerpcu-remote-x86_64.exe` + `.sha256` on a GitHub Release.

**Architecture:** A non-default cargo feature `powerpcu` adds `src/powerpcu.rs`. That module fills `hbb_common::config::{APP_NAME, HARD_SETTINGS, OVERWRITE_SETTINGS, OVERWRITE_LOCAL_SETTINGS, DEFAULT_LOCAL_SETTINGS, BUILTIN_SETTINGS}` at startup from `load_custom_client()`. Isolation and feature cuts are one-line `#[cfg(feature = "powerpcu")]` hooks. Flutter hides UI behind a builtin option `powerpcu=Y`. Branding is applied at build time by a script. A new workflow builds and releases.

**Tech Stack:** Rust 1.75 (CI toolchain), Flutter 3.24.5 desktop, PowerShell 7 (branding script), Node 24 (icon generation, dev-time only), GitHub Actions `windows-2022`.

**Spec:** `docs/superpowers/specs/2026-11-22-powerpcu-remote-design.md` (approved 2026-11-22). Read it first. Section numbers below (§5.2, R1…) refer to it.

## Global Constraints

- Feature off ⇒ byte-for-byte stock behaviour. Every change to an existing file is inside `#[cfg(feature = "powerpcu")]` (Rust) or behind `isPowerPcuBuild()` (Dart). The only exceptions are declarations: the `[features]` entry, the `mod` line and the `OPTION_POWERPCU` const.
- Do not modify `libs/hbb_common` (submodule), `build.py`, `.github/workflows/flutter-build.yml` or any other existing workflow, `LICENCE`, or `src/lang/*`.
- `APP_NAME` is exactly `PowerPCU-Remote`. The packed exe is exactly `powerpcu-remote.exe`. The release assets are exactly `powerpcu-remote-x86_64.exe` and `powerpcu-remote-x86_64.exe.sha256`.
- Server values are exactly `rustdesk.powerpcu.com` and key `Yl+eLj+8+MvlyMQcg117JyVvyy1ojHSGaY2+N7aYmO8=`.
- Never set `password`, `salt`, `disable-installation` or `allow-logon-screen-password`.
- AGENTS.md rules apply:
  - no `unwrap()`/`expect()` outside tests and lock acquisition;
  - peer-driven code paths log at `trace` only;
  - one `use` block per crate;
  - no comments that restate code;
  - smallest diff.
- No new crate dependencies (`sha2` is already a root dependency). No new FFI functions.
- Tag pattern is `remote-v*`, e.g. `remote-v1.5.3-1`. Internal version stays `1.5.3`.
- Commit messages end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Use `git commit -F <file>`; no heredocs.

## Review Focus

1. **An existing user config or a stock RustDesk on the same relay.** PowerPCU must never present the user's RustDesk ID. Any ID outside 1_000_000_000..2_000_000_000 (e.g. the MAC ID `Config::load()` fills in) is replaced; check `apply()` order (APP_NAME before any `Config` access).
2. **The exe renamed to `…host=x,key=y….exe`, or `RUSTDESK_APPNAME` set.** It must still use `rustdesk.powerpcu.com` (R2). Covered by the manual checklist; R2 is a one-line early return.
3. **Uninstall or install scripts calling our exe with shared-driver commands.** These must be no-ops. Pinned by the `is_shared_driver_command` unit test (Task 1) plus the core_main hook (Task 2).
4. **A peer that keeps sending chat, voice-call or switch-sides messages.** They are dropped with no log above `trace` and no CM traffic (Task 3).
5. **The branding script run twice, or run after upstream changes `Runner.rc`.** A second run must be a no-op or fail loudly, never write a half-branded file (Task 5 verification step).

---

### Task 1: Built-in settings module and startup hook (§5.2, R1)

**Files:**
- Create: `src/powerpcu.rs`
- Modify: `Cargo.toml` (`[features]`, after `screencapturekit = [...]`): add `powerpcu = []`
- Modify: `src/lib.rs` (after `mod kcp_stream;`): add `#[cfg(any(feature = "powerpcu", test))]` + `mod powerpcu;`
- Modify: `libs/base/src/config/keys.rs` (in the built-in options group, after `OPTION_ALLOW_COMMAND_LINE_SETTINGS_WHEN_SETTINGS_DISABLED`): add `pub const OPTION_POWERPCU: &str = "powerpcu";`. Do **not** add it to `KEYS_BUILDIN_SETTINGS`.
- Modify: `src/common.rs` `load_custom_client()` (line ~2518): first statement becomes the hook `#[cfg(feature = "powerpcu")] { crate::powerpcu::apply(); return; }`
- Test: `#[cfg(test)] mod tests` inside `src/powerpcu.rs`

**Interfaces:**
- Produces (all `pub(crate)` in `src/powerpcu.rs`):
  - `const APP_NAME: &str = "PowerPCU-Remote";`
  - `const RENDEZVOUS_SERVER: &str = "rustdesk.powerpcu.com";`
  - `const KEY: &str = "Yl+eLj+8+MvlyMQcg117JyVvyy1ojHSGaY2+N7aYmO8=";`
  - `struct Settings { hard, overwrite, overwrite_local, default_local, builtin }`, every field `HashMap<String, String>`
  - `fn settings() -> Settings`
  - `fn derive_id(machine_uuid: &[u8]) -> String`
  - `fn is_shared_driver_command(arg: &str) -> bool`
  - `#[cfg(feature = "powerpcu")] fn apply()`
- Mark `apply()` `#[cfg(feature = "powerpcu")]`. Gate the other items so the module compiles warning-free both under `test` without the feature and under the feature without `test` (`#[cfg_attr(not(feature = "powerpcu"), allow(dead_code))]` on the module items is acceptable).
- Use `base::config::keys::*` constants where one exists, otherwise string literals. `conn-type`, `disable-ab`, `disable-account`, `disable-settings`, `allow-hide-cm` and `lang` may have constants in `hbb_common::config::keys` re-exported through `base::config::keys`; grep first.

- [ ] **Step 1: Write the failing tests** in `src/powerpcu.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_name_is_installable_and_matches_exe() {
        assert!(APP_NAME.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'));
        assert!(APP_NAME.eq_ignore_ascii_case("powerpcu-remote"));
    }

    #[test]
    fn consent_and_server_are_locked() {
        let s = settings();
        assert_eq!(s.hard["conn-type"], "incoming");
        assert_eq!(s.overwrite["approve-mode"], "click");
        assert_eq!(s.overwrite["verification-method"], "use-temporary-password");
        assert_eq!(s.overwrite["custom-rendezvous-server"], RENDEZVOUS_SERVER);
        assert_eq!(s.overwrite["key"], KEY);
        assert_eq!(s.overwrite["access-mode"], "custom");
        assert_eq!(s.overwrite_local["allow-remote-cm-modification"], "N");
    }

    #[test]
    fn no_password_path_is_preset() {
        let s = settings();
        for k in ["password", "salt", "disable-installation"] {
            assert!(!s.hard.contains_key(k), "{k}");
        }
        assert!(!s.builtin.contains_key("allow-logon-screen-password"));
        assert_eq!(s.builtin["disable-change-permanent-password"], "Y");
    }

    #[test]
    fn only_screen_input_clipboard_and_files_are_enabled() {
        let s = settings();
        for k in ["enable-keyboard", "enable-clipboard", "enable-file-transfer"] {
            assert_eq!(s.overwrite[k], "Y", "{k}");
        }
        for k in ["enable-audio", "enable-camera", "enable-terminal", "enable-tunnel",
                  "enable-remote-printer", "enable-remote-restart", "enable-record-session",
                  "enable-block-input", "enable-privacy-mode", "enable-lan-discovery",
                  "direct-server", "allow-auto-update", "allow-remote-config-modification"] {
            assert_eq!(s.overwrite[k], "N", "{k}");
        }
        assert_eq!(s.overwrite_local["enable-check-update"], "N");
        assert_eq!(s.builtin["register-device"], "N");
        assert_eq!(s.builtin["powerpcu"], "Y");
    }

    #[test]
    fn derived_id_is_stable_and_outside_mac_range() {
        assert_eq!(derive_id(b"test-machine"), "1031630599");
        assert_eq!(derive_id(b"other-machine"), "1427890610");
        let id: u64 = derive_id(b"x").parse().unwrap();
        assert!((1_000_000_000..2_000_000_000).contains(&id));
    }

    #[test]
    fn shared_driver_commands_are_recognised() {
        for c in ["--install-remote-printer", "--uninstall-remote-printer",
                  "--uninstall-cert", "--uninstall-amyuni-idd", "--install-idd"] {
            assert!(is_shared_driver_command(c), "{c}");
        }
        for c in ["--install", "--uninstall", "--service", "--server", "--cm"] {
            assert!(!is_shared_driver_command(c), "{c}");
        }
    }
}
```

- [ ] **Step 2: Run the tests and confirm they fail to compile.** `settings`/`derive_id` are not defined yet.
  - Run: `cargo test --locked --lib powerpcu::`
  - Expected: compile error naming `settings` / `derive_id` / `is_shared_driver_command`.
  - If the local machine cannot build the root crate (no vcpkg; see Task 6 note), record that and rely on the CI test step in Task 6.

- [ ] **Step 3: Implement `settings()`, `derive_id()`, `is_shared_driver_command()`.**
  - `settings()` returns exactly the §5.2 table:
    - HARD: `conn-type`, `disable-account`, `disable-ab`, `disable-settings`
    - OVERWRITE: every OVERWRITE row, including `allow-hide-cm=N`, `enable-trusted-devices=N`, `allow-websocket=N`, `allow-insecure-tls-fallback=N`
    - OVERWRITE_LOCAL: `enable-check-update=N`, `allow-remote-cm-modification=N`
    - DEFAULT_LOCAL: `lang=th`
    - BUILTIN: the six `hide-*-settings` keys, `hide-powered-by-me`, `disable-change-permanent-password`, `disable-change-id`, `disable-unlock-pin`, `register-device=N`, `powerpcu=Y`
  - `derive_id` algorithm (pinned by the test vectors):

```rust
// sha256(machine_uuid ‖ APP_NAME), first 8 bytes big-endian, folded into 10 digits
let digest = Sha256::new().chain_update(machine_uuid).chain_update(APP_NAME.as_bytes()).finalize();
let mut b = [0u8; 8];
b.copy_from_slice(&digest[..8]);
(1_000_000_000 + u64::from_be_bytes(b) % 1_000_000_000).to_string()
```

  - `is_shared_driver_command`: membership in the five commands from the test.

- [ ] **Step 4: Implement `apply()`**, in this order (the order is a requirement):
  1. `*config::APP_NAME.write().unwrap() = APP_NAME.to_owned();`
  2. Extend `HARD_SETTINGS`, `OVERWRITE_SETTINGS`, `OVERWRITE_LOCAL_SETTINGS`, `DEFAULT_LOCAL_SETTINGS`, `BUILTIN_SETTINGS` with `settings()`. Lock-acquire `unwrap()` is allowed.
  3. `if config::Config::get_id_or(String::new()).is_empty() { config::Config::set_id(&derive_id(&hbb_common::get_uuid())); }`
- [ ] **Step 5: Add the R1 hook** in `src/common.rs` `load_custom_client()`, as the first statement:

```rust
#[cfg(feature = "powerpcu")]
{
    crate::powerpcu::apply();
    return;
}
```

- [ ] **Step 6: Run the tests and confirm they pass.**
  - Run: `cargo test --locked --lib powerpcu::`
  - Expected: `6 passed`.
  - Also run `cargo check --locked --lib --features powerpcu` if the machine can build; expected: no errors and no new warnings in touched files.
- [ ] **Step 7: Commit.** Message: `feat(powerpcu): built-in identity, locked settings and own ID`

### Task 2: Isolation hooks (§5.3 R2, R3/R4, R5, R6, R10, R11)

R3/R4 refinement (same effect as the spec, smaller diff): the install, uninstall and update scripts call **our own exe** with `--install-remote-printer`, `--uninstall-remote-printer`, `--uninstall-cert`, `--uninstall-amyuni-idd` and `--install-idd`. So one guard in the CLI dispatcher turns all of them into no-ops, and the script text stays untouched. Update the R3/R4 rows of the spec in the same commit to say this.

**Files:**
- Modify: `src/core_main.rs` (inside the `#[cfg(windows)] { … }` block that starts the `if args[0] == "--uninstall"` chain, ~line 235): insert the guard before the chain
- Modify: `src/platform/windows.rs` `get_license_from_exe_name()` (~2212): first statement
- Modify: `src/platform/windows.rs` `get_valid_subkey()` (~1351): skip both `IS1` lookups
- Modify: `src/updater.rs` `check_update()` (~189): first statement
- Modify: `src/virtual_display_manager.rs` `is_virtual_display_supported()` (~26) and `reset_all()` (~124): first statements
- Modify: `docs/superpowers/specs/2026-11-22-powerpcu-remote-design.md` (R3/R4 rows)

**Interfaces:**
- Consumes: `crate::powerpcu::is_shared_driver_command(&str) -> bool` (Task 1)

- [ ] **Step 1: core_main guard.**

```rust
#[cfg(feature = "powerpcu")]
if crate::powerpcu::is_shared_driver_command(&args[0]) {
    return None;
}
```

- [ ] **Step 2: R2.** In `get_license_from_exe_name`:
  - add `#[cfg(feature = "powerpcu")] hbb_common::bail!("exe-name license is disabled in this build");`
  - `bail!` is already used in this file; match the existing import style.
- [ ] **Step 3: R5.** In `get_valid_subkey`, wrap the two `IS1` blocks with `#[cfg(not(feature = "powerpcu"))]` (attribute on each block), leaving the app-name lookups untouched.
- [ ] **Step 4: R6.** In `check_update`, add `#[cfg(feature = "powerpcu")] return Ok(());` before the macOS block. If the compiler flags the rest as unreachable, use `#[cfg(feature = "powerpcu")] { let _ = manually; return Ok(()); }`. Either way, no warnings.
- [ ] **Step 5: R10/R11.** Add `#[cfg(feature = "powerpcu")] return false;` to `is_virtual_display_supported` and `#[cfg(feature = "powerpcu")] return Ok(());` to `reset_all`.
- [ ] **Step 6: Verify.**
  - Run: `cargo test --locked --lib powerpcu::` → `6 passed` (unchanged).
  - Run: `git diff --stat` → only the six files above changed, and only by added lines, plus the `#[cfg(not(...))]` attribute lines for R5.
  - Full Windows compile is verified by the Task 6 CI run.
- [ ] **Step 7: Commit.** Message: `feat(powerpcu): keep shared RustDesk drivers, server lock and updater untouched`

### Task 3: Drop chat, voice call and switch sides server-side (§5.3 R7–R9)

**Files:**
- Modify: `src/server/connection.rs`:
  - arm `Some(misc::Union::ChatMessage(c))` (~3966)
  - arm `Some(misc::Union::SwitchSidesRequest(s))` (~4040)
  - arm `Some(message::Union::VoiceCallRequest(request))` (~4142)

**Interfaces:** none.

- [ ] **Step 1: Add an early exit as the first statement of each arm body.** It must skip all side effects of that arm: no `send_to_cm`, no `chat_unanswered`, no auto-disconnect timer, no switch-sides UUID insertion, no voice-call timestamp. Read the enclosing function to choose the right form; if the arm is inside a `match` whose value is unused, a `#[cfg(feature = "powerpcu")] { let _ = &c; }` followed by `#[cfg(not(feature = "powerpcu"))] { …existing body… }` is **not** allowed (it rewrites the arm). Allowed forms: an early `return true;`/`continue`-equivalent that the surrounding loop already uses for "message handled", or a cfg-gated guard arm placed *before* the existing arm, if that compiles with no unreachable-pattern warning when the feature is off. Log nothing, or `log::trace!` only.
- [ ] **Step 2: Verify.**
  - Run: `cargo test --locked --lib powerpcu::` → `6 passed`.
  - Run: `git diff src/server/connection.rs` → only added lines inside or just before those three arms.
- [ ] **Step 3: Commit.** Message: `feat(powerpcu): ignore chat, voice call and switch sides from peers`

### Task 4: Flutter UI hooks (§5.4 D1–D3)

**Files:**
- Create: `flutter/lib/common/powerpcu.dart`
- Modify: `flutter/lib/consts.dart`: add `const String kOptionPowerPcu = "powerpcu";` next to `kOptionHideElevateButtonInAcceptWindow`
- Modify: `flutter/lib/desktop/pages/desktop_home_page.dart`:
  - `buildLeftPane`: `if (!isOutgoingOnly) buildPasswordBoard(context),` → `if (!isOutgoingOnly && !isPowerPcuBuild()) buildPasswordBoard(context),`
  - `buildTip`: the `desk_tip` text condition `if (!isOutgoingOnly)` → `if (!isOutgoingOnly && !isPowerPcuBuild())`
- Modify: `flutter/lib/desktop/pages/server_page.dart`:
  - In the header `Offstage` of the chat/file icon (~line 557), extend `offstage:` with `|| (isPowerPcuBuild() && client.type_() != ClientType.file)`. The file-transfer log button stays.
  - In `_PrivilegeBoardState.build` non-camera list: prefix the `audio`, `restart` and `recording` `buildPermissionIcon(` entries with `if (!isPowerPcuBuild())`, and extend the existing `if (isWindows)` / `if (bind.mainSupportedPrivacyModeImpls() != '[]')` conditions with `&& !isPowerPcuBuild()`. Camera branch untouched.
- Modify: `flutter/lib/desktop/pages/desktop_setting_page.dart` About card: after the `Website` `InkWell`, insert `if (isPowerPcuBuild()) ...powerPcuAboutLines(linkStyle),`
- Add `import 'package:flutter_hbb/common/powerpcu.dart';` to the three pages.

**Interfaces:**
- Produces (`flutter/lib/common/powerpcu.dart`):
  - `bool isPowerPcuBuild()`, which returns `bind.mainGetBuildinOption(key: kOptionPowerPcu) == 'Y'`
  - `List<Widget> powerPcuAboutLines(TextStyle linkStyle)`, which returns exactly two widgets:
    - a `SelectionArea(child: Text('PowerPCU Remote — based on RustDesk · AGPL-3.0'))` with `.marginSymmetric(vertical: 4.0)`
    - an `InkWell` that launches `https://github.com/WorkDii/powerpcu_desk` via `launchUrlString`, with text `'Source code: https://github.com/WorkDii/powerpcu_desk'` in `linkStyle`
  - Plain strings, **not** `translate()`.
- Consumes: builtin option `powerpcu=Y` (Task 1)

- [ ] **Step 1: Create `powerpcu.dart` and the const.**
- [ ] **Step 2: Apply the D1–D3 edits listed above.**
- [ ] **Step 3: Verify.**
  - Run: `git diff --stat -- flutter` → only the five files listed.
  - Run: `git diff -- flutter` → every change is an added `isPowerPcuBuild()` condition, the import, the const, or the new file.
  - Dart compilation is verified by the Task 6 CI build. If `dart`/`flutter` is available locally, also run `flutter analyze lib/common/powerpcu.dart` → no issues.
- [ ] **Step 4: Commit.** Message: `feat(powerpcu): hide password board, chat button and cut permissions; add AGPL source link`

### Task 5: Logo assets and build-time branding (§5.5, logo C)

**Files:**
- Create: `res/powerpcu/logo.svg`
  - viewBox `0 0 24 24`
  - `<rect width="24" height="24" rx="5" fill="#004c28"/>`
  - ECG `M3.6 11.4 H8 L9.5 7.1 L11.5 15.8 L13.4 11.4 H17.5`: stroke `#fff`, width 2, round caps and joins
  - `<circle cx="17.8" cy="17.8" r="5.4" fill="#10b981"/>`
  - cursor `M16.1 15.3 L16.5 20.1 L17.9 18.7 L19.5 20.5 L20.5 19.6 L18.9 17.8 L20.7 17.5 Z`, fill `#fff`
- Create: `res/powerpcu/logo-small.svg`: same, stroke 2.1, circle r 5.6. Hand-tune the cursor so it reads at 16 px; show the owner the 16/20/24 renders before committing.
- Create: `res/powerpcu/wordmark.svg`: icon at left (48 px) plus the text `PowerPCU Remote`, `font-family="Segoe UI, sans-serif"`, weight 600, fill `#004c28`, canvas 600×120
- Create: `res/powerpcu/gen-icons.mjs`
  - Loads `@resvg/resvg-js` and `png-to-ico` through `createRequire(path.join(process.env.POWERPCU_ICON_TOOLS, 'package.json'))`, so nothing is added to the repo's package files.
  - Renders `logo-small.svg` for sizes ≤24 and `logo.svg` above, and writes `res/powerpcu/out/`:
    - `app_icon.ico` (16,20,24,32,40,48,64,256)
    - `icon.ico` (16,24,32,48,64,128,256)
    - `tray-icon.ico` (16,20,24,32)
    - `assets-icon.ico` (16,24,32,48,64,128,256)
    - `assets-icon.png` (256)
    - `icon.svg` (copy of `logo.svg`)
    - `logo.png` (wordmark at 600×120, system fonts loaded)
- Create: `res/powerpcu/out/*`: commit the generated files.
- Create: `res/powerpcu/apply-branding.ps1`
- Create: `res/powerpcu/README.md`: 10 lines max: how to regenerate icons and what `apply-branding.ps1` touches.

**Interfaces:**
- Produces: `pwsh res/powerpcu/apply-branding.ps1` (no args; run from the repo root). Exit 0 on success, non-zero with a message on any failed anchor. Consumed by Task 6.

**`apply-branding.ps1` behaviour:**
- **Copies:**
  - `out/app_icon.ico` → `flutter/windows/runner/resources/app_icon.ico`
  - `out/icon.ico` → `res/icon.ico`
  - `out/tray-icon.ico` → `res/tray-icon.ico`
  - `out/assets-icon.ico` → `flutter/assets/icon.ico`
  - `out/assets-icon.png` → `flutter/assets/icon.png`
  - `out/icon.svg` → `flutter/assets/icon.svg`
  - `out/logo.png` → `flutter/assets/logo.png`
- **Replaces in `flutter/windows/runner/Runner.rc`.** Read and write with `[IO.File]::ReadAllText` / `WriteAllText` and `UTF8Encoding($false)`, keeping CRLF:

| Find | Replace |
|---|---|
| `"CompanyName", "Purslane Tech Pte. Ltd."` | `"CompanyName", "WorkDii"` |
| `"FileDescription", "RustDesk Remote Desktop"` | `"FileDescription", "PowerPCU Remote"` |
| `"InternalName", "rustdesk"` | `"InternalName", "powerpcu-remote"` |
| `All rights reserved."` | `All rights reserved. Modified by WorkDii (AGPL-3.0)."` |
| `"OriginalFilename", "rustdesk.exe"` | `"OriginalFilename", "powerpcu-remote.exe"` |
| `"ProductName", "RustDesk"` | `"ProductName", "PowerPCU Remote"` |

- **Replaces in `libs/portable/Cargo.toml`** (same read/write rules):

| Find | Replace |
|---|---|
| `ProductName = "RustDesk"` | `ProductName = "PowerPCU Remote"` |
| `OriginalFilename = "rustdesk.exe"` | `OriginalFilename = "powerpcu-remote-x86_64.exe"` |
| `FileDescription = "RustDesk Remote Desktop"` | `FileDescription = "PowerPCU Remote"` |

- **Fails if a find string is missing.** Check every find string before writing anything, so a failure leaves files unchanged. Running the script twice therefore fails with a clear "already branded or upstream changed" message.

- [ ] **Step 1: Write the three SVGs and `gen-icons.mjs`.** Install the tools outside the repo, e.g. `npm --prefix "$SCRATCH/icon-tools" i @resvg/resvg-js@2.6.2 png-to-ico@2.1.8`, then run `POWERPCU_ICON_TOOLS="$SCRATCH/icon-tools" node res/powerpcu/gen-icons.mjs`.
  - Expected: the seven files in `res/powerpcu/out/`.
  - Each `.ico` lists the sizes above (check with a quick Node read of the ICONDIR header, or `file`).
- [ ] **Step 2: Render a contact sheet** (16/20/24/32/48/256 and `logo.png`) into an Orca chat visual and send it to the owner. Continue with the next steps meanwhile; apply any feedback before the PR is marked ready.
- [ ] **Step 3: Write `apply-branding.ps1`.**
- [ ] **Step 4: Verify the script.**
  - Run: `pwsh res/powerpcu/apply-branding.ps1; git status --short` → exit 0. Modified: `Runner.rc`, `app_icon.ico`, `res/icon.ico`, `res/tray-icon.ico`, `libs/portable/Cargo.toml`. Untracked or modified: the four `flutter/assets/` files.
  - Run: `git diff flutter/windows/runner/Runner.rc libs/portable/Cargo.toml` → exactly the 6 + 3 replacements.
  - Run the script a second time → non-zero exit, and `git diff` is unchanged.
  - Then restore: `git checkout -- flutter/windows/runner/Runner.rc flutter/windows/runner/resources/app_icon.ico res/icon.ico res/tray-icon.ico libs/portable/Cargo.toml`. Delete only the four `flutter/assets/` files the script created; check `git status` first that they were untracked.
- [ ] **Step 5: Commit** `res/powerpcu/**` only. Message: `feat(powerpcu): logo C assets and build-time branding script`

### Task 6: Release workflow, upstream workflows off, first CI build (§5.6, §9)

**Files:**
- Create: `.github/workflows/powerpcu-remote.yml`

**Workflow content:**
- **Name:** `PowerPCU Remote (Windows x64)`
- **Triggers:** `push: tags: ['remote-v*']`, `pull_request: branches: [master]`, `workflow_dispatch`.
- **Permissions:** top-level `contents: read`.
- **`env`**, copied verbatim from `flutter-build.yml`: `RUST_VERSION: "1.75"`, `LLVM_VERSION: "15.0.6"`, `FLUTTER_VERSION: "3.24.5"`, `VCPKG_BINARY_SOURCES: "clear;x-gha,readwrite"`, `VCPKG_COMMIT_ID: "9e593bb18ea69cc5095e012465dcd675a822ed0d"`.
- **`jobs.generate-bridge`:** `uses: ./.github/workflows/bridge.yml`
- **`jobs.build`:** `needs: generate-bridge`, `runs-on: windows-2022`. Steps:
  1. Copy from `flutter-build.yml` job `build-for-windows-flutter`, taking only the x64 path with the same pinned action SHAs:
     - cache-env export
     - checkout with `submodules: recursive`
     - download `bridge-artifact`
     - LLVM
     - Flutter (x64)
     - replace engine with `rustdesk/engine` `windows-x64-release.zip`
     - patch flutter (dropdown diff)
     - Rust toolchain `${{ env.RUST_VERSION }}`, target `x86_64-pc-windows-msvc`, rustfmt
     - rust-cache
     - run-vcpkg
     - vcpkg install `--triplet x64-windows-static` (the bash step as-is)
  2. `pwsh res/powerpcu/apply-branding.ps1`
  3. `cargo test --locked --release --lib --features flutter,hwcodec,vram,powerpcu powerpcu::`
  4. `cargo build --locked --features flutter,hwcodec,vram,powerpcu --lib --release`
  5. `python3 .\build.py --portable --flutter --hwcodec --vram --skip-cargo --skip-portable-pack`
  6. `mv ./flutter/build/windows/x64/runner/Release ./powerpcu-remote`, then `Rename-Item ./powerpcu-remote/rustdesk.exe powerpcu-remote.exe`
  7. Bash: `sed -i '/dpiAware/d' res/manifest.xml`; `pushd libs/portable`; `pip3 install -r requirements.txt`; `python3 ./generate.py -f ../../powerpcu-remote/ -o . -e ../../powerpcu-remote/powerpcu-remote.exe`; `popd`; `mkdir -p dist`; `mv ./target/release/rustdesk-portable-packer.exe dist/powerpcu-remote-x86_64.exe`
  8. Bash, in `dist`: `sha256sum powerpcu-remote-x86_64.exe > powerpcu-remote-x86_64.exe.sha256 && sha256sum -c powerpcu-remote-x86_64.exe.sha256`
  9. `actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a`, name `powerpcu-remote-x86_64`, path `dist/*`, `if-no-files-found: error`
- **Not copied:** arm64, usbmmidd, printer driver, TopMostWindow / `WindowInjection.dll`, signing, MSI, Runner.res copy.
- **`jobs.release`:**
  - `if: startsWith(github.ref, 'refs/tags/remote-v')`, `needs: build`, `runs-on: ubuntu-latest`, `permissions: contents: write`
  - Download the artifact, then `softprops/action-gh-release@de2c0eb89ae2a093876385947365aca7b0e5f844` with `tag_name: ${{ github.ref_name }}`, `files: dist/*`, `prerelease: false`.
  - The body includes the SHA-256 line (read from the `.sha256` file in a prior step), `Source: https://github.com/WorkDii/powerpcu_desk/tree/${{ github.ref_name }}` and `License: AGPL-3.0 (based on RustDesk)`.

- [ ] **Step 1: Write the workflow.** Validate syntax: `npx --yes @action-validator/cli .github/workflows/powerpcu-remote.yml` (or `actionlint` if available) → no errors.
- [ ] **Step 2: Disable upstream workflows on the fork.** The owner approved this on 2026-11-22.
  - Run: `gh workflow list --repo WorkDii/powerpcu_desk --all`. If it is empty, use `gh api repos/WorkDii/powerpcu_desk/actions/workflows`.
  - Then run `gh workflow disable <file> --repo WorkDii/powerpcu_desk` for: `ci.yml`, `flutter-ci.yml`, `flutter-nightly.yml`, `flutter-tag.yml`, `fdroid.yml`, `update-webpki-roots.yml`, `playground.yml`, `clear-cache.yml`.
  - Leave the reusable `bridge.yml`, `flutter-build.yml` and `third-party-RustDeskTempTopMostWindow.yml` alone.
  - Expected: those eight show `disabled_manually`.
  - If they are not registered until the branch is pushed, push first (a push to a non-master branch triggers none of them), then disable, then open the PR.
- [ ] **Step 3: Commit, push the branch, open a draft PR** to `WorkDii/powerpcu_desk:master`.
  - Title: `PowerPCU Remote (phase 1, WorkDii/powerpcu#386)`.
  - Body: summary, the regression surface table (spec §11), known limits (spec §7) and the manual checklist (spec §6). It ends with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`.
- [ ] **Step 4: Watch the `PowerPCU Remote (Windows x64)` run.**
  - Fix failures with the systematic-debugging skill, as new commits on the branch.
  - Expected: green, with artifact `powerpcu-remote-x86_64` containing both files.
  - Download it and run `sha256sum -c` locally → `OK`.
- [ ] **Step 5: Local smoke test** (this machine is Windows 11).
  - Run the artifact exe without elevation.
  - Confirm `%LOCALAPPDATA%\powerpcu-remote\powerpcu-remote.exe` exists and `%APPDATA%\PowerPCU-Remote\config\PowerPCU-Remote.toml` is created.
  - `powerpcu-remote.exe --get-id` prints a 10-digit ID starting with `1`.
  - Take a screenshot of the home window for the PR.
  - Do **not** install the service or touch any existing RustDesk on this machine without the owner's go-ahead; the full checklist in spec §6 is for the owner's test PC.
- [ ] **Step 6: Mark the PR ready for review** and report to the coordinator with the run link, artifact SHA-256, screenshots and checklist. **Do not merge.**

### Task 7: Final whole-branch review

- [ ] **Step 1: Dispatch a fresh reviewer** (superpowers:requesting-code-review) on `master...feat/powerpcu-remote-386` with the spec and this plan.
- [ ] **Step 2: Triage findings per AGENTS.md** ("Corner cases raised in review"). Fix crashes, consent bypasses and isolation breaks. List the rest as known limits in the PR body.
- [ ] **Step 3: Re-run CI if code changed.** Update the PR body's regression surface. Report.
