use crate::utils::get_display;
use crate::utils::DisplayServer;
use duct::cmd;

pub fn main(output: Option<String>, clipboard: bool) {
    let filename = match output {
        Some(output) => output,
        None => {
            let s_dir = "/tmp/screenshots";
            let iso_date_time = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
            // Make s_dir if not exists
            std::fs::create_dir_all(s_dir).expect("Failed to create screenshot directory");
            format!("{s_dir}/{}.png", iso_date_time)
        }
    };
    match get_display() {
        DisplayServer::Wayland => {
            println!("Running on Wayland");
            wayland_screenshot(filename, clipboard);
        }
        DisplayServer::X11 => {
            println!("Running on X11");
            x11_screenshot(filename, clipboard);
        }
        DisplayServer::Other => {
            todo!("Screenshot only implemented for wayland and X11");
        }
    }
}

/// See
/// /home/ryan/.config/hypr/take-screenshot.sh
fn wayland_screenshot(output: String, clipboard: bool) {
    // Get the dimensions with slurp
    let dim = cmd!("slurp")
        .read()
        .expect("Failed to get dimensions using slurp (Is slurp installed?)");
    // Use grim
    cmd!("grim", "-g", dim.trim(), &output)
        .run()
        .expect("Failed to take screenshot");

    // wl-copy -t image/png < $f_name
    if clipboard {
        match cmd!("wl-copy", "-t", "image/png").stdin_path(output).run() {
            Ok(_) => println!("Copied to clipboard"),
            Err(e) => eprintln!("Failed to copy to clipboard: {}", e),
        }
    }
}

fn x11_screenshot(output: String, clipboard: bool) {
    // Use grim
    cmd!("maim", "--select", &output)
        .run()
        .expect("Failed to take screenshot");

    // wl-copy -t image/png < $f_name
    if clipboard {
        match cmd!("xclip", "--selection", "clipboard", "-t", "image/png")
            .stdin_path(output)
            .run()
        {
            Ok(_) => println!("Copied to clipboard"),
            Err(e) => eprintln!("Failed to copy to clipboard: {}", e),
        }
    }
}

pub fn get_clipboard() -> Option<String> {
    match get_display() {
        DisplayServer::Wayland => return cmd!("wl-paste").read().ok(),
        DisplayServer::X11 => return cmd!("xclip", "-o").read().ok(),
        DisplayServer::Other => {
            todo!("Screenshot only implemented for wayland and X11");
        }
    }
}

pub fn set_clipboard(input: String) -> std::result::Result<(), std::io::Error> {
    match get_display() {
        DisplayServer::Wayland => {
            cmd!("wl-copy").stdin_bytes(input).stdout_capture().run()?;
            Ok(())
        }
        DisplayServer::X11 => {
            cmd!("xclip").stdin_bytes(input).stdout_capture().run()?;
            Ok(())
        }
        DisplayServer::Other => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Clibpoard only implemented for wayland and X11",
            ));
        }
    }
}
