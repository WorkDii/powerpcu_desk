use std::collections::HashMap;

use base::config::keys;
#[cfg(feature = "powerpcu")]
use hbb_common::config;
use hbb_common::sha2::{Digest, Sha256};

pub(crate) const APP_NAME: &str = "PowerPCU-Remote";
pub(crate) const RENDEZVOUS_SERVER: &str = "rustdesk.powerpcu.com";
pub(crate) const KEY: &str = "Yl+eLj+8+MvlyMQcg117JyVvyy1ojHSGaY2+N7aYmO8=";

// Install and uninstall scripts run these against our own exe; they would
// remove drivers and certificates that a user's RustDesk shares with us.
const SHARED_DRIVER_COMMANDS: &[&str] = &[
    "--install-remote-printer",
    "--uninstall-remote-printer",
    "--uninstall-cert",
    "--uninstall-amyuni-idd",
    "--install-idd",
];

pub(crate) struct Settings {
    pub(crate) hard: HashMap<String, String>,
    pub(crate) overwrite: HashMap<String, String>,
    pub(crate) overwrite_local: HashMap<String, String>,
    pub(crate) default_local: HashMap<String, String>,
    pub(crate) builtin: HashMap<String, String>,
}

fn to_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

pub(crate) fn settings() -> Settings {
    Settings {
        hard: to_map(&[
            ("conn-type", "incoming"),
            ("disable-account", "Y"),
            ("disable-ab", "Y"),
            ("disable-settings", "Y"),
        ]),
        overwrite: to_map(&[
            (keys::OPTION_CUSTOM_RENDEZVOUS_SERVER, RENDEZVOUS_SERVER),
            (keys::OPTION_KEY, KEY),
            (keys::OPTION_APPROVE_MODE, "click"),
            (keys::OPTION_VERIFICATION_METHOD, "use-temporary-password"),
            ("allow-hide-cm", "N"),
            (keys::OPTION_ACCESS_MODE, "custom"),
            (keys::OPTION_ENABLE_KEYBOARD, "Y"),
            (keys::OPTION_ENABLE_CLIPBOARD, "Y"),
            (keys::OPTION_ENABLE_FILE_TRANSFER, "Y"),
            (keys::OPTION_ENABLE_AUDIO, "N"),
            (keys::OPTION_ENABLE_CAMERA, "N"),
            (keys::OPTION_ENABLE_TERMINAL, "N"),
            (keys::OPTION_ENABLE_TUNNEL, "N"),
            (keys::OPTION_ENABLE_REMOTE_PRINTER, "N"),
            (keys::OPTION_ENABLE_REMOTE_RESTART, "N"),
            (keys::OPTION_ENABLE_RECORD_SESSION, "N"),
            (keys::OPTION_ENABLE_BLOCK_INPUT, "N"),
            (keys::OPTION_ENABLE_PRIVACY_MODE, "N"),
            (keys::OPTION_ALLOW_REMOTE_CONFIG_MODIFICATION, "N"),
            (keys::OPTION_ENABLE_LAN_DISCOVERY, "N"),
            (keys::OPTION_DIRECT_SERVER, "N"),
            (keys::OPTION_ENABLE_TRUSTED_DEVICES, "N"),
            (keys::OPTION_ALLOW_AUTO_UPDATE, "N"),
            (keys::OPTION_ALLOW_WEBSOCKET, "N"),
            (keys::OPTION_ALLOW_INSECURE_TLS_FALLBACK, "N"),
        ]),
        overwrite_local: to_map(&[
            (keys::OPTION_ENABLE_CHECK_UPDATE, "N"),
            (keys::OPTION_ALLOW_REMOTE_CM_MODIFICATION, "N"),
        ]),
        default_local: to_map(&[(keys::OPTION_LANGUAGE, "th")]),
        builtin: to_map(&[
            (keys::OPTION_HIDE_SECURITY_SETTINGS, "Y"),
            (keys::OPTION_HIDE_NETWORK_SETTINGS, "Y"),
            (keys::OPTION_HIDE_SERVER_SETTINGS, "Y"),
            (keys::OPTION_HIDE_PROXY_SETTINGS, "Y"),
            (keys::OPTION_HIDE_WEBSOCKET_SETTINGS, "Y"),
            (keys::OPTION_HIDE_REMOTE_PRINTER_SETTINGS, "Y"),
            (keys::OPTION_HIDE_POWERED_BY_ME, "Y"),
            (keys::OPTION_DISABLE_CHANGE_PERMANENT_PASSWORD, "Y"),
            (keys::OPTION_DISABLE_CHANGE_ID, "Y"),
            (keys::OPTION_DISABLE_UNLOCK_PIN, "Y"),
            (keys::OPTION_REGISTER_DEVICE, "N"),
            (keys::OPTION_POWERPCU, "Y"),
        ]),
    }
}

// Stock RustDesk derives its ID from the MAC address, so on a shared relay it
// would collide with the user's own RustDesk. The 10-digit range cannot meet a
// MAC-derived ID (at most 0x1FFFFFFF) and is the same on every run.
pub(crate) fn derive_id(machine_uuid: &[u8]) -> String {
    let digest = Sha256::new()
        .chain_update(machine_uuid)
        .chain_update(APP_NAME.as_bytes())
        .finalize();
    let mut head = [0u8; 8];
    head.copy_from_slice(&digest[..8]);
    (1_000_000_000 + u64::from_be_bytes(head) % 1_000_000_000).to_string()
}

// Config::load() fills a missing ID from the MAC address before apply() can look,
// so "empty" never reaches us; anything outside the 10-digit range is replaced.
// A server-assigned ID after UUID_MISMATCH is in that range and is kept.
pub(crate) fn needs_own_id(id: &str) -> bool {
    !matches!(id.parse::<u64>(), Ok(n) if (1_000_000_000..2_000_000_000).contains(&n))
}

pub(crate) fn is_shared_driver_command(arg: &str) -> bool {
    SHARED_DRIVER_COMMANDS.contains(&arg)
}

// APP_NAME must be set before anything touches `Config`: it names the config file.
#[cfg(feature = "powerpcu")]
pub(crate) fn apply() {
    *config::APP_NAME.write().unwrap() = APP_NAME.to_owned();
    let s = settings();
    config::HARD_SETTINGS.write().unwrap().extend(s.hard);
    config::OVERWRITE_SETTINGS.write().unwrap().extend(s.overwrite);
    config::OVERWRITE_LOCAL_SETTINGS
        .write()
        .unwrap()
        .extend(s.overwrite_local);
    config::DEFAULT_LOCAL_SETTINGS
        .write()
        .unwrap()
        .extend(s.default_local);
    config::BUILTIN_SETTINGS.write().unwrap().extend(s.builtin);
    if needs_own_id(&config::Config::get_id()) {
        config::Config::set_id(&derive_id(&hbb_common::get_uuid()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_name_is_installable_and_matches_exe() {
        assert!(APP_NAME
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'));
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
        for k in [
            "enable-audio",
            "enable-camera",
            "enable-terminal",
            "enable-tunnel",
            "enable-remote-printer",
            "enable-remote-restart",
            "enable-record-session",
            "enable-block-input",
            "enable-privacy-mode",
            "enable-lan-discovery",
            "direct-server",
            "allow-auto-update",
            "allow-remote-config-modification",
        ] {
            assert_eq!(s.overwrite[k], "N", "{k}");
        }
        assert_eq!(s.overwrite_local["enable-check-update"], "N");
        assert_eq!(s.builtin["register-device"], "N");
        assert_eq!(s.builtin["powerpcu"], "Y");
    }

    #[test]
    fn thai_is_the_default_language() {
        assert_eq!(settings().default_local["lang"], "th");
    }

    #[test]
    fn derived_id_is_stable_and_outside_mac_range() {
        assert_eq!(derive_id(b"test-machine"), "1031630599");
        assert_eq!(derive_id(b"other-machine"), "1427890610");
        let id: u64 = derive_id(b"x").parse().unwrap();
        assert!((1_000_000_000..2_000_000_000).contains(&id));
    }

    #[test]
    fn mac_derived_ids_are_replaced_but_ten_digit_ids_kept() {
        assert!(needs_own_id("288599328"));
        assert!(needs_own_id(""));
        assert!(needs_own_id("DESKTOP-PC01"));
        assert!(!needs_own_id("1427890610"));
        assert!(!needs_own_id(&derive_id(b"any-machine")));
    }

    #[test]
    fn shared_driver_commands_are_recognised() {
        for c in [
            "--install-remote-printer",
            "--uninstall-remote-printer",
            "--uninstall-cert",
            "--uninstall-amyuni-idd",
            "--install-idd",
        ] {
            assert!(is_shared_driver_command(c), "{c}");
        }
        for c in ["--install", "--uninstall", "--service", "--server", "--cm"] {
            assert!(!is_shared_driver_command(c), "{c}");
        }
    }
}
