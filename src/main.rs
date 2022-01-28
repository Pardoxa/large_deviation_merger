use{
    structopt::StructOpt
};

mod file_parser;
mod parser;
mod worker;
mod cmd;

pub use file_parser::*;
pub use parser::*;
pub use worker::*;
pub use cmd::*;

fn main() {

    let opt = Opt::from_args();
    match opt{
        Opt::Merge(m) => merge(m),
        Opt::LogColRange(opt) => print_log_col_range(opt),
        Opt::ExampleJson => worker::example()
    }
}


#[derive(Debug, StructOpt, Clone)]
#[structopt(about = "Merge your WangLandau Probabilities. You can also insert simple sampling probabilities")]
pub enum Opt
{
    Merge(Merge),
    LogColRange(LogColRange),
    /// Prints an example json file. This json file is needed for the merging to specify what you want to merge
    ExampleJson
}

#[derive(Debug, Clone, StructOpt)]
/// Merge logarithmic probability densitys from Wang Landau (or Entropic Sampling)
pub struct Merge
{
    #[structopt(long, short)]
    /// path to json file, which specifies the merge job
    pub json: String
}

#[derive(Debug, Clone, StructOpt)]
/// As the json-array log_cols is a bit inconvinient,
/// this helps in creating it. Try it out.
pub struct LogColRange
{
    #[structopt(long, short)]
    /// Leftest index
    pub left: usize,

    #[structopt(long, short)]
    /// rightest index
    pub right: usize,

    #[structopt(long)]
    /// trim left to be used everywhere
    pub trim_left: Option<usize>,

    #[structopt(long)]
    /// trim right to be used everywhere
    pub trim_right: Option<usize>,
}
