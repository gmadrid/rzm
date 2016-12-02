use clap::{App, AppSettings, Arg, ArgMatches};
use rzm::{Error, Result};
use std::borrow::Cow;
use std::env;
use std::ffi::OsString;
use std::path::Path;

const ZFILE: &'static str = "ZFILE";
const STACK_SIZE: &'static str = "stacksize";
const DEFAULT_STACK_SIZE: &'static str = "61440";

pub struct Args<'a> {
  matches: ArgMatches<'a>,
}

impl<'a> Args<'a> {
  pub fn parse() -> Result<Args<'a>> {
    let matches = parse_from(env::args_os())?;
    Ok(Args { matches: matches })
  }

  pub fn zfile(&self) -> Cow<Path> {
    Cow::Borrowed(Path::new(self.matches.value_of(ZFILE).unwrap()))
  }

  // pub fn stacksize(&self) -> Result<u16> {
  //   self.matches
  //     .value_of(STACK_SIZE)
  //     .unwrap_or(DEFAULT_STACK_SIZE)
  //     .parse::<u16>()
  //     .map_err(|e| Error::ParseIntError(STACK_SIZE, e))
  // }
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

    // Arguments.
    .arg(Arg::with_name(ZFILE)
      .required(true)
      .multiple(false)
      .index(1))
    .arg(Arg::with_name(STACK_SIZE)
      .long(STACK_SIZE)
      .visible_alias("ss")
      .takes_value(true)
      .multiple(false)
      .number_of_values(1)
      .help("Size of the ZMachine stack (in bytes)")
      .default_value(DEFAULT_STACK_SIZE))

    // Process it.
    .get_matches_from_safe(itr)
    .map_err(Error::from)
}
