use std::io::IsTerminal;

use tracing_subscriber::EnvFilter;

use crate::cli::ColorChoice;

/// Determine whether to use ANSI colors in output.
pub(crate) fn should_use_color(color: &ColorChoice) -> bool {
    match color {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => {
            std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none()
        }
    }
}

/// Initialize the tracing subscriber with stderr output, color awareness, and level control.
///
/// - `verbose`: count of -v flags (0=warn, 1=info, 2=debug, 3+=trace)
/// - `trace`: if true, override to trace level regardless of verbose count
/// - `color`: color choice from CLI
pub fn init_logging(verbose: u8, trace: bool, color: &ColorChoice) {
    let level = if trace {
        "trace"
    } else {
        match verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        }
    };

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(should_use_color(color))
        .with_target(false)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level)),
        )
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_always_returns_true() {
        assert!(should_use_color(&ColorChoice::Always));
    }

    #[test]
    fn color_never_returns_false() {
        assert!(!should_use_color(&ColorChoice::Never));
    }

    #[test]
    fn color_auto_respects_no_color_env() {
        let key = "NO_COLOR";
        let original = std::env::var(key).ok();

        // Set NO_COLOR — should return false regardless of TTY
        std::env::set_var(key, "1");
        assert!(!should_use_color(&ColorChoice::Auto));

        // Restore
        match original {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
}
