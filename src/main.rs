mod arguments;
mod tui;

#[tokio::main]
async fn main() {
    use arguments::{Expected, TerminationCause};
    use std::{panic, process};

    if cfg!(windows) {
        panic!("Endolphine is not supported in Windows")
    }

    panic::set_hook(Box::new(|e| {
        tui::disable();

        if let Some(e) = e.payload().downcast_ref::<String>() {
            tui::terminate(e);
        } else if let Some(e) = e.payload().downcast_ref::<&str>() {
            tui::terminate(e);
        }

        process::exit(1);
    }));

    match arguments::parse_args() {
        Expected::OpenEndolphine(path) => {
            tui::enable();
        }
        Expected::OpenConfigEditor => {}
        Expected::Termination(cause) => match cause {
            TerminationCause::InvalidPath(path) => {
                panic!("Invalid path detected: {}", path.to_string_lossy())
            }
        },
    }
}
