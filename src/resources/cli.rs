use bevy::prelude::*;

/// Command-line arguments parsed at startup.
/// Used for test automation and save-based feature verification.
#[derive(Resource, Debug, Default)]
pub struct CliArgs {
    /// Save file to load on startup (bypasses main menu).
    /// Usage: `cargo run -- --load <save_name>`
    pub load_save: Option<String>,

    /// Override the F5 quicksave name. Useful for creating test saves.
    /// Usage: `cargo run -- --save-as test_feature`
    /// Then press F5 in-game to save to "test_feature" instead of "quicksave".
    pub save_as: Option<String>,
}

impl CliArgs {
    /// Parse command-line arguments.
    /// Supports:
    /// - `--load <save_name>`: Load specified save on startup
    /// - `--save-as <save_name>`: Override F5 quicksave name
    pub fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut cli = CliArgs::default();

        let mut i = 1; // Skip program name
        while i < args.len() {
            match args[i].as_str() {
                "--load" => {
                    if i + 1 < args.len() {
                        cli.load_save = Some(args[i + 1].clone());
                        info!("CLI: Will load save '{}' on startup", args[i + 1]);
                        i += 2;
                    } else {
                        warn!("CLI: --load requires a save name argument");
                        i += 1;
                    }
                }
                "--save-as" => {
                    if i + 1 < args.len() {
                        cli.save_as = Some(args[i + 1].clone());
                        info!("CLI: F5 will save to '{}' instead of 'quicksave'", args[i + 1]);
                        i += 2;
                    } else {
                        warn!("CLI: --save-as requires a save name argument");
                        i += 1;
                    }
                }
                arg => {
                    if arg.starts_with('-') {
                        warn!("CLI: Unknown argument '{}'", arg);
                    }
                    i += 1;
                }
            }
        }

        cli
    }
}
