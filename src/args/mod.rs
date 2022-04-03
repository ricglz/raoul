use clap::{Arg, Command, ArgMatches};

pub fn parse_args() -> ArgMatches {
    Command::new("raoul")
        .version("1.0")
        .author("ricglz")
        .about("My cool programming language")
        .arg(Arg::new("file")
            .value_name("FILE")
            .help("Sets a oran file to parse")
            .required(true))
        .get_matches()
}
