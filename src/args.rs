use clap::{App, AppSettings, Arg, ArgMatches};
use result::{Error, Result};
use std::borrow::Cow;
use std::env;
use std::ffi::OsString;
use std::path::Path;

const ZFILE: &'static str = "ZFILE";

pub struct Args<'a> {
  matches: ArgMatches<'a>,
}

impl<'a> Args<'a> {
  pub fn parse() -> Result<Args<'a>> {
    let matches = try!(parse_from(env::args_os()));
    Ok(Args { matches: matches })
  }

  pub fn zfile(&self) -> Cow<Path> {
    Cow::Borrowed(Path::new(self.matches.value_of(ZFILE).unwrap()))
  }
}

fn parse_from<'a, I, T>(itr: I) -> Result<ArgMatches<'a>>
  where I: IntoIterator<Item = T>,
        T: Into<OsString> {
  App::new("rzm")
  // App configuration
    .about("Collection of image tools in one command")
    .author(crate_authors!())
    .version(crate_version!())
    .setting(AppSettings::StrictUtf8)
    .setting(AppSettings::UnifiedHelpMessage)
    .setting(AppSettings::VersionlessSubcommands)
    .arg(Arg::with_name(ZFILE)
      .required(true)
      .index(1))

    // Process it.
    .get_matches_from_safe(itr)
    .map_err(Error::from)
}
