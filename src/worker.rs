use sampling::LogBase;
use serde::{Serialize, Deserialize};
use std::{fs::File, io::BufWriter};

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
    pub global_comment: Option<String>
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
        path: "file1.dat".to_owned(),
        index_hist_left: 0,
        comment: None,
        sep: None,
        log_cols: log_cols1,
        index_hist_right: None,
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
        path: "file2.dat".to_owned(),
        index_hist_left: 0,
        comment: Some("%".to_owned()),
        sep: Some(",".to_owned()),
        log_cols: log_cols2,
        index_hist_right: Some(1),
    };

    let file_vec = vec![file_info1, file_info2];

    let job = Job{
        out: output.to_owned(),
        files: file_vec,
        merge: MergeType::Average,
        hist: HistType::HistIsizeFast,
        global_comment: Some("#".to_owned())
    };

    serde_json::to_writer_pretty(std::io::stdout(), &job).unwrap();
    
}


impl Job{
    pub fn work(&self) 
    {
        let (hists, log_probs): (Vec<_>, Vec<_>) = self.files.iter()
            .flat_map(
                |file_info|
                {
                    let (hist, log) = file_info.get_logs_and_hists();

                    hist.into_iter()
                        .zip(log.into_iter())
                }
            ).unzip();
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

        glued.write(buf).unwrap()
    }
}