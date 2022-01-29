use std::{fs::File, io::BufReader};
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, Value};

use crate::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HistType
{
    HistUsizeFast,
    HistIsizeFast
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MergeType{
    Average,
    Derivative
}

impl Default for HistType
{
    fn default() -> Self {
        Self::HistIsizeFast
    }
}

pub fn parse(file: &str) -> Job
{
    let file = File::open(file)
        .expect("unable to open file");
    
    let reader = BufReader::new(file);

    let json: Value = from_reader(reader)
        .expect("Invalid Json");

    let file_infos_json = json.get("files")
        .expect("Json is missing array 'files'");
    
    assert!(file_infos_json.is_array(), "'files' must be an array of file infos!");
    let file_array = file_infos_json.as_array().unwrap();

    let comment = json.get("global_comment")
        .map(
            |v|
            {
                v.as_str().expect("Invalid 'global_comment'").to_owned()
            }
        );


    let mut file_infos: Vec<FileInfo> = file_array.iter()
        .map(
            |obj| 
            {
                serde_json::from_value(obj.clone())
                    .expect("Invalid Json in 'files' array")
            }
            )
        .collect();

    if comment.is_some() {
        file_infos.iter_mut()
            .filter(|file_info| file_info.comment.is_none())
            .for_each(
                |f|
                f.comment = comment.clone()
            );
    }

    file_infos.iter_mut().for_each(FileInfo::sort_cols);

    let hist_type = match json.get("hist"){
        Some(val) => {
            match serde_json::from_value(val.clone()){
                Ok(hist) => hist,
                Err(e) => {
                    eprintln!("Error: {:?} Invalid Hist Type - currently only one type is implemented, so I will fallback to the default", e);
                    // TODO Print out valid types
                    HistType::default()
                }
            }
        },
        None => {
            eprintln!("Warning, no hist type ('hist') specified, using default: {:?}", HistType::default());
            HistType::default()
        }
    };

    let out = match json.get("out")
    {
        Some(v) => {
            match v.as_str(){
                Some(out) => out.to_owned(),
                None => panic!("Invalid output type - should be string. Note: This is the file that will be created for the output")
            }
        },
        None => {
            let def = "merged.glued";
            eprintln!("Missing output name ('out') - using default - {}", def);
            def.to_owned()                        
        }
    };

    let merge: MergeType = match json.get("merge")
    {
        Some(v) => {
            serde_json::from_value(v.clone())
                .expect("Invalid Merge type ('merge')")
        },
        None => MergeType::Average
    };

    let bin_size = json.get("bin_size")
        .map(
            |size|
            {
                match size.as_f64(){
                    Some(v) => v,
                    None => {
                        let error = "bin_size parsing error";
                        let s = size.as_str()
                            .expect(error);
                        s.parse()
                            .expect(error)
                    }
                }
            }
        );

    let bin_start = json.get("bin_starting_point")
        .map(
            |size|
            {
                match size.as_f64(){
                    Some(v) => v,
                    None => {
                        let error = "bin_starting_point parsing error";
                        let s = size.as_str()
                            .expect(error);
                        s.parse()
                            .expect(error)
                    }
                }
            }
        );

    Job { 
        out, 
        hist: hist_type,
        files: file_infos, 
        merge,
        global_comment: comment,
        bin_size,
        bin_starting_point: bin_start
    }
}