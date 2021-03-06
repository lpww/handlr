use error::{Error, Result};

mod apps;
mod cli;
mod common;
mod config;
mod error;

fn main() -> Result<()> {
    use clap::Clap;
    use cli::Cmd;
    use common::MimeType;
    use std::convert::TryFrom;

    let mut apps = apps::MimeApps::read()?;
    crate::config::Config::load()?;

    let res = || -> Result<()> {
        match Cmd::parse() {
            Cmd::Set { mime, handler } => {
                apps.set_handler(mime.0, handler)?;
            }
            Cmd::Add { mime, handler } => {
                apps.add_handler(mime.0, handler)?;
            }
            Cmd::Launch { mime, args } => {
                apps.get_handler(&mime.0)?.launch(args)?;
            }
            Cmd::Get { mime, json } => {
                apps.show_handler(&mime.0, json)?;
            }
            Cmd::Open { paths } => {
                let mime = MimeType::try_from(paths[0].as_str())?.0;
                apps.get_handler(&mime)?.open(paths)?;
            }
            Cmd::List => {
                apps.print()?;
            }
            Cmd::Unset { mime } => {
                apps.remove_handler(&mime.0)?;
            }
            Cmd::Autocomplete {
                desktop_files,
                mimes,
            } => {
                if desktop_files {
                    apps.list_handlers()?;
                } else if mimes {
                    common::db_autocomplete()?;
                }
            }
        }
        Ok(())
    }();

    match (res, atty::is(atty::Stream::Stdout)) {
        (Err(e), _) if matches!(e, Error::Cancelled) => {
            std::process::exit(1);
        }
        (Err(e), true) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
        (Err(e), false) => {
            std::process::Command::new("notify-send")
                .args(&["handlr error", &e.to_string()])
                .spawn()?;
            std::process::exit(1);
        }
        _ => Ok(()),
    }
}
