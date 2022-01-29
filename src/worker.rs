use sampling::{LogBase, IntervalOrder};
use serde::{Serialize, Deserialize};
use std::{fs::File, io::BufWriter};
use glob::glob;

use crate::*;

pub fn merge(task: Merge)
{
    let job = parser::parse(&task.json);
    job.work();
    println!("Success! Output written to {}", job.out)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Job{
    pub out: String,
    pub files: Vec<FileInfo>,
    pub hist: HistType,
    pub merge: MergeType,
    pub global_comment: Option<String>,
    pub bin_size: Option<f64>,
    pub bin_starting_point: Option<f64>
}

pub fn glob_create(options: CreateJob)
{
    let right = match options.log_col_right {
        Some(v) => v,
        None => options.log_col_left + 1
    };
    if right <= options.log_col_left {
        panic!("log_col_right must be larger than log_col_left!")
    }
    let files: Vec<_> = glob(&options.globbing)
        .expect("Error in globbing pattern")
        .filter_map(
            |entry|
            {
                match entry{
                    Err(e) => 
                    {
                        println!("Warning, globbing error! {:?}", e);
                        None
                    },
                    Ok(path) => {
                        let path = path.to_str().unwrap().to_owned();
                        
                        let log_cols: Vec<_> = (options.log_col_left..right)
                            .map(
                                |index|
                                {
                                    LogCol{
                                        index,
                                        trim_left: None,
                                        trim_right: None
                                    }
                                }
                            ).collect();

                        let f = FileInfo{
                            path,
                            index_hist_left: options.hist_col_left,
                            index_hist_right: options.hist_col_right,
                            comment: None,
                            sep: None,
                            log_cols,
                            shift: options.shift
                        };
                        Some(f)
                    }
                }
            }
        ).collect();

    let job = Job{
        files,
        bin_size: options.bin_size,
        bin_starting_point: options.bin_starting_point,
        merge: options.merge,
        out: options.out,
        global_comment: options.global_comment,
        hist: HistType::HistIsizeFast
    };

    match options.job_file{
        None => serde_json::to_writer_pretty(std::io::stdout(), &job),
        Some(file) => {
            let f = File::create(file).expect("unable to create file");
            let buf = BufWriter::new(f);
            serde_json::to_writer_pretty(buf, &job)
        }
    }.unwrap()
}

pub fn example()
{
    let output = "output.dat";

    let log_cols1: Vec<_> = (3..5)
        .map(
            |index|
            {
                LogCol::new(index)
            }
        ).collect();
    
    let file_info1 = FileInfo{
        path: "RELATIVE_PATH_FROM_WHERE_YOU_ARE/file1.dat".to_owned(),
        index_hist_left: Some(0),
        comment: None,
        sep: None,
        log_cols: log_cols1,
        index_hist_right: None,
        shift: Some(23)
    };

    let mut log_cols2: Vec<_> = (3..5)
        .map(
            |index|
            {
                LogCol::new(index)
            }
        ).collect();

    log_cols2[1].trim_right = Some(14);

    let file_info2 = FileInfo{
        path: "ABSOLUTE_PATH/file2.dat".to_owned(),
        index_hist_left: None,
        comment: Some("%".to_owned()),
        sep: Some(",".to_owned()),
        log_cols: log_cols2,
        index_hist_right: Some(1),
        shift: None
    };

    let file_vec = vec![file_info1, file_info2];

    let job = Job{
        out: output.to_owned(),
        files: file_vec,
        merge: MergeType::Average,
        hist: HistType::HistIsizeFast,
        global_comment: Some("#".to_owned()),
        bin_size: None,
        bin_starting_point: None
    };

    serde_json::to_writer_pretty(std::io::stdout(), &job).unwrap();
    
}


impl Job{
    pub fn work(&self) 
    {
        let mut container: Vec<_> = self.files.iter()
            .flat_map(
                |file_info|
                {
                    let (hist, log) = file_info.get_logs_and_hists();
                    hist.into_iter()
                        .zip(log.into_iter())
                }
            ).collect();

        // now I have to sort them! Otherwise I might get glue errors
        container
            .sort_unstable_by(|a,b| a.0.left_compare(&b.0));

        let (hists, log_probs) = container.into_iter().unzip();

        let glued = match self.merge
        {
            MergeType::Average => {
                sampling::glue::average_merged_and_aligned(log_probs, hists, LogBase::Base10)
            },
            MergeType::Derivative => {
                sampling::glue::derivative_merged_and_aligned(log_probs, hists, LogBase::Base10)
            }
        }.expect("Unable to glue");
        

            

        let output = File::create(&self.out)
            .expect("Unable to create output file");
        let buf = BufWriter::new(output);

        match self.bin_size{
            None => glued.write(buf).unwrap(),
            Some(bin_size) => {
                let s = match self.bin_starting_point{
                    Some(s) => s,
                    None => {
                        eprintln!("Warning: bin_size specified, but no bin_starting_point! Using bin_size as bin_starting_point");
                        bin_size
                    }
                };
                glued.write_rescaled(buf, bin_size, s).unwrap()
            }
        }
        
    }
}