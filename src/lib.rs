//! Fly, the native-route native mod: survival flight on a key.
//!
//! The brain is pure standard library — no game types — so it compiles and
//! tests wherever. The policy is one toggle: while it is on, the local player
//! is re-granted flight every tick (the game's own spawn and mode sync resets
//! `may_fly` from the game mode, and the server's ability packets say the
//! same, so a granted flight dies quietly without the re-assert); while it is
//! off, the mod touches nothing. The toggle is a key mapping (the Controls
//! entry, default R): the tick mixin polls its click, flips the state and
//! writes both flight fields at once — the engine's own remote tick notices
//! the `flying` flip and tells the server on its own, so the mod sends no
//! packets of its own.
//!
//! Clean-room: written for Horizon from the game's own public behavior; it
//! copies nothing from any mod.
//!
//! Copyright (c) 2026 hakergeniusz. MIT.

use std::sync::atomic::{AtomicBool, Ordering};

use mod_api::{
    Hooks, KeyConflictContext, KeyRegistration, RegistrationContext, Registrar, SettingSpec,
};

/// The mod's registry id: the settings page and the mapping's owner.
pub const ID: &str = "fly";
/// The key mapping's id: polled by the tick helper as `consume_click(ID, TOGGLE)`.
pub const TOGGLE: &str = "toggle";

/// The translation keys the Controls entry and the settings row show.
pub const LABEL_TOGGLE: &str = "key.fly.toggle";
pub const LABEL_ENABLED: &str = "fly.setting.enabled";

/// Whether the mod is on: a missing setting means ON — the mod exists to
/// fly, so installing it is asking for it; only `false` turns it off,
/// trimmed like the game reads its settings.
pub fn setting_on(value: Option<&str>) -> bool {
    !matches!(value, Some(v) if v.trim() == "false")
}

/// The click's gates besides playing (the helper checks that): the mod on,
/// and not a mode that already flies (creative and spectator have the
/// game's own flight).
pub fn toggle_gates(enabled: bool, creative: bool, spectator: bool) -> bool {
    enabled && !creative && !spectator
}

/// The flight fields to write for the toggle's new value: granting flight
/// sets `may_fly` and lifts off; landing clears both, so no stale grant
/// survives the toggle.
pub fn apply(on: bool) -> (bool, bool) {
    if on { (true, true) } else { (false, false) }
}

/// Re-asserted every tick while the toggle is on: the game resets `may_fly`
/// from the game mode on every spawn (`spawn_player`) and the server's
/// ability packets say `may_fly = false` in survival, so a granted flight
/// must be re-granted or it silently dies. The flight stays on until the
/// player toggles it off — landing by touch does not end it, exactly like a
/// fly mod should. Creative and spectator already fly; nothing to enforce.
/// `None` leaves the fields alone.
pub fn reassert(
    on: bool,
    creative: bool,
    spectator: bool,
    may_fly: bool,
    flying: bool,
) -> Option<(bool, bool)> {
    if on && !creative && !spectator && (!may_fly || !flying) {
        Some((true, true))
    } else {
        None
    }
}

// The toggle's own state, shared with the tick helper (the mod crate is
// compiled before the game's copied functions, which call into it by name).
// Default: not flying.
static FLIGHT_ON: AtomicBool = AtomicBool::new(false);

/// Whether the player's flight is toggled on.
pub fn flight_on() -> bool {
    FLIGHT_ON.load(Ordering::Acquire)
}

/// Remember the toggle's new value.
pub fn set_flight(on: bool) {
    FLIGHT_ON.store(on, Ordering::Release);
}

#[derive(Default)]
pub struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// The Enabled toggle on the mod's page, and the fly key in Controls
    /// (gameplay conflict: it does nothing under a menu).
    fn register(&self, registrar: &mut dyn Registrar, _: &RegistrationContext) {
        let _ = registrar.register_setting(SettingSpec::toggle(
            "enabled",
            LABEL_ENABLED,
            true,
        ));
        registrar.register_key_mapping(
            KeyRegistration::new(TOGGLE, "key.keyboard.r")
                .label(LABEL_TOGGLE)
                .context(KeyConflictContext::InGame),
        );
    }

    /// Implemented as a no-op so the game always calls the
    /// `native_client_tick_pre` dispatch the tick mixin lives in: the game
    /// skips dispatches no mod listens to, and the mixin must run.
    fn client_tick_pre(&self, _: &mut dyn mod_api::ClientHost, _: &mut mod_api::ClientTickPreEvent) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Installed and never touched: the setting does not exist yet, and the
    /// mod is on — installing it is asking for the flight. Only an explicit
    /// `false` turns it off.
    #[test]
    fn installed_means_on_until_declined() {
        assert!(setting_on(None), "the default must be on");
        assert!(setting_on(Some(" true \n")));
        assert!(setting_on(Some("on")), "anything but false is on");
        assert!(!setting_on(Some("false")));
    }

    /// The click answers only its gates: the mod on, and a mode that does
    /// not already fly. Playing is the helper's own check.
    #[test]
    fn the_click_answers_only_its_gates() {
        assert!(toggle_gates(true, false, false));
        assert!(
            !toggle_gates(false, false, false),
            "the mod is off"
        );
        assert!(!toggle_gates(true, true, false), "creative already flies");
        assert!(
            !toggle_gates(true, false, true),
            "spectator already flies"
        );
    }

    /// Granting lifts off, landing clears both — no stale grant survives.
    #[test]
    fn apply_maps_the_flight_fields() {
        assert_eq!(apply(true), (true, true));
        assert_eq!(apply(false), (false, false));
    }

    /// While toggled on, a reset `may_fly`/`flying` is re-granted — whether
    /// the reset came from a spawn's mode sync or the server's ability
    /// packet. Creative and spectator already fly; the mod off; and an
    /// already-flying player all leave the fields alone.
    #[test]
    fn reassert_restores_only_a_lost_grant() {
        assert_eq!(reassert(true, false, false, false, false), Some((true, true)));
        assert_eq!(
            reassert(true, false, false, true, false),
            Some((true, true)),
            "may_fly survived, the lift did not"
        );
        assert_eq!(
            reassert(true, false, false, false, true),
            Some((true, true)),
            "the lift survived, the grant did not"
        );
        assert_eq!(reassert(true, false, false, true, true), None, "already flying");
        assert_eq!(reassert(true, true, false, false, false), None, "creative flies on its own");
        assert_eq!(reassert(true, false, true, false, false), None, "spectator flies on its own");
        assert_eq!(reassert(false, false, false, false, false), None, "the mod is off");
    }

    /// The toggle's state round-trips.
    #[test]
    fn the_flight_state_round_trips() {
        assert!(!flight_on(), "not flying out of the box");
        set_flight(true);
        assert!(flight_on());
        set_flight(false);
        assert!(!flight_on());
    }
}
