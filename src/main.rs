use abigen::{AbiConfig, Api, Template};
use clap::Parser;
use std::{io::Write, path::PathBuf};

/// Generate a stable C ABI (application binary interface) for a C++ library.
///
/// Analyzes the provided C++ header file, and generates a C99 header and C++ implementation with functions that call
/// into the header's class methods. If template files are supplied, the generated content will be inserted between
/// "@abigen:start" and "@abigen:end" comment tags.
#[derive(Parser)]
struct Cli {
    /// A C++ header file to be mapped to C
    #[arg(long, short = 'i')]
    pub input: PathBuf,

    /// A file that will be updated with the generated header data and written to the output
    #[arg(long, alias = "th")]
    pub template_h: Option<PathBuf>,

    /// A file that will be updated with the generated function implementations and written to the output
    #[arg(long, alias = "tc")]
    pub template_c: Option<PathBuf>,

    /// One of "null", "stdout", "stderr", or the path to an output file
    #[arg(long, default_value = "null")]
    pub header: OutputType,

    /// One of "null", "stdout", "stderr", or the path to an output file
    #[arg(long, default_value = "null")]
    pub code: OutputType,

    /// The path to a config file controlling how the C functions are generated
    #[arg(long, alias = "cf")]
    pub config_file: Option<PathBuf>,

    #[arg(long, alias = "ex")]
    pub exception_type: Option<String>,

    #[arg(long, alias = "exsuccess")]
    pub success_value: Option<String>,

    #[arg(long, alias = "exnull")]
    pub nullref_value: Option<String>,

    #[arg(long, alias = "exgeneric")]
    pub generic_failure_value: Option<String>,

    /// An exception type name that should be caught, and the corresponding return value, separated by '='
    #[arg(long, alias = "excustom")]
    pub custom_exception: Vec<String>,
}

#[derive(Debug, Clone, Default)]
enum OutputType {
    #[default]
    Null,
    Stdout,
    Stderr,
    File(PathBuf),
}

impl From<&str> for OutputType {
    fn from(value: &str) -> Self {
        match value {
            "null" => Self::Null,
            "stdout" => Self::Stdout,
            "stderr" => Self::Stderr,
            p => Self::File(PathBuf::from(p)),
        }
    }
}

impl Write for OutputType {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Null => Ok(buf.len()),
            Self::Stdout => std::io::stdout().write(buf),
            Self::Stderr => std::io::stderr().write(buf),
            Self::File(p) => std::fs::write(p, buf).map(|_| buf.len()),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn main() {
    let mut args = Cli::parse();
    let mut config = AbiConfig::default();

    // read input files

    let api = Api::from_file(&args.input).expect("Failed to load input header");

    let template_h = if let Some(p) = args.template_h {
        Template::from_file(p).expect("Failed to load header template")
    } else if let OutputType::File(f) = &args.header
        && let Ok(true) = std::fs::exists(f)
    {
        Template::from_file(f)
            .expect("Failed to load existing header output file for use as a template")
    } else {
        Template::default()
    };

    let template_c = if let Some(p) = args.template_c {
        Template::from_file(p).expect("Failed to load code template")
    } else if let OutputType::File(f) = &args.code
        && let Ok(true) = std::fs::exists(f)
    {
        Template::from_file(f)
            .expect("Failed to load existing code output file for use as a template")
    } else {
        Template::default()
    };

    if let Some(p) = args.config_file {
        let buf = std::fs::read(p).expect("Failed to read config file");
        config = toml::from_slice(&buf).expect("Failed to parse config file");
    }

    // apply config options

    if let Some(t) = args.exception_type {
        config.exception.value_type = t;
    }
    if let Some(s) = args.success_value {
        config.exception.success_value = s;
    }
    if let Some(n) = args.nullref_value {
        config.exception.nullref_value = n;
    }
    if let Some(g) = args.generic_failure_value {
        config.exception.generic_failure_value = g;
    }
    if args.custom_exception.len() > 0 {
        config.exception.custom_values = args
            .custom_exception
            .into_iter()
            .filter_map(|s| {
                s.split_once('=')
                    .map(|(e, v)| (e.to_string(), v.to_string()))
            })
            .collect();
    }

    // write output files

    write!(args.header, "{}", template_h.header(&api, &config))
        .expect("Failed to write header file");

    write!(args.code, "{}", template_c.code(&api, &config)).expect("Failed to write code file");
}
