use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Configuration {
    #[arg(long, default_value = "./", env)]
    pub download_path: String,
    #[arg(long, action=ArgAction::SetTrue, env)]
    pub r#override: bool,
    #[arg(long, action=ArgAction::SetTrue, env)]
    pub skip_verification: bool,
}

impl std::fmt::Display for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Override: {}", &self.r#override)?;
        writeln!(f, "Skip Verification: {}", &self.skip_verification)?;
        writeln!(f, "Download Path: {}", &self.download_path)
    }
}