mod arch;
mod error;
mod io;
mod pack;
mod unpack;

use bpaf::{construct, positional, short, OptionParser, Parser};
use std::{fs::File, path::Path};

use crate::arch::Laby2;
use crate::error::*;
use crate::io::{XorReadable, XorWritable};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum AppBehaviour {
    Unpack,
    Pack,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct RunConfig {
    input_path: String,
    output_path: String,
    app_behaviour: AppBehaviour,
}

impl RunConfig {
    fn parser() -> OptionParser<Self> {
        let unpack = short('u')
            .long("unpack")
            .help("Unpack the game files to a directory")
            .req_flag(AppBehaviour::Unpack);
        let pack = short('p')
            .long("pack")
            .help("Pack existing game files to an archive")
            .req_flag(AppBehaviour::Pack);

        let app_behaviour = construct!([unpack, pack]);

        let input_path = positional("input").help("Input path.\n When packing, point this to the directory with unpacked files.\n When unpacking, point this to the game archive file.");
        let output_path = positional("output").help("Output path.\n When packing, enter the output file path.\n When unpacking, enter the output directory path.");

        construct!(RunConfig {
            app_behaviour,
            input_path,
            output_path
        })
        .to_options()
    }
}

fn main() -> Result<(), RepackerError> {
    let run_config = RunConfig::parser().run();

    match run_config.app_behaviour {
        AppBehaviour::Unpack => {
            let file = File::open(run_config.input_path)?;
            Laby2::unpack(file.xor_read(), Path::new(&run_config.output_path))
        }
        AppBehaviour::Pack => {
            let file = File::create(run_config.output_path)?;
            Laby2::pack(file.xor_write(), Path::new(&run_config.input_path))
        }
    }
}
