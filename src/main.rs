use{
    structopt::StructOpt
};

mod file_parser;
mod parser;
mod worker;

pub use file_parser::*;
pub use parser::*;
pub use worker::*;

fn main() {

    let opt = Opt::from_args();
    match opt{
        Opt::Merge(m) => merge(m),
        Opt::LogColRange(opt) => print_log_col_range(opt),
        Opt::ExampleJson => worker::example(),
        Opt::CreateJob(opt) => glob_create(opt)
    }
}


#[derive(Debug, StructOpt, Clone)]
#[structopt(about = "Merge your WangLandau Probabilities. You can also insert simple sampling probabilities")]
pub enum Opt
{
    Merge(Merge),
    LogColRange(LogColRange),
    /// Prints an example json file. This json file is needed for the merging to specify what you want to merge
    ExampleJson,
    CreateJob(CreateJob)
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
    /// Leftest column
    pub left: usize,

    #[structopt(long, short)]
    /// rightest column
    pub right: usize,

    #[structopt(long)]
    /// trim left to be used everywhere
    pub trim_left: Option<usize>,

    #[structopt(long)]
    /// trim right to be used everywhere
    pub trim_right: Option<usize>,
}

/// For quickly creating the file list from a pattern
#[derive(Debug, Clone, StructOpt)]
pub struct CreateJob
{
    #[structopt(long, short)]
    /// Pattern of all the files you want to include - uses globbing so, e.g., the wildcard * works
    pub globbing: String,

    #[structopt(long)]
    /// Which column represents the histogram?
    /// If ommited, the line-number (without counting comments)
    /// will be used as bin
    pub hist_col_left: Option<usize>,

    #[structopt(long)]
    /// Currently not nessessary, in here for future expansions
    pub hist_col_right: Option<usize>,

    #[structopt(long)]
    /// which columns contain the logarithmic probabilities? - left border
    pub log_col_left: usize,

    #[structopt(long, short)]
    /// rightest column, exclusive. Can be omitted if only one column should be considered
    pub log_col_right: Option<usize>,

    /// Name of output file
    #[structopt(long, short, default_value = "merged.out")]
    pub out: String,

    #[structopt(long, short, default_value = "Average")]
    /// Which merge algorithm to use? Options: "Average" or "Derivative"
    pub merge: MergeType,

    #[structopt(long)]
    /// Setting the global comment
    pub global_comment: Option<String>,

    #[structopt(long)]
    /// Setting the bin size
    pub bin_size: Option<f64>,

    #[structopt(long, short)]
    /// Setting the bin starting point
    pub bin_starting_point: Option<f64>,

    #[structopt(long, short)]
    /// Instead of printing the json file to the terminal,
    /// it will be written into the file instead
    pub job_file: Option<String>,

    #[structopt(long, short)]
    /// Used to shift the histograms
    pub shift: Option<isize>
}