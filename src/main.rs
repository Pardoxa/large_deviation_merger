use{
    std::{path::PathBuf, fs::File},
    serde_json::to_writer_pretty,
};

mod file_parser;
mod parser;
mod worker;
use std::io::BufWriter;

pub use file_parser::*;
pub use parser::*;
pub use worker::*;

fn main() {

    let log_cols = vec![
        LogCol::new(2),
        LogCol::new(3)
    ];

    let test = FileInfo{
        path: PathBuf::new(),
        index_hist_left: 0,
        index_hist_right: None,
        log_cols,
        comment: None,
        sep: None
    };

    let test_json = File::create("FileInfo.json")
        .expect("unable to create file");
    let buf = BufWriter::new(test_json);

    to_writer_pretty(buf, &test)
        .unwrap();

    let job = parser::parse("testconfig.json");

    println!("Job: {:?}", job);

    job.work()
}
