use clap::{App, AppSettings, ArgMatches};
use result::{Error, Result};
use std::env;
use std::ffi::OsString;

pub fn parse<'a>() -> Result<ArgMatches<'a>> {
  parse_from(env::args_os())
}

fn parse_from<'a, I, T>(itr: I) -> Result<ArgMatches<'a>>
  where I: IntoIterator<Item = T>,
        T: Into<OsString> {
  App::new("imt")
  // App configuration
    .about("Collection of image tools in one command")
    .author(crate_authors!())
    .version(crate_version!())
    .setting(AppSettings::StrictUtf8)
    .setting(AppSettings::UnifiedHelpMessage)
    .setting(AppSettings::VersionlessSubcommands)

    // Process it.
    .get_matches_from_safe(itr)
    .map_err(Error::from)
}
