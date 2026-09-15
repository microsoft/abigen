use abigen::{AbiConfig, Api, Template};
use clap::Parser;
use std::{io::Write, path::PathBuf};

#[derive(Parser)]
struct Cli {
    /// A C++ header file to be mapped to C
    #[arg(long, short = 'i')]
    pub input: PathBuf,

    /// A file that will be updated with the generated header data and written to the output
    #[arg(long)]
    pub template_h: Option<PathBuf>,

    /// A file that will be updated with the generated function implementations and written to the output
    #[arg(long)]
    pub template_c: Option<PathBuf>,

    /// One of "null", "stdout", "stderr", or the path to an output file
    #[arg(long, default_value = "null")]
    pub header: OutputType,

    /// One of "null", "stdout", "stderr", or the path to an output file
    #[arg(long, default_value = "null")]
    pub code: OutputType,
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
            p => Self::File(PathBuf::from(p))
        }
    }
}

impl Write for OutputType {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Null => {
                Ok(buf.len())
            },
            Self::Stdout => {
                std::io::stdout().write(buf)
            },
            Self::Stderr => {
                std::io::stderr().write(buf)
            },
            Self::File(p) => {
                std::fs::write(p, buf).map(|_| buf.len())
            },
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn main() {
    let mut args = Cli::parse();

    let config = AbiConfig::default();

    let api = Api::from_file(&args.input)
        .expect("Failed to load input header");

    let template_h = if let Some(p) = args.template_h {
        Template::from_file(p).expect("Failed to load header template")
    } else {
        Template::default()
    };

    let template_c = if let Some(p) = args.template_c {
        Template::from_file(p).expect("Failed to load code template")
    } else {
        Template::default()
    };

    write!(args.header, "{}", template_h.header(&api, &config))
        .expect("Failed to write header file");

    write!(args.code, "{}", template_c.code(&api, &config))
        .expect("Failed to write code file");
}
