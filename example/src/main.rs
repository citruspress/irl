extern crate clap;
use clap::{App, Arg};
use main_error::MainError;

fn main() -> Result<(), MainError> {
    let matches = App::new("irlib-cli")
        .version("0.1.0")
        .author("Citruspress")
        .about("Emits IR commands using PWM")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets the configuration file")
                .takes_value(true)
                .default_value("config.toml"),
        )
        .arg(
            Arg::with_name("signal")
                .short("s")
                .long("signal")
                .value_name("NAME")
                .help("Specifies the signal to emit")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let config = matches.value_of("config").unwrap();
    let signal = matches.value_of("signal").unwrap();

    let remote = irl::Remote::from_config(config)?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    remote.emit(signal)?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    Ok(())
}
