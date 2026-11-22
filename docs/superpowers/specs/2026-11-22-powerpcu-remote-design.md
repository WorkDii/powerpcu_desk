# PowerPCU Remote — เอกสารออกแบบ (เฟส 1 ของ WorkDii/powerpcu#386)

สถานะ: ร่างเพื่อขออนุมัติ · วันที่ 2026-11-22 · branch `feat/powerpcu-remote-386` · ฐาน RustDesk 1.5.3 (`534b04752`)

## 1. เป้าหมายและเหตุผล

ทีมดูแลของ PowerPCU ต้องรีโมตเข้าหน้าจอเครื่องของ รพ.สต. เพื่อช่วยเหลือ ทุกวันนี้ใช้ RustDesk ตัวมาตรฐาน
แล้วให้ PowerPCU Box เขียนทับ `%APPDATA%\RustDesk\config\RustDesk2.toml` ของผู้ใช้ให้ชี้ไปที่ relay ของเรา
(`powerpcu/server/internal/modules/remote/config.go`) ถ้าเครื่องนั้นมี RustDesk ที่ผู้ใช้ติดตั้งเองอยู่แล้ว
โปรแกรมสองตัวจะชนกันทั้งเรื่องการตั้งค่า, ID และ Windows Service

เฟส 1 คือสร้าง **PowerPCU Remote** จาก fork `WorkDii/powerpcu_desk`: โปรแกรมรับการรีโมตอย่างเดียว
มีตัวตนแยกจาก RustDesk 100% ฝังเซิร์ฟเวอร์ของเราไว้ในตัว และบังคับให้เจ้าของเครื่องกดยอมรับทุกครั้ง
ผลลัพธ์คือ exe ไฟล์เดียวบน GitHub Release ที่ Box จะดาวน์โหลดไปแคชและแจกต่อในเฟส 2

## 2. Goals / Non-goals / Anti-goals

**Goals**
- G1 ติดตั้งและรันคู่กับ RustDesk ของผู้ใช้ได้ โดยไม่แก้การตั้งค่า, ID, service หรือไดรเวอร์ของ RustDesk ตัวนั้น
- G2 รับการรีโมตเท่านั้น ใช้ได้แค่ดูจอ, คุมเมาส์/คีย์บอร์ด, Clipboard และ File Transfer
- G3 เจ้าของเครื่องต้องกด "ยอมรับ" ทุกครั้ง ทั้งโหมด portable และโหมด service ไม่มีรหัสผ่านถาวร ไม่มี unattended access
- G4 ใช้ `rustdesk.powerpcu.com` + key ของเราเสมอ ผู้ใช้เปลี่ยนเซิร์ฟเวอร์จาก UI, CLI หรือชื่อไฟล์ไม่ได้
- G5 ไฟล์ exe ตัวเดียวรันแบบ portable ได้โดยไม่ต้องใช้สิทธิ์ Admin และกดติดตั้งเป็น Windows Service ได้เมื่อมีสิทธิ์ Admin
- G6 แบรนด์ PowerPCU Remote (โลโก้แบบ C) ทุกจุดที่ผู้ใช้เห็นบน Windows
- G7 CI สร้าง `powerpcu-remote-x86_64.exe` + `powerpcu-remote-x86_64.exe.sha256` บน GitHub Release อัตโนมัติ
- G8 ปฏิบัติตาม AGPL-3.0: คงไฟล์ `LICENCE` และแสดงลิงก์ซอร์ส `https://github.com/WorkDii/powerpcu_desk`

**Non-goals (เฟสนี้ยังไม่ทำ)**
- แพลตฟอร์มอื่นนอกจาก Windows x64 (macOS, Linux, Android, iOS, web, ARM64, 32-bit)
- แก้ฝั่ง PowerPCU Box/Launcher (เปลี่ยน URL/SHA, เลิกเขียน config ของ RustDesk, ย้ายเครื่องเก่า) — เป็นเฟส 2
- บันทึกว่าใครรีโมตเข้าเครื่องไหนเมื่อไร (ข้อกำหนดใน #386) — ต้องมี API server ซึ่ง hbbs ฉบับ OSS ไม่มี จึงแยกไปทำภายหลัง
- MSI installer, code signing, ระบบอัปเดตตัวเองของ PowerPCU Remote (Box เป็นผู้แจกเวอร์ชันใหม่)

**Anti-goals (ถ้าเกิดเรื่องเหล่านี้ ถือว่าล้มเหลวแม้โปรแกรมทำงานได้)**
- มีทางใดก็ตามที่เข้าเครื่องได้โดยเจ้าของไม่ได้กดยอมรับ
- ติดตั้งหรือถอน PowerPCU Remote แล้วทำให้ RustDesk ของผู้ใช้เสีย (ไดรเวอร์ปริ้นเตอร์, จอเสมือน, service, config)
- โปรแกรมถูกอัปเดตทับด้วย RustDesk ตัวมาตรฐานจาก rustdesk.com/GitHub ของ RustDesk
- แก้ submodule `libs/hbb_common` หรือรื้อโค้ดเดิมจน merge RustDesk รุ่นใหม่ลำบาก

## 3. ข้อจำกัด

- ต้องทำตาม `AGENTS.md` ของ repo: diff เล็กที่สุด, เพิ่มโค้ดใหม่แทนการรื้อของเดิม, call site ในไฟล์กลางเป็น hook บรรทัดเดียว,
  **ปิด feature แล้วต้องได้โค้ดเดิมทุกบรรทัด**, ไม่ `unwrap()` ในโค้ด production, log ที่เกิดซ้ำต่อข้อความจาก peer ห้ามใช้ระดับ `debug` ขึ้นไป
- ห้ามรัน `flutter_rust_bridge_codegen` ในเครื่อง และไม่เพิ่มฟังก์ชัน FFI ใหม่ (ใช้ `mainGetBuildinOption` ที่มีอยู่แล้ว)
- ชื่อแอปที่ติดตั้งได้ต้องตรง `[A-Za-z0-9_-]` (`src/platform/mod.rs:42`) และชื่อ exe ต้องเท่ากับชื่อแอป (ไม่สนตัวพิมพ์เล็ก/ใหญ่) (`src/platform/windows.rs:1508`)
- CI ฝั่ง Windows ใช้ Rust 1.75, Flutter 3.24.5, Flutter engine ของ RustDesk, vcpkg triplet `x64-windows-static` เหมือนต้นทาง
- เครื่องพัฒนาไม่มี Flutter และ vcpkg จึงยืนยันการ build เต็มรูปแบบได้ใน CI เท่านั้น
- ไม่มีใบรับรอง code signing (exe ไม่ได้เซ็น)

## 4. แนวทางที่พิจารณา (อนุมัติแล้ว 2026-11-22)

| แนวทาง | สรุป | ผล |
|---|---|---|
| **1. Cargo feature `powerpcu` + โมดูลใหม่ `src/powerpcu.rs`** | ตั้ง `APP_NAME` และ map การตั้งค่าใน `hbb_common::config` (เป็น `pub` อยู่แล้ว) ตอนเริ่มโปรแกรม จุดเดียวกับที่ `read_custom_client` ใช้ ส่วนจุดอื่นเป็น hook สั้น ๆ ครอบด้วย `#[cfg(feature = "powerpcu")]` | **เลือก** ค่าฝังใน binary, ไม่แตะ submodule, ปิด feature = RustDesk เดิม |
| 2. `custom.txt` เซ็นด้วยกุญแจของเราเอง | เปลี่ยน public key ใน `read_custom_client` แล้วให้ CI เซ็นไฟล์ | ตัดทิ้ง: ถ้าไฟล์หาย โปรแกรมกลายเป็น "RustDesk" แล้วชนทันที และต้องเก็บ private key ใน CI |
| 3. Fork `hbb_common` | แก้ค่าเริ่มต้นใน submodule | ตัดทิ้ง: ต้องดูแลสอง repo และยังต้องมี hook ใน crate หลักอยู่ดี |

การตัดฟีเจอร์ใช้วิธี "ล็อกค่าให้ฝั่ง server ปฏิเสธจริง แล้วค่อยซ่อน UI" ไม่ลบโค้ด
ส่วน CI แยกเป็น workflow ใหม่ ไม่แก้ `flutter-build.yml` เดิม (เจ้าของอนุมัติทั้งสองเรื่อง)

## 5. การออกแบบ

### 5.1 ตัวตนของโปรแกรม (เจ้าของเลือกแบบ ก.)

`APP_NAME = "PowerPCU-Remote"` ค่าทั้งหมดในตารางนี้มาจากชื่อนี้อัตโนมัติ ไม่ต้องเขียนโค้ดเพิ่ม

| สิ่ง | ค่า | ที่มา |
|---|---|---|
| exe ที่ติดตั้ง/แพ็กไว้ | `powerpcu-remote.exe` | เปลี่ยนชื่อหลัง `flutter build` (5.6) |
| Windows Service | `PowerPCU-Remote` (display name `PowerPCU-Remote Service`) | `start_os_service` / `service_control.h` |
| โฟลเดอร์ติดตั้ง | `%ProgramFiles%\PowerPCU-Remote\` | `windows.rs:1436` |
| ตั้งค่า (ผู้ใช้) | `%APPDATA%\PowerPCU-Remote\config\` | `Config::path` |
| Named pipe | `\\.\pipe\PowerPCU-Remote\query*` | `Config::ipc_path` |
| URI / file type | `powerpcu-remote://`, `HKCR\powerpcu-remote` | `get_uri_prefix()` |
| Uninstall key, shortcut, firewall rule, tray mutex | `PowerPCU-Remote…` | `install_me` |
| โฟลเดอร์แตกไฟล์ของ portable | `%LOCALAPPDATA%\powerpcu-remote\` | `libs/portable` `app_dir_name()` จากชื่อ exe ข้างใน |
| ชื่อหน้าต่าง / tray / ข้อความ UI | `PowerPCU-Remote` (ข้อความที่มีคำว่า RustDesk ถูกแทนอัตโนมัติ) | `lang.rs:274` |

### 5.2 การตั้งค่าที่ฝังในตัว — `src/powerpcu.rs` (ไฟล์ใหม่)

- `Cargo.toml`: เพิ่ม `powerpcu = []` ใน `[features]` (ไม่ใส่ใน `default`) และใน `src/lib.rs`
  เพิ่ม `#[cfg(feature = "powerpcu")] mod powerpcu;`
- แยกเป็นฟังก์ชันบริสุทธิ์ที่คืนค่าชุดการตั้งค่า เพื่อให้ทดสอบได้โดยไม่ต้องแตะ global
  กับฟังก์ชัน `apply()` ที่เขียนลง `config::APP_NAME`, `HARD_SETTINGS`, `OVERWRITE_SETTINGS`, `OVERWRITE_LOCAL_SETTINGS`,
  `DEFAULT_LOCAL_SETTINGS`, `BUILTIN_SETTINGS`
- Hook: บรรทัดแรกของ `load_custom_client()` (`src/common.rs:2518`) คือ
  `#[cfg(feature = "powerpcu")] { crate::powerpcu::apply(); return; }`
  ฟังก์ชันนี้ถูกเรียกจาก `core_main` (ทุก process native: `--service`, `--server`, `--cm`, `--tray`, หน้าต่างหลัก)
  และจาก `flutter_ffi::initialize` ซึ่งบน desktop ส่ง `custom_client_config` เป็นค่าว่างเสมอ (`native_model.dart:239`)
  การ `return` ทำให้ `custom.txt` (ทั้งแบบเซ็นและ `./custom.txt` ตอน debug) ถูกข้าม จึงไม่มีใครเพิ่มค่า HARD/BUILTIN ได้
- ค่าเซิร์ฟเวอร์และ key อยู่เป็น `const` ในไฟล์นี้ไฟล์เดียว เปลี่ยนเซิร์ฟเวอร์ต้อง build ใหม่ (ตั้งใจให้เป็นแบบนั้น)
- **ID ของตัวเอง (สำคัญด้านความปลอดภัย):** ปกติ RustDesk สร้าง ID จาก MAC address (`Config::get_auto_id`, `hbb_common/src/config.rs:1185`)
  และสร้างแค่ครั้งแรกที่ ID ยังว่าง (`Config::get_id`, `:1339`) ถ้าปล่อยไว้ PowerPCU Remote จะได้ **ID เดียวกับ RustDesk ของผู้ใช้**
  ถ้า RustDesk ตัวนั้นลงทะเบียนกับ `rustdesk.powerpcu.com` ด้วย (เกิดได้จริง เพราะ Box ปัจจุบันเขียนเซิร์ฟเวอร์ของเราลง config ของ RustDesk)
  hbbs จะรับทั้งสองตัวเพราะ IP เดียวกัน แล้วทีมอาจเชื่อมต่อไปโดน RustDesk ของผู้ใช้ ซึ่งอาจมีรหัสผ่านถาวรและไม่ต้องกดยอมรับ
  - ทางแก้: ใน `apply()` หลังตั้ง `APP_NAME` แล้ว ถ้า ID ปัจจุบัน**ไม่ใช่เลขในช่วง `1_000_000_000..2_000_000_000`** (`needs_own_id`) ให้ `Config::set_id(<ID ที่คำนวณ>)`
    (ตรวจ "ID ว่าง" ไม่ได้ เพราะ `Config::load()` เติม ID จาก MAC ให้ทันทีที่โหลด config ที่ยังไม่มี ID — `config.rs:774` — พบตอนทดสอบ build แรก)
  - ID ที่คำนวณ: `sha256(hbb_common::get_uuid() ‖ MAC ‖ "PowerPCU-Remote")` (ผสม MAC เพราะเครื่องที่โคลนดิสก์โดยไม่ sysprep มี machine GUID ซ้ำกัน; คำนวณครั้งแรกครั้งเดียวแล้วเก็บใน config) แปลงเป็นเลขในช่วง `1_000_000_000..2_000_000_000`
    - ช่วงนี้ไม่ทับกับ ID ที่ได้จาก MAC (ค่าสูงสุดคือ `0x1FFFFFFF` ≈ 5.4×10⁸)
    - ทุก process (UI/server/service) คำนวณได้ค่าเดียวกัน แข่งกันเขียนก็ไม่ขัดกัน
    - ติดตั้งใหม่หรือล้าง config บนเครื่องเดิมก็ได้ ID เดิม
  - ถ้าบังเอิญชนกับเครื่องอื่น hbbs ส่ง `UUID_MISMATCH` แล้วโค้ดเดิม (`rendezvous_mediator.rs:1328`) สุ่ม ID ใหม่ในช่วงเดียวกันให้
    รอบถัดไป ID อยู่ในช่วงแล้ว จึงไม่ถูกเขียนกลับ (ไม่วนลูป และไม่มี state ใหม่)

| Map | Key | ค่า | ผล |
|---|---|---|---|
| `APP_NAME` | — | `PowerPCU-Remote` | ตัวตนตาม 5.1 |
| HARD | `conn-type` | `incoming` | หน้าจอแบบรับอย่างเดียว, ปฏิเสธการเชื่อมต่อขาออก (`client.rs:436`) |
| HARD | `disable-account` | `Y` | ไม่มีแท็บ Account / การ login |
| HARD | `disable-ab` | `Y` | ไม่มี Address Book |
| HARD | `disable-settings` | `Y` | ซ่อนแท็บ Security/Network/Printer, ไอคอนแก้รหัสผ่าน, บล็อก CLI `--password`/`--option`/`--set-id` |
| HARD | `password`, `salt`, `disable-installation` | **ไม่ตั้ง** | ไม่มีรหัสผ่านถาวรที่ฝังไว้, ยังติดตั้งได้ |
| OVERWRITE | `custom-rendezvous-server` | `rustdesk.powerpcu.com` | ล็อกเซิร์ฟเวอร์ (บันทึกทับไม่ได้) |
| OVERWRITE | `key` | `Yl+eLj+8+MvlyMQcg117JyVvyy1ojHSGaY2+N7aYmO8=` | ล็อก key |
| OVERWRITE | `relay-server`, `api-server` | `""` | ล็อกเป็นค่าว่าง: ใช้ relay ที่ hbbs ให้มา และ `--config` ตั้งค่าเองไม่ได้ |
| OVERWRITE | `approve-mode` | `click` | รับได้ด้วยการกดเท่านั้น (`connection.rs:3078`) |
| OVERWRITE | `verification-method` | `use-temporary-password` | ไม่มีการตรวจรหัสผ่านถาวรเลย (ป้องกันอีกชั้น) |
| OVERWRITE | `allow-hide-cm` | `N` | หน้าต่างรับการเชื่อมต่อซ่อนไม่ได้ |
| OVERWRITE | `access-mode` | `custom` | ให้สิทธิ์รายข้อด้านล่างมีผล (`full` จะข้ามทุกข้อ) |
| OVERWRITE | `enable-keyboard`, `enable-clipboard`, `enable-file-transfer` | `Y` | ฟีเจอร์ที่เก็บไว้ |
| OVERWRITE | `enable-audio`, `enable-camera`, `enable-terminal`, `enable-tunnel`, `enable-remote-printer`, `enable-remote-restart`, `enable-record-session`, `enable-block-input`, `enable-privacy-mode` | `N` | ฟีเจอร์ที่ตัดออก ฝั่ง server ปฏิเสธตอน login หรือปิดในเซสชัน |
| OVERWRITE | `allow-remote-config-modification` | `N` | ฝั่งรีโมตแก้การตั้งค่าเครื่องเราไม่ได้ |
| OVERWRITE | `enable-lan-discovery` | `N` | ไม่ตอบ LAN discovery |
| OVERWRITE | `direct-server` | `N` | ไม่เปิดพอร์ต 21118 สำหรับ direct IP |
| OVERWRITE | `enable-trusted-devices` | `N` | ป้องกันอีกชั้น |
| OVERWRITE | `allow-auto-update` | `N` | **จำเป็น** — Windows auto-updater ไม่ได้กัน custom client ไว้ |
| OVERWRITE | `allow-websocket`, `allow-insecure-tls-fallback` | `N` | ลดช่องทางที่ไม่ใช้ |
| OVERWRITE_LOCAL | `enable-check-update` | `N` | ไม่เช็กอัปเดตกับ rustdesk.com |
| OVERWRITE_LOCAL | `allow-remote-cm-modification` | `N` | เมาส์ของฝั่งรีโมตกด "ยอมรับ" ในหน้าต่าง CM ไม่ได้ |
| DEFAULT_LOCAL | `lang` | `th` | ภาษาเริ่มต้นเป็นไทย (ผู้ใช้เปลี่ยนได้) |
| BUILTIN | `hide-security-settings`, `hide-network-settings`, `hide-server-settings`, `hide-proxy-settings`, `hide-websocket-settings`, `hide-remote-printer-settings` | `Y` | ซ่อนส่วนตั้งค่าที่ไม่ใช้ |
| BUILTIN | `hide-powered-by-me` | `Y` | ซ่อนลิงก์ "Powered by RustDesk" (การให้เครดิตย้ายไปอยู่หน้า About, 5.4) |
| BUILTIN | `disable-change-permanent-password`, `disable-change-id`, `disable-unlock-pin` | `Y` | ปิดการตั้งรหัสผ่านถาวร / เปลี่ยน ID |
| BUILTIN | `register-device` | `N` | `get_api_server()` คืนค่าว่าง จึงไม่ยิง heartbeat/sysinfo/audit ไป `:21114` ทุก 15 วินาที |
| BUILTIN | `allow-logon-screen-password` | **ไม่ตั้ง** | เป็นทางรหัสผ่านทางเดียวที่เหลือในโหมด click ต้องปิดไว้ |
| BUILTIN | `powerpcu` | `Y` | ธงให้ฝั่ง Dart รู้ว่าเป็น PowerPCU build (5.4) — เพิ่ม `OPTION_POWERPCU` ใน `libs/base/src/config/keys.rs` |

ค่า OVERWRITE ทำให้ปุ่มในหน้าตั้งค่าเป็นสีเทา (`is_option_fixed`) และ `set_option` บันทึกไม่ได้
ค่าที่ Pro server ส่งมาทาง heartbeat ก็ทับไม่ได้ด้วย

### 5.3 Hook ฝั่ง Rust (ทุกจุดครอบด้วย `#[cfg(feature = "powerpcu")]` และเป็นการเพิ่มบรรทัดเท่านั้น)

| # | ไฟล์ / จุด | เปลี่ยนอะไร | เหตุผล |
|---|---|---|---|
| R1 | `src/common.rs` `load_custom_client()` | เรียก `powerpcu::apply()` แล้ว return | ฉีดค่าตาม 5.2 |
| R2 | `src/platform/windows.rs` `get_license_from_exe_name()` (`:2212`) | คืน `Err` ทันที | ชื่อไฟล์แบบ `…host=…,key=…` หรือ env `RUSTDESK_APPNAME` ชนะค่าที่ล็อกไว้ (`bootstrap()`, `get_custom_rendezvous_server`, `get_key`, `get_api_server_`) จุดนี้จุดเดียวปิดได้ทุกทาง |
| R3/R4 | `src/core_main.rs` ต้น chain คำสั่ง Windows (`if args[0] == "--uninstall"`) | ถ้า `powerpcu::is_shared_driver_command(args[0])` ให้ `return None` (คำสั่ง `--install-remote-printer`, `--uninstall-remote-printer`, `--uninstall-cert`, `--uninstall-amyuni-idd`, `--install-idd`) | สคริปต์ติดตั้ง/ถอน/อัปเดตเรียก exe ของเราเองด้วยคำสั่งเหล่านี้ ถ้าปล่อยไว้จะลบ "RustDesk v4 Printer Driver", ไดรเวอร์จอเสมือน (amyuni/usbmmidd) และใบรับรองทดสอบไดรเวอร์ระดับเครื่อง ที่ RustDesk ของผู้ใช้ใช้อยู่ จุดเดียวนี้ครอบทุกสคริปต์โดยไม่ต้องแก้ข้อความสคริปต์ |
| R5 | `src/platform/windows.rs` `get_valid_subkey()` (`:1351`) | ข้ามการค้น `IS1` (GUID Inno Setup ของ RustDesk รุ่นเก่า) | ไม่อย่างนั้นบนเครื่องที่มี RustDesk แบบ Inno รุ่นเก่า PowerPCU จะเข้าใจว่าโฟลเดอร์ของ RustDesk เป็นของตัวเอง แล้วถอนผิดตัวได้ |
| R6 | `src/updater.rs` `check_update()` (`:189`) | return `Ok(())` ทันที | กันอีกชั้นนอกจาก `allow-auto-update=N`: updater ดาวน์โหลดจาก `github.com/rustdesk` เท่านั้น ถ้าทำงานจะได้ RustDesk ตัวมาตรฐานทับ |
| R7 | `src/server/connection.rs` arm `misc::Union::ChatMessage` (`:3966`) | ทิ้งข้อความ ไม่ส่งต่อไป CM | ตัด Chat ฝั่ง server (ไม่มีตัวเลือกให้ปิด) |
| R8 | `src/server/connection.rs` arm `message::Union::VoiceCallRequest` (`:4142`) | ทิ้ง | ตัด voice call (ไม่มีตัวเลือกให้ปิด) |
| R9 | `src/server/connection.rs` arm `misc::Union::SwitchSidesRequest` (`:4040`) | ทิ้ง | switch sides จะเปิด `--connect` ขาออก ซึ่งขัดกับโหมดรับอย่างเดียว |

| R10 | `src/virtual_display_manager.rs` `is_virtual_display_supported()` (`:26`), `plug_out_monitor()`, `get_platform_additions()` | คืน `false` / `Ok(())` / map ว่าง | ปิดจอเสมือนทั้งหมด: ฝั่งรีโมตเสียบจอเสมือนไม่ได้ (`connection.rs:4840` ตอบว่าไม่รองรับ), ถอดจอเสมือน amyuni ของ RustDesk ผู้ใช้ไม่ได้, ไม่โฆษณาเมนูจอเสมือนให้ฝั่งรีโมต และไม่เสียบจอ headless อัตโนมัติ (`display_service.rs:989`) |
| R12 | `src/platform/windows.rs` `try_kill_broker()` | return ทันที | ทุกครั้งที่ server เริ่ม (`server.rs:737`) โค้ดเดิมสั่ง `taskkill RuntimeBroker_rustdesk.exe` ซึ่งจะปิด privacy mode ของ RustDesk ผู้ใช้; PowerPCU ไม่ใช้ broker (`enable-privacy-mode=N`) |
| R11 | `src/virtual_display_manager.rs` `reset_all()` (`:124`) | คืน `Ok(())` ทันที | ตอนเซสชันสุดท้ายจบ (`connection.rs:7220`) โค้ดเดิมจะถอดจอเสมือน amyuni **ทุกจอในเครื่อง** รวมถึงจอที่ RustDesk ของผู้ใช้กำลังใช้ |

R7–R9 รับข้อความที่ peer ควบคุมจังหวะได้ ถ้าจะ log ให้ใช้ `log::trace!` เท่านั้น

### 5.4 Hook ฝั่ง Flutter (ซ่อน UI เมื่อ `powerpcu=Y`)

- ไฟล์ใหม่ `flutter/lib/common/powerpcu.dart`: `bool isPowerPcuBuild() => bind.mainGetBuildinOption(key: kOptionPowerPcu) == 'Y';`
  (`mainGetBuildinOption` มีอยู่แล้วทั้ง native และ web bridge จึงไม่ต้องเพิ่ม FFI) และเพิ่มค่าคงที่ `kOptionPowerPcu` ข้างค่าคงที่ option อื่น
- ทุกจุดด้านล่างใช้รูปแบบ `if (!isPowerPcuBuild()) <widget เดิม>` ถ้าไม่ได้ตั้งธงนี้ UI จะเหมือนเดิมทุกอย่าง

| # | ไฟล์ | เปลี่ยนอะไร |
|---|---|---|
| D1 | `flutter/lib/desktop/pages/desktop_home_page.dart` | ซ่อนแผงรหัสผ่านชั่วคราว (โหมด click แสดงเป็น "-") และข้อความ `desk_tip` ที่พูดถึงรหัสผ่าน เหลือ ID กับสถานะ |
| D2 | `flutter/lib/desktop/pages/server_page.dart` (หน้าต่าง CM) | ซ่อนปุ่ม Chat ที่หัวหน้าต่าง (ปุ่มดู log การโอนไฟล์ยังอยู่) และซ่อนไอคอนสิทธิ์ของฟีเจอร์ที่ตัดออก (เสียง, ปริ้นเตอร์, กล้อง, รีสตาร์ต, บันทึกหน้าจอ, block input) เหลือ คีย์บอร์ด/เมาส์, Clipboard, ไฟล์ |
| D3 | `flutter/lib/desktop/pages/desktop_setting_page.dart` การ์ด About | เพิ่มบรรทัด "PowerPCU Remote — พัฒนาต่อจาก RustDesk · AGPL-3.0" และลิงก์ `https://github.com/WorkDii/powerpcu_desk` เป็นข้อความตรง ๆ **ไม่ผ่าน `translate()`** เพราะ `translate()` จะแทนคำว่า RustDesk ด้วยชื่อแอป (คงข้อความลิขสิทธิ์เดิมของ RustDesk ไว้ตามเงื่อนไข AGPL) |

ไม่ต้องแก้ส่วน Chat ฝั่ง Dart นอกจากปุ่ม เพราะ R7 ทิ้งข้อความตั้งแต่ server จึงไม่มีอะไรไปเปิดหน้าแชตอัตโนมัติ

### 5.5 แบรนด์และไอคอน (`res/powerpcu/`)

- **ต้นฉบับเวกเตอร์ (commit):** `res/powerpcu/logo.svg` (viewBox 24, โลโก้แบบ C: สี่เหลี่ยมมุมโค้ง `rx=5` สี `#004c28`,
  เส้น ECG สีขาวจากพิกัดใน `powerpcu/web/public/favicon.svg` ขยับขึ้นซ้ายตามต้นแบบ
  `M3.6 11.4 H8 L9.5 7.1 L11.5 15.8 L13.4 11.4 H17.5`, ป้ายวงกลม `#10b981` ที่ (17.8,17.8) r≈5.4 และลูกศรเคอร์เซอร์สีขาว)
  กับ `res/powerpcu/logo-small.svg` สำหรับ ≤24px (เส้นหนาขึ้น เช่น stroke 2.1, ป้าย r≈5.6) และ `res/powerpcu/wordmark.svg`
  (ไอคอน + ตัวอักษร "PowerPCU Remote" ด้วยฟอนต์ Segoe UI สำหรับโลโก้หน้าแรก; commit ไฟล์ PNG ที่ render แล้ว CI จึงไม่ขึ้นกับฟอนต์)
- **สคริปต์สร้างไฟล์ (commit):** `res/powerpcu/gen-icons.mjs` ใช้ Node + `@resvg/resvg-js` + `png-to-ico` (ลงใน scratch ตอนรัน ไม่เพิ่มเป็น dependency ของ repo)
  แล้ว commit ไฟล์ผลลัพธ์ทั้งหมดใน `res/powerpcu/out/` เพื่อให้ CI ไม่ต้องใช้ Node
- **ไฟล์ผลลัพธ์และที่ใช้:**

| ไฟล์ผลลัพธ์ | ขนาด | คัดลอกไปที่ (ตอน build) | ใช้โดย |
|---|---|---|---|
| `app_icon.ico` | 16,20,24,32,40,48,64,256 | `flutter/windows/runner/resources/app_icon.ico` | ไอคอนของ `powerpcu-remote.exe`, Explorer, shortcut |
| `icon.ico` | 16,24,32,48,64,128,256 | `res/icon.ico` | ไอคอนของไฟล์ portable ที่ดาวน์โหลด (`libs/portable/build.rs`) |
| `tray-icon.ico` | 16,20,24,32 | `res/tray-icon.ico` | tray (กรณีไม่มี `assets/icon.png`) |
| `assets-icon.ico` | 16..256 | `flutter/assets/icon.ico` | ไอคอนหน้าต่าง (`win32_window.cpp`) |
| `assets-icon.png` | 256 | `flutter/assets/icon.png` | tray, `loadIcon()` |
| `icon.svg` | — | `flutter/assets/icon.svg` | `loadIcon()` fallback |
| `logo.png` | ≤600×120 (แสดง ≤300×60) | `flutter/assets/logo.png` | โลโก้หน้าแรก |

(`libs/portable/src/res/label.png` เป็นแค่คำว่า "Loading..." ไม่มีแบรนด์ จึงไม่เปลี่ยน)

- **สคริปต์ใส่แบรนด์ตอน build:** `res/powerpcu/apply-branding.ps1` (CI รัน, นักพัฒนารันเองได้)
  คัดลอกไฟล์ตามตาราง แล้วแก้ข้อความ version resource ใน `flutter/windows/runner/Runner.rc`
  (`ProductName`/`FileDescription` = `PowerPCU Remote`, `InternalName` = `powerpcu-remote`, `OriginalFilename` = `powerpcu-remote.exe`,
  `CompanyName` = `WorkDii`, `LegalCopyright` คงของ RustDesk แล้วต่อท้าย "Modified by WorkDii, AGPL-3.0")
  และ `[package.metadata.winres]` ใน `libs/portable/Cargo.toml` ด้วยค่าชุดเดียวกัน
  ทุกการแทนที่ต้องตรวจว่าเจอข้อความต้นทางจริง ถ้าไม่เจอให้ fail (กันกรณี upstream เปลี่ยนแล้วแบรนด์หายเงียบ ๆ)
  ไฟล์ใน repo จึงยังเป็นของ RustDesk เดิม ปิด feature/ไม่รันสคริปต์ = build แบบ RustDesk เดิม

### 5.6 Build และ CI — `.github/workflows/powerpcu-remote.yml` (ไฟล์ใหม่)

- **Trigger:**
  - `push: tags: ['remote-v*']` → build + Release
  - `pull_request` ที่แตะไฟล์ PowerPCU (`src/powerpcu.rs`, `res/powerpcu/**`, `flutter/lib/common/powerpcu.dart`, ตัว workflow เอง) → build + เก็บเป็น artifact ไม่ออก Release
  - `workflow_dispatch`
  - tag `remote-v*` ไม่ตรง pattern ของ `flutter-tag.yml` (`v1.2.3`, `1.2.3`, `v1.2.3-4`)
- **Jobs:**
  - เรียก `bridge.yml` (reusable, `workflow_call`) เพื่อสร้างไฟล์ bridge เหมือนต้นทาง
  - job Windows x64 ตัวเดียว คัดลอกขั้นตอนจาก job `build-for-windows-flutter` ใน `flutter-build.yml` (toolchain, Flutter engine ของ RustDesk, vcpkg)
  - ตัดขั้นตอนที่ไม่ใช้ออก: arm64, usbmmidd, printer driver, `WindowInjection.dll`, การเซ็น, MSI
- **ลำดับ build:**
  1. `res/powerpcu/apply-branding.ps1`
  2. `cargo test --locked --lib --features flutter,powerpcu powerpcu::` (ให้เทสต์ใน 5.2 ถูกรันจริง)
  3. `cargo build --locked --features flutter,hwcodec,vram,powerpcu --lib --release`
  4. `python3 build.py --portable --flutter --hwcodec --vram --skip-cargo --skip-portable-pack` (ไม่แก้ `build.py`)
  5. เปลี่ยนชื่อ `rustdesk.exe` → `powerpcu-remote.exe` ในโฟลเดอร์ Release
  6. `libs/portable/generate.py -f <dir> -o . -e <dir>/powerpcu-remote.exe` → `powerpcu-remote-x86_64.exe`
  7. สร้าง `powerpcu-remote-x86_64.exe.sha256` (รูปแบบ `sha256sum`: `<hex>  powerpcu-remote-x86_64.exe`)
- **Release:**
  - `softprops/action-gh-release` ด้วย `permissions: contents: write`
  - ชื่อ tag แบบ `remote-v1.5.3-1` (เลขรุ่น RustDesk + ลำดับ build ของเรา) เวอร์ชันภายในยังเป็น 1.5.3 เพื่อให้คุยกับ peer ได้ตามปกติ
  - คำอธิบาย Release ใส่ SHA-256, ลิงก์ซอร์สของ tag นั้น และระบุว่าเป็น AGPL-3.0
- Workflow เดิมของ RustDesk ไม่แก้ไฟล์ แต่จะทำงานบน fork ด้วย (ดู 9.1)

### 5.7 การทำงานจริงบนเครื่องลูก

```
Box ดาวน์โหลด → powerpcu-remote-x86_64.exe (portable packer)
   └─ แตกไฟล์ไปที่ %LOCALAPPDATA%\powerpcu-remote\ → รัน powerpcu-remote.exe (สิทธิ์ผู้ใช้ปกติ)
        ├─ โหมด portable: ID ของตัวเอง (5.2, ไม่ซ้ำ RustDesk ของผู้ใช้), ทุกการเชื่อมต่อขึ้นหน้าต่าง CM ให้กด ยอมรับ/ปฏิเสธ
        │     └─ ถ้ามีสิทธิ์ Admin กดปุ่ม Elevate ใน CM ได้ (portable service ชั่วคราว) เพื่อให้ฝั่งรีโมตจัดการหน้าต่าง UAC
        └─ กด "ติดตั้ง" บนหน้าแรก → UAC → %ProgramFiles%\PowerPCU-Remote\ + service PowerPCU-Remote
              └─ โหมด service: CM ยังขึ้นในเซสชันของผู้ใช้ และยังต้องกดยอมรับทุกครั้ง (approve-mode=click ถูกล็อก)
```

ถ้าไม่มีผู้ใช้ login หรือหน้าจอถูกล็อก จะไม่มีใครกดยอมรับได้ ฝั่งรีโมตจะรอจนหมดเวลา (180 วินาที) แล้วหลุด
แปลว่าไม่มี unattended access ตามที่ตั้งใจ

### 5.8 สัญญาอนุญาต AGPL-3.0

- ไม่แตะ `LICENCE` และคงข้อความลิขสิทธิ์เดิมของ RustDesk ในหน้า About
- หน้า About (D3), version resource และคำอธิบาย GitHub Release ระบุลิงก์ซอร์ส `https://github.com/WorkDii/powerpcu_desk` และ AGPL-3.0
- Repo เป็น public ซอร์สของทุก tag ที่ออก Release ดาวน์โหลดได้จาก GitHub

## 6. การทดสอบและการยืนยันผล

- **Rust unit tests** (`#[cfg(test)]` ใน `src/powerpcu.rs`) ประกาศโมดูลด้วย `#[cfg(any(feature = "powerpcu", test))]`
  ส่วน `apply()` ที่เขียน global ให้ครอบด้วย `#[cfg(feature = "powerpcu")]` อย่างเดียว ตามกติกาใน `AGENTS.md`
  เทสต์ถูกรันใน workflow `powerpcu-remote.yml` (CI เดิมของ RustDesk ถูกปิดบน fork ตามข้อ 9) และจะรันใน `cargo test --workspace` เองถ้าเปิด CI เดิมกลับมา:
  - ฟังก์ชันคำนวณ ID ให้ค่าคงที่สำหรับ uuid เดียวกัน อยู่ในช่วง `1_000_000_000..2_000_000_000` และต่างกันเมื่อ uuid ต่างกัน
  - `APP_NAME` มีแต่อักขระ `[A-Za-z0-9_-]` (กติกาเดียวกับ `validate_install_app_name` ซึ่งเป็น private) และตรงกับ stem ของ `powerpcu-remote.exe` เมื่อไม่สนตัวพิมพ์
  - มี `conn-type=incoming`, `approve-mode=click`
  - เซิร์ฟเวอร์/key ตรงค่าที่กำหนด
  - **ไม่มี** `password` และ `allow-logon-screen-password`
  - รันใน CI ข้อ 5.6
- **Build:** workflow ใน PR ต้องผ่านและได้ artifact `powerpcu-remote-x86_64.exe`
- **ทดสอบมือบน Windows 10/11 ที่มี RustDesk ตัวมาตรฐานติดตั้งเป็น service อยู่แล้ว** (เช็กลิสต์ใน PR):
  1. รัน portable โดยไม่ใช้สิทธิ์ Admin ได้ หน้าแรกเป็นภาษาไทย มีโลโก้ PowerPCU ไม่มีช่องกรอก ID ปลายทาง
  2. RustDesk ของผู้ใช้ยังทำงาน ID เดิม config เดิม (`%APPDATA%\RustDesk\…` ไม่ถูกแก้) และ ID ของ PowerPCU Remote เป็นเลข 10 หลักที่ไม่ซ้ำกับ ID ของ RustDesk
     ทดสอบซ้ำในกรณีที่ RustDesk ของผู้ใช้ตั้งเซิร์ฟเวอร์เป็น `rustdesk.powerpcu.com` ไว้ด้วย: เชื่อมต่อไปที่ ID ของ PowerPCU ต้องได้หน้าต่างยอมรับของ PowerPCU เสมอ
  3. เชื่อมต่อจาก RustDesk ฝั่งทีมผ่าน `rustdesk.powerpcu.com` → ต้องขึ้นหน้าต่างให้กดยอมรับ, กดปฏิเสธแล้วเข้าไม่ได้
  4. ดูจอ, เมาส์/คีย์บอร์ด, Clipboard, ส่งไฟล์สองทาง ใช้ได้; Chat, voice call, terminal, tunnel, switch sides, จอเสมือน ไม่มีผล
  5. หน้าตั้งค่าไม่มีส่วนเซิร์ฟเวอร์; คัดลอก exe เป็นชื่อ `…host=evil.example…` แล้วยังเชื่อมต่อ `rustdesk.powerpcu.com`
  6. ติดตั้งเป็น service ได้ (service `PowerPCU-Remote` อยู่คู่กับ `RustDesk`), ฝั่งรีโมตจัดการหน้าต่าง UAC ได้หลังผู้ใช้กดยอมรับ, ขณะล็อกจอไม่มีทางเข้า
  7. ถอนการติดตั้ง PowerPCU Remote แล้ว RustDesk ของผู้ใช้ยังทำงาน, ไดรเวอร์ปริ้นเตอร์ของ RustDesk ยังอยู่ (`Get-PrinterDriver`)
  8. ค่า SHA-256 ในไฟล์ `.sha256` ตรงกับไฟล์ที่ดาวน์โหลด

## 7. ข้อจำกัดที่ยอมรับ (Known limits)

- **exe ไม่ได้เซ็น:** ถ้าเปิดจากไฟล์ที่ดาวน์โหลดผ่านเบราว์เซอร์ (มี Mark-of-the-Web) SmartScreen จะเตือน แต่การดาวน์โหลดโดย Box/Launcher ไม่ติดเครื่องหมายนี้
- **Portable packer:** สั่ง `taskkill RuntimeBroker_rustdesk.exe` ตอนแตกไฟล์ครั้งแรก และสคริปต์ติดตั้ง/ถอนก็มีบรรทัดเดียวกัน (ส่วนที่เกิดทุกครั้งที่ server เริ่มปิดแล้วด้วย R12) ผลคือ privacy mode ของ RustDesk ผู้ใช้หลุด *ถ้ากำลังใช้งานอยู่ตอนนั้นพอดี* การแก้ต้องเพิ่ม feature ให้ crate packer จึงยังไม่ทำ
- **ไฟล์ temp ที่ขึ้นต้น `rustdesk-*.exe` และเก่ากว่า 1 ชั่วโมงจะถูกลบ** (เป็นไฟล์ชั่วคราวของ RustDesk เอง ไม่มีผลต่อการใช้งาน)
- **ตอนล็อกจอ/ยังไม่มีใคร login:** ฝั่งรีโมตจะรอ 180 วินาทีก่อนหลุด แทนที่จะถูกปฏิเสธทันที
- **IPv6 probe ใช้ STUN สาธารณะที่ hard-code ไว้ (Cloudflare/Google)** แค่หา IP สาธารณะ ไม่มีข้อมูลเซสชันผ่าน
- **ยังเห็นข้อความ "PowerPCU-Remote" (มีขีด)** ในชื่อหน้าต่าง/tray/service เป็นผลของการเลือกแบบ ก.
- **หน้า About ยังมีลิงก์ rustdesk.com เดิม** (privacy/website) เพราะไม่ลบของเดิม

## 8. นอกขอบเขต — งานเฟส 2 ฝั่ง PowerPCU Box (บันทึกไว้ส่งต่อ)

- `AgentURL` → `https://github.com/WorkDii/powerpcu_desk/releases/download/<tag>/powerpcu-remote-x86_64.exe`, `AgentSHA256` → ค่าจากไฟล์ `.sha256`
- เลิกใช้ `ConfigPlan` ที่เขียนทับ `%APPDATA%\RustDesk\…\RustDesk2.toml` และคืนไฟล์ `.powerpcu-bak` ให้เครื่องที่เคยถูกเขียนทับ
- ตั้งชื่อไฟล์ที่ launcher บันทึกเป็น `powerpcu-remote.exe` ห้ามลงท้าย `install.exe` (จะเข้าโหมดติดตั้งทันที) และห้ามมี `-qs`
- บันทึกการรีโมต (ใคร/เครื่องไหน/เมื่อไร) ต้องมี API ฝั่งเรา ถ้าทำ ให้เปลี่ยน `register-device` ใน `src/powerpcu.rs` แล้วตั้ง `api-server`
- **ผลของโหมด service ต่อ flow ของ launcher (#48 §3.1/§3.3):**
  - เมื่อติดตั้งเป็น service แล้ว PowerPCU Remote จะรอรับการเชื่อมต่อตลอดเวลา ทีมเชื่อมต่อได้โดยไม่ต้องมีเซสชันจาก launcher และไม่มีแถวใน `remote_sessions`
  - เหลือแค่การกดยอมรับในหน้าต่าง CM ซึ่งเจ้าของเลือกแบบนี้ไว้ในสเปกข้อ 2
  - การกด "ติดตั้ง" ระหว่างเซสชันจาก launcher จะปิด process ที่ launcher เฝ้าอยู่ แถวบันทึกของเซสชันนั้นจึงปิดก่อนเวลา
  - เฟส 2 ต้องออกแบบการบันทึกโดยไม่พึ่งอายุของ process
- Launcher/Box อ่าน ID ของเครื่องได้ด้วย `powerpcu-remote.exe --get-id` (มีอยู่แล้วใน `core_main.rs:464`) ID คงที่ต่อเครื่องตาม 5.2

## 9. เรื่องที่ต้องให้เจ้าของตัดสิน / ทำเอง

1. **ปิด workflow เดิมของ RustDesk บน fork** (`flutter-ci`, `ci`, `flutter-nightly`, `flutter-tag`, `fdroid`, `update-webpki-roots`, `playground`, `clear-cache`)
   ด้วย `gh workflow disable` (เป็นการตั้งค่า repo ไม่แก้ไฟล์) ไม่อย่างนั้น PR นี้จะสั่ง build ทุกแพลตฟอร์มของ RustDesk
   และหลัง merge จะ build ซ้ำทุกครั้งที่ push master ส่วน nightly ปกติ GitHub ปิด schedule บน fork ไว้อยู่แล้ว — ต้องได้รับอนุญาตก่อนทำ

## 10. สิ่งที่ปล่อยให้ผู้ลงมือตัดสินใจเอง

- รูปแบบโค้ดภายใน `src/powerpcu.rs` และชื่อฟังก์ชัน, รูปแบบการทิ้งข้อความใน R7–R9 (ต้องไม่มี side effect และไม่มี log ระดับ debug+)
- ตำแหน่งที่แน่นอนของ widget ใน D1–D3 (ตรวจเลขบรรทัดใหม่ เพราะอ้างอิงจากการสำรวจ)
- รายละเอียดการวาดพิกัดโลโก้ให้คมที่ 16px และตัวอักษร wordmark (เจ้าของตรวจภาพก่อน commit)
- การแบ่ง job/step ใน workflow ตราบใดที่ได้ผลลัพธ์ตาม 5.6

## 11. พื้นที่เสี่ยงกระทบของเดิม (regression surface)

ไฟล์เดิมที่แก้ ทุกจุดอยู่ใน `#[cfg(feature = "powerpcu")]` หรือ `if (!isPowerPcuBuild())` ถ้าไม่เปิด feature พฤติกรรมเหมือนเดิมทุกอย่าง

| ไฟล์ | เหตุผลที่เลี่ยงไม่ได้ |
|---|---|
| `Cargo.toml` (`[features]`), `src/lib.rs` (`mod`) | ประกาศ feature และโมดูล |
| `libs/base/src/config/keys.rs` | ค่าคงที่ `OPTION_POWERPCU` (เป็นที่รวม option keys ตามกติกา repo) |
| `src/common.rs` | R1 จุดฉีดค่า |
| `src/platform/windows.rs`, `src/core_main.rs` | R2–R5 จุดชนกับ RustDesk ของผู้ใช้ |
| `src/updater.rs` | R6 กันการอัปเดตทับ |
| `src/server/connection.rs` | R7–R9 ตัด Chat/voice call/switch sides ฝั่ง server |
| `src/virtual_display_manager.rs` | R10–R11 ปิดจอเสมือน และกันการถอดจอเสมือนของ RustDesk ผู้ใช้ |
| `src/platform/windows.rs` (`try_kill_broker`) | R12 ไม่ปิด privacy-mode broker ของ RustDesk ผู้ใช้ |
| `flutter/lib/desktop/pages/desktop_home_page.dart`, `server_page.dart`, `desktop_setting_page.dart`, ไฟล์ค่าคงที่ option ฝั่ง Dart | D1–D3 ซ่อน UI |

ไฟล์ใหม่: `src/powerpcu.rs`, `flutter/lib/common/powerpcu.dart`, `res/powerpcu/**`, `.github/workflows/powerpcu-remote.yml`, เอกสารนี้
ไฟล์ที่ไม่แตะ: `libs/hbb_common` (submodule), `flutter-build.yml` และ workflow เดิมทั้งหมด, `build.py`, `LICENCE`, `src/lang/*`

## 12. ข้อที่ผู้ออกแบบตัดสินเอง (ขอให้เจ้าของตรวจ)

1. ID ของ PowerPCU Remote คำนวณจากรหัสเครื่อง + MAC ได้เลข 10 หลักคงที่ต่อเครื่อง (5.2)
2. ปิดเสียงและจอเสมือนด้วย เพราะไม่อยู่ในรายการ "เก็บไว้" (5.2, R10–R11)
3. ภาษาเริ่มต้นเป็นไทย ผู้ใช้เปลี่ยนเองได้
4. ไม่ส่ง heartbeat/sysinfo ไป API (`register-device=N`) เพราะ hbbs ฉบับ OSS ไม่มี API
5. ซ่อน "Powered by RustDesk" บนหน้าแรก แล้วย้ายการให้เครดิตกับลิงก์ซอร์สไปไว้หน้า About
6. หน้าแรกซ่อนช่องรหัสผ่านและข้อความที่พูดถึงรหัสผ่าน (D1)
7. Version resource ใช้ `CompanyName = WorkDii`
8. ตั้งชื่อ tag ว่า `remote-v<รุ่น RustDesk>-<ลำดับ>` เช่น `remote-v1.5.3-1`
9. ใส่แบรนด์ด้วยสคริปต์ตอน build แทนการแก้ไฟล์ไอคอนและ `Runner.rc` ใน repo โดยตรง
