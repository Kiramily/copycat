use clap::{value_parser, Arg, Command};
use std::path::PathBuf;

fn main() {
    let args = Command::new("copycat")
        .arg(
            Arg::new("source")
                .help("The Source Directory to copy")
                .index(1)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("destination")
                .help("The Destination Directory to copy")
                .index(2)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("threads")
                .short('t')
                .long("threads")
                .help("The number of threads to use")
                .value_parser(value_parser!(usize)),
        )
        .get_matches();

    let source = args.get_one::<PathBuf>("source").unwrap();
    let destination = args.get_one::<PathBuf>("destination").unwrap();
    let threads: Option<usize> = args.get_one::<usize>("threads").map(ToOwned::to_owned);

    assert!(source.exists(), "The source Directory does not exists!");
    copykitty::copy(&source, &destination, threads).unwrap();
}
