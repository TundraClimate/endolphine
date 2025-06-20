mod arguments;

#[tokio::main]
async fn main() {
    use arguments::{Expected, TerminationCause};

    match arguments::parse_args() {
        Expected::OpenEndolphine(path) => {}
        Expected::OpenConfigEditor => {}
        Expected::Termination(cause) => {}
    }
}
