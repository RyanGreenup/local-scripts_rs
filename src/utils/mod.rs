


pub enum DisplayServer {
    Wayland,
    X11,
    Other,
}



fn is_wayland() -> bool {
    // check env
    let env_set = std::env::var("WAYLAND_DISPLAY").is_ok();

    // check if XDG_SESSION_TYPE is wayland
    let xdg_session = std::env::var("XDG_SESSION_TYPE")
        .map(|s| s == "wayland")
        .unwrap_or(false);

    // are both set
    env_set && xdg_session
}

fn is_x11() -> bool {
    // check env
    let env_set = std::env::var("DISPLAY").is_ok();

    // check if XDG_SESSION_TYPE is x11
    let xdg_session = std::env::var("XDG_SESSION_TYPE")
        .map(|s| s == "x11")
        .unwrap_or(false);

    // are both set
    env_set && xdg_session
}

pub fn get_display() -> DisplayServer {
    if is_wayland() {
        DisplayServer::Wayland
    } else if is_x11() {
        DisplayServer::X11
    } else {
        DisplayServer::Other
    }
}
