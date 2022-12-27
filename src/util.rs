// for stdout().flush
use std::io::Write;

/// Display a prompt asking for confirmation by the user
///
/// Returns true if the user confirmed, false in all other cases
pub fn confirm_action() -> bool {
    let mut input = String::new();

    loop {
        print!("Is this ok? [yN] ");
        std::io::stdout().flush().unwrap();

        if std::io::stdin().read_line(&mut input).is_err() {
            return false;
        }

        match input.trim() {
            "y" | "Y" => return true,
            "n" | "N" => return false,
            _ => (),
        }
        input.clear();
    }
}
