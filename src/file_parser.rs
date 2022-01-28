use std::{path::PathBuf, io::{BufReader, BufRead}, str::FromStr};
use serde::{Serialize, Deserialize};
use core::ops::Deref;
use std::fs::File;
use sampling::{HistIsizeFast, Histogram};
use core::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileInfo{
    pub path: PathBuf,
    pub index_hist_left: usize,
    pub index_hist_right: Option<usize>,
    pub log_cols: Vec<LogCol>,
    pub comment: Option<String>,
    pub sep: Option<String>
}

pub enum LeftRight{
    Left,
    Right
}

pub struct HistIndex
{
    which: LeftRight,
    index: usize
}


impl FileInfo{

    fn collect_vals<'a, I, I2, T>(&self, mut iter: I, sorted_index_iter: I2, target: &mut [Vec<T>])
    where I: Iterator<Item=&'a str>,
        I2: Iterator<Item=usize>,
        T: FromStr, <T as FromStr>::Err: Debug
    {
        let mut last_index_absolute = 0;
        sorted_index_iter
            .zip(target.iter_mut())
            .for_each(
                |(index_absolute, vec)|
                {
                    let index_rel = index_absolute - last_index_absolute;
                    last_index_absolute = index_absolute + 1;
                    let nth = match iter.nth(index_rel){
                        Some(v) => v,
                        None => panic!("Error, column {:?} does not exist in {:?}", index_absolute, self)
                    };
                    
                    let val: T = match nth.parse(){
                        Ok(v) => v,
                        Err(e) => panic!("Parse error {:?} in column {:?} in {:?}", e, index_absolute, self)
                    };
                    vec.push(val);
                }
            )
    }

    pub fn sort_cols(&mut self)
    {
        self.log_cols.sort_unstable_by_key(|s| *s.deref());
        let len = self.log_cols.len();
        self.log_cols
            .dedup_by_key(|a| a.index);

        if len != self.log_cols.len() {
            eprintln!("Warning, deleted duplicate columns in {:?}", self);
        }
    }

    pub fn get_log_prob(&self) -> Vec<Vec<f64>>
    {
        let file = File::open(&self.path)
            .expect("unable to open file");
        let buf_reader = BufReader::new(file);

        let mut log_probs = vec![Vec::new(); self.log_cols.len()];

        for line in buf_reader.lines()
        {
            let line = line.unwrap();
            if let Some(pattern) = &self.comment
            {
                if line.starts_with(pattern){
                    continue;
                }
            }
            let iter = self.log_cols.iter().map(|e| e.index);

            match &self.sep{
                Some(sep) => self.collect_vals(line.split(sep), iter, &mut log_probs),
                None => self.collect_vals(line.split_whitespace(), iter, &mut log_probs)
            };
        }

        log_probs
    }

    pub fn get_hist_fast(&self) -> HistIsizeFast
    {
        let file = File::open(&self.path)
            .expect("unable to open file");
        let buf_reader = BufReader::new(file);

        let mut hist_indizes = vec![
            HistIndex{
                index: self.index_hist_left,
                which: LeftRight::Left
            }];
        if let Some(index) = self.index_hist_right{
            hist_indizes.push(
                HistIndex{
                    index,
                    which: LeftRight::Right
                }
            );
        }

        hist_indizes.sort_unstable_by_key(|e| e.index);

        let mut hist_bins: Vec<Vec<isize>> = vec![Vec::new(); hist_indizes.len()];


        for line in buf_reader.lines()
        {
            let line = line.unwrap();
            if let Some(pattern) = &self.comment
            {
                if line.starts_with(pattern){
                    continue;
                }
            }

            let iter = hist_indizes.iter().map(|e| e.index);

            match &self.sep{
                Some(sep) => self.collect_vals(line.split(sep), iter, &mut hist_bins),
                None => self.collect_vals(line.split_whitespace(), iter, &mut hist_bins)
            };
        }
        
        match self.index_hist_right{
            Some(_) => {
                let mut left = None;
                let mut right = None;
                
                hist_bins.iter()
                    .zip(hist_indizes.iter())
                    .for_each(
                        |(bins, index)|
                        {
                            match index.which{
                                LeftRight::Left => {
                                    left = bins.first().copied()
                                },
                                LeftRight::Right => {
                                    right = bins.last().copied()
                                }
                            }
                        }
                    );
                
                HistIsizeFast::new(left.unwrap(), right.unwrap())

            },
            None => {
                let left = *hist_bins[0].first().unwrap();
                let right_inclusive = *hist_bins[0].last().unwrap();
                HistIsizeFast::new_inclusive(left, right_inclusive)
            }
        }.unwrap()
    }

    pub fn get_logs_and_hists(&self) -> (Vec<HistIsizeFast>, Vec<Vec<f64>>)
    {
        let mut logs = self.get_log_prob();
        let e_hist = self.get_hist_fast();

        let hists: Vec<_> = logs.iter_mut()
            .zip(self.log_cols.iter())
            .map(
                |(log_vec, col)|
                {
                    if !log_vec.iter().any(|v| v.is_finite()){
                        panic!("Only Invalid entries in {:?} in {:?}", col, self)
                    }
                    
                    let mut index_left = 0;
                    for (index, val) in log_vec.iter().enumerate()
                    {
                        if !val.is_nan(){
                            index_left = index;
                            break;
                        }
                    }

                    let mut index_right = log_vec.len() - 1;
                    for (index, val) in log_vec.iter().enumerate().rev()
                    {
                        if !val.is_nan(){
                            index_right = index;
                            break;
                        }
                    }

                    if let Some(trim_left) = col.trim_left{
                        index_left += trim_left;
                        if index_left >= index_right {
                            eprintln!("Did you trim to much? {:?} {:?}", col, self);
                        }
                    }

                    if let Some(trim_right) = col.trim_right{
                        index_right -= trim_right;
                        if index_left >= index_right {
                            eprintln!("Did you trim to much? {:?} {:?}", col, self);
                        }
                    }

                    // remove NaNs
                    *log_vec = log_vec[index_left..=index_right].to_vec();
                    
                    let mut iter = e_hist.bin_iter();

                    let left = iter.nth(index_left).unwrap();
                    let diff = index_right - index_left - 1;
                    let right = iter.nth(diff).unwrap();
                    let hist = HistIsizeFast::new_inclusive(left, right).unwrap();
                    
                    assert_eq!(hist.bin_count(), log_vec.len(), "Lenght of Hist does not match length of logvec");
                    hist
                }
            ).collect();
        (hists, logs)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogCol{
    pub index: usize,
    pub trim_right: Option<usize>,
    pub trim_left: Option<usize>
}

impl LogCol{
    pub fn new(index: usize) -> Self
    {
        Self{
            index,
            trim_left: None,
            trim_right: None
        }
    }
}

impl Deref for LogCol
{   
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.index
    }
}