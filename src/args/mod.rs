use clap::{Arg, Command, ArgMatches};

pub fn parse_args() -> ArgMatches {
    Command::new("raoul")
        .version("1.0")
        .author("ricglz")
        .about("My cool programming language")
        .arg(Arg::new("file")
            .value_name("FILE")
            .help("Sets a file to parse")
            .required(true))
        .arg(Arg::new("verbose")
            .short('v')
            .long("verbose")
            .value_name("VERBOSE")
            .help("Makes it verbose")
            .default_value("false")
            .takes_value(false)
            .required(false))
        .get_matches()
}
