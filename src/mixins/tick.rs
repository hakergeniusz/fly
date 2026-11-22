// Every tick, on the window's Game only: the fly toggle's press (the
// registered `toggle` mapping's click, polled here) flips the flight state
// and writes both flight fields at once — the engine's remote tick notices
// the `flying` flip and tells the server by itself, so this helper sends no
// packets of its own. While the toggle is on and the mode does not already
// fly, the local player's flight grant is re-asserted — the game's spawn
// sync (`spawn_player`) and the server's ability packets both reset
// `may_fly`/`flying` from the game mode, and a granted flight dies quietly
// without this. Toggled off, the fields were already cleared at the click.
// Copyright (c) 2026 hakergeniusz. MIT.
fn fly_tick(game: &mut crate::game::Game<'_>) {
    if !game.playing() {
        return;
    }
    let enabled = fly::setting_on(game.stored_setting("fly", "enabled").as_deref());
    let creative = game.creative;
    let spectator = game.mode == horizon_client::command::args::GameMode::Spectator;
    if fly::toggle_gates(enabled, creative, spectator) && game.consume_click("fly", "toggle") {
        let on = !fly::flight_on();
        fly::set_flight(on);
        if let Some(p) = game.player.as_mut() {
            let (may_fly, flying) = fly::apply(on);
            p.may_fly = may_fly;
            p.flying = flying;
        }
    }
    let on = fly::flight_on() && enabled;
    if let Some(p) = game.player.as_mut() {
        if let Some((may_fly, flying)) = fly::reassert(on, creative, spectator, p.may_fly, p.flying)
        {
            p.may_fly = may_fly;
            p.flying = flying;
        }
    }
}
