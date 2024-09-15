use args::Arguments;
use clap::Parser;

pub mod args;

fn main() {
    let args = Arguments::parse();

    let subscriber_builder = tracing_subscriber::fmt()
        .with_max_level(args.level)
        .with_thread_ids(true)
        .without_time()
        .compact();

    tracing::subscriber::set_global_default(subscriber_builder.finish()).unwrap();

    tracing::debug!(?args);

    let source = args.source.0;
    let destination = args.destination.0;

    assert!(source.exists(), "The source Directory does not exists!");

    copykitty::copy(
        &source,
        &destination,
        args.follow_symlinks,
        args.comparison.into(),
        !args.disable_copy_metadata,
        args.threads,
    )
    .unwrap();
}
