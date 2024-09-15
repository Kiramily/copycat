use clap::builder::{TypedValueParser, ValueParserFactory};
use copykitty::Comparison;
use std::path::PathBuf;
use tracing::level_filters::LevelFilter;

#[derive(Clone, Debug)]
pub struct Directory(pub PathBuf);

#[derive(Clone, Debug)]
pub struct DirectoryParser;

impl ValueParserFactory for Directory {
    type Parser = DirectoryParser;

    fn value_parser() -> Self::Parser {
        DirectoryParser
    }
}

impl TypedValueParser for DirectoryParser {
    type Value = Directory;

    fn parse_ref(
        &self,
        _: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let path = PathBuf::from(value);

        if !path.exists() || path.is_dir() {
            Ok(Directory(path))
        } else {
            let mut error = clap::Error::new(clap::error::ErrorKind::ValueValidation);

            if let Some(arg) = arg {
                error.insert(
                    clap::error::ContextKind::InvalidArg,
                    clap::error::ContextValue::String(arg.to_string()),
                );
            }

            error.insert(
                clap::error::ContextKind::InvalidValue,
                clap::error::ContextValue::String(format!(
                    "the path {} is not a valid directory",
                    path.display()
                )),
            );

            Err(error)
        }
    }
}

#[derive(clap::Parser, Debug)]
pub struct Arguments {
    #[clap(index = 1)]
    pub source: Directory,

    #[clap(index = 2)]
    pub destination: Directory,

    #[clap(
        long,
        short = 't',
        help = "The number of threads to use. Defaults to the maximum supported by the CPU"
    )]
    pub threads: Option<usize>,

    #[clap(
        long,
        short = 'f',
        help = "Follows the symlinks and recreates the directory structure. (not tested and potentially buggy)"
    )]
    pub follow_symlinks: bool,

    #[clap(
		value_enum,
		default_value_t=ComparisonStrategy::Date,
		short = 's',
		long = "comparison-strategy",
		help = "which factor decides if a file should be overwritten"
	)]
    pub comparison: ComparisonStrategy,

    #[clap(
        long,
        help = "disable copying metadata. Useful for platforms that don't support copying metadata"
    )]
    pub disable_copy_metadata: bool,

    #[clap(
		value_enum,
		default_value_t=LoggingLevel::Info,
		short = 'l',
		long = "level",
		help = "The logging level")]
    pub level: LoggingLevel,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
pub enum LoggingLevel {
    Info,
    Debug,
    Off,
}

impl From<LoggingLevel> for LevelFilter {
    fn from(val: LoggingLevel) -> Self {
        match val {
            LoggingLevel::Info => LevelFilter::INFO,
            LoggingLevel::Debug => LevelFilter::DEBUG,
            LoggingLevel::Off => LevelFilter::OFF,
        }
    }
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq)]
pub enum ComparisonStrategy {
    #[clap(help = "copies every file whether they exist or not")]
    None,
    #[clap(help = "skips if the destination file exists")]
    Exists,
    #[clap(help = "compares the file modification dates")]
    Date,
}

impl From<ComparisonStrategy> for Comparison {
    fn from(value: ComparisonStrategy) -> Self {
        match value {
            ComparisonStrategy::None => Comparison::None,
            ComparisonStrategy::Exists => Comparison::Exists,
            ComparisonStrategy::Date => Comparison::Date,
        }
    }
}
