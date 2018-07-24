extern crate loggerv;
#[macro_use]
extern crate structopt;
extern crate icfp2018;

use structopt::StructOpt;

use icfp2018::nanobot;
use icfp2018::nanobot::Result;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short = "v", parse(from_occurrences))]
    verbose: u64,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "run")]
    Run {
        #[structopt(long = "bots")]
        bots: Option<usize>,
        #[structopt(long = "src")]
        src: Option<String>,
        #[structopt(long = "tgt")]
        target: Option<String>,
        #[structopt(long = "output")]
        output: Option<String>,
    },
    #[structopt(name = "ci")]
    Ci {},
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    loggerv::init_with_verbosity(opt.verbose).unwrap();
    match opt.cmd {
        Command::Run {
            bots,
            src,
            target,
            output,
        } => nanobot::run(bots, src, target, output),
        Command::Ci {} => nanobot::ci(),
    }
}
