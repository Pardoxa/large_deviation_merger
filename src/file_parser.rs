use std::{io::{BufReader, BufRead, Read}, str::FromStr};
use serde::{Serialize, Deserialize};
use core::ops::Deref;
use std::fs::File;
use sampling::{HistIsizeFast, Histogram};
use core::fmt::Debug;
use crate::{LogColRange};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileInfo{
    pub path: String,
    pub index_hist_left: Option<usize>,
    pub index_hist_right: Option<usize>,
    pub log_cols: Vec<LogCol>,
    pub comment: Option<String>,
    pub sep: Option<String>,
    pub shift: Option<isize>
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

    fn count_lines<T>(&self, reader: BufReader<T>) -> isize
    where T: Read
    {
        let mut counter: isize = 0;
        for line in reader.lines()
        {
            let s = match line {
                Ok(s) => s,
                Err(e) => {
                    panic!("Error in {:?} while reading file. Error is {:?}", self, e)
                }
            };
            match &self.comment
            {
                Some(c) => {
                    if !s.starts_with(c)
                    {
                        counter +=1;
                    }
                },
                None => counter +=1
            }
            
        }
        counter
    }

    pub fn sort_cols(&mut self)
    {
        self.log_cols.sort_unstable_by_key(|s| s.index);
        let len = self.log_cols.len();
        self.log_cols
            .dedup_by_key(|a| a.index);

        if len != self.log_cols.len() {
            eprintln!("Warning, deleted duplicate columns in {:?}", self);
            eprintln!("Note: If you actually wish to include an interval muliple times you can \
                - you have to specify the same file twice in the file array");
        }
    }

    pub fn get_log_prob(&self) -> Vec<Vec<f64>>
    {
        let file = File::open(&self.path)
            .expect("unable to open file");
        let buf_reader = BufReader::new(file);

        let mut log_probs: Vec<Vec<f64>> = vec![Vec::new(); self.log_cols.len()];

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

        log_probs.iter_mut()
            .for_each(
                |v|
                {
                    v.iter_mut()
                        .for_each(
                            |val|
                            {
                                if !val.is_finite()
                                {
                                    *val = f64::NAN;
                                }
                            }
                        )
                }
            );

        log_probs
    }

    pub fn get_hist_fast(&self) -> HistIsizeFast
    {
        let file = File::open(&self.path)
            .expect("unable to open file");
        let buf_reader = BufReader::new(file);

        let shift = self.shift.unwrap_or(0);

        let index_hist_left = match self.index_hist_left {
            Some(v) => v,
            None => {
                let lines = self.count_lines(buf_reader);
                return HistIsizeFast::new(shift, lines + shift)
                    .expect("unable to create histogram");
            }
        };

        let mut hist_indizes = vec![
            HistIndex{
                index: index_hist_left,
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

        hist_bins.iter()
            .for_each(
                |hist_vec| 
                {
                    let sorted = hist_vec.iter()
                        .zip(hist_vec.iter().skip(1))
                        .all(|(&a, &b)| a < b);
                    assert!(
                        sorted, 
                        "Error: Your Histogram is not ordered correctly- it has to start with the smallest value and go up monotonically {:?}", 
                        self
                    );
                }
            );
        
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
                
                HistIsizeFast::new(shift + left.unwrap(), shift + right.unwrap())

            },
            None => {
                let left = *hist_bins[0].first().unwrap();
                let right_inclusive = *hist_bins[0].last().unwrap();
                HistIsizeFast::new_inclusive(shift + left, shift + right_inclusive)
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

                    // remove NaNs and trim interval
                    *log_vec = log_vec[index_left..=index_right].to_vec();
                    
                    let mut iter = e_hist.bin_iter();

                    let left = iter.nth(index_left).unwrap();
                    let diff = index_right - index_left - 1;
                    let right = iter.nth(diff).unwrap();
                    let hist = HistIsizeFast::new_inclusive(left, right).unwrap();
                    
                    assert_eq!(hist.bin_count(), log_vec.len(), "Lenght of Hist does not match length of log_vec");
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

pub fn print_log_col_range(opt: LogColRange)
{
    let v: Vec<_> = (opt.left..opt.right)
        .map(
            |index|
            {
                LogCol{
                    index,
                    trim_left: opt.trim_left,
                    trim_right: opt.trim_right
                }
            }
        ).collect();
    print!("\"log_cols\": ");
    serde_json::to_writer_pretty(std::io::stdout(), &v).unwrap()
}

