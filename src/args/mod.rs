use clap::{Arg, ArgMatches, Command};

pub fn parse_arguments() -> ArgMatches {
    Command::new("raoul")
        .version("1.0")
        .author("ricglz")
        .about("My cool programming language")
        .arg(
            Arg::new("file")
                .value_name("FILE")
                .help("Sets a file to parse")
                .required(true),
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .value_name("DEBUG")
                .help("Displays debugging prints throughout the process")
                .default_value("false")
                .takes_value(false)
                .required(false),
        )
        .get_matches()
}
