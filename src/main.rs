extern crate log;

use clap::{command, Parser, ArgGroup};
use log::{LevelFilter};
use log::{trace, debug, info, warn, error};
use env_logger::Builder;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
//#[group(args = ["lat", "lon", "alt"], required = false)]
#[clap(group(
    clap::ArgGroup::new("Receiver location")
        .required(false)
        .multiple(true)
        .args(&["lat", "lon", "alt"]),
))]
#[clap(group(
    clap::ArgGroup::new("Multilateration server connection")
        .required(false)
        .multiple(true)
        .args(&["user", "server", "no_udp"]),
))]
struct Cli {
  // Sets the latitude
  #[arg(short = None, long = "lat", action, env = "MLAT_LAT", help = "Latitude of the receiver, in decimal degrees. Required.")]
  lat: u32,

  // Sets the longitude
  #[arg(short = None, long = "lon", action, env = "MLAT_LON", help = "Longitude of the receiver, in decimal degrees. Required.")]
  lon: u32,

  // Sets the altitude
  #[arg(short = None, long = "alt", action, env = "MLAT_ALT", help = "Altitude of the receiver (height above ellipsoid). Required. Defaults to metres, but units may be specified with a 'ft' or 'm' suffix. (Except if they're negative).")]
  alt: u32,

  // Sets the privacy flag
  #[arg(short = None, long = "privacy", action = clap::ArgAction::SetTrue, default_value_t = false, env = "MLAT_PRIVACY", help = "Sets the privacy flag for this receiver. Currently, this removes the receiver location pin from the coverage maps.")]
  privacy: bool,

  // Sets the user
  #[arg(short = None, long = "user", action, env = "MLAT_USER", help = "User information to give to the server. Used to get in touch if there are problems.")]
  user: String,

  // Sets the server 
  #[arg(short = None, long = "server", action, env = "MLAT_SERVER", help = "host:port of the multilateration of the server to connect to")]
  server: String,

  // Sets the no UDP flag
  #[arg(short = None, long = "no-udp", action = clap::ArgAction::SetFalse, default_value_t = true, env = "MLAT_NO_UDP", help = "Don't offer to use UDP transport for sync/mlat messages")]
  no_udp: bool,


  // Manage debugging information
  #[arg(short = 'v', long = "log-level", alias = "loglevel", action = clap::ArgAction::Set, default_value_t = String::from("info"), value_parser = ["off", "error", "warn", "info", "debug", "trace"], env = "BLADERF_ADSB_LOG_LEVEL", help = "Log level")]
  log_level: String,
  #[arg(short = None, long = "log-style", alias = "logstyle", action = clap::ArgAction::Set, default_value_t = String::from("auto"), value_parser = ["auto", "always", "never"], env = "BLADERF_ADSB_LOG_STYLE", help = "Manage color for log messages")]
  log_style: String,
}

// References:
// https://docs.rs/clap/latest/clap/enum.ArgAction.html
fn main() {
    let cli = Cli::parse();

    // setup logging
    let mut builder = Builder::new();
    builder.filter_level(LevelFilter::from_str(cli.log_level.as_str()).unwrap());
    builder.parse_write_style(cli.log_style.as_str());
    builder.init();

    //let remote: bool = cli.remote.clone();
    
    println!("{:?}", cli);

    error!("error");
    warn!("warn");
    info!("info");
    debug!("debug");
    trace!("trace");

    //ctrlc::set_handler(move || {
    //  debug!("received Ctrl+C!");
    //})
    //.expect("Error setting Ctrl-C handler");
}
