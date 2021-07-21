use std::path::PathBuf;

use clap::arg_enum;
use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// The QPU to run on
    #[structopt(short, long, possible_values = &QPU::variants(), case_insensitive = true)]
    qpu: QPU,

    /// Number of times to run the program
    #[structopt(short, long, default_value = "1")]
    shots: u16,

    /// File containing the Quil program
    #[structopt(name = "FILE", parse(from_os_str))]
    file: PathBuf,
}

arg_enum! {
    #[derive(Debug)]
    enum QPU {
        Aspen9,
        QVM,
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let opt = Opt::from_args();
    let quil = std::fs::read_to_string(opt.file).unwrap();
    match opt.qpu {
        QPU::Aspen9 => {
            let result = qcs::qpu::run_program(&quil, opt.shots, "ro", "Aspen-9")
                .await
                .unwrap();
            println!("{:#?}", result);
        }
        QPU::QVM => {
            let result = qcs::qvm::run_program(&quil, opt.shots, "ro").await.unwrap();
            println!("{:#?}", result);
        }
    }
}
