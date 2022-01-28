use sampling::LogBase;

use crate::*;


#[derive(Debug)]
pub struct Job{
    pub out: String,
    pub file_infos: Vec<FileInfo>,
    pub hist_type: HistType,
    pub merge_type: MergeType
}


impl Job{
    pub fn work(&self) 
    {
        let (hists, log_probs): (Vec<_>, Vec<_>) = self.file_infos.iter()
            .flat_map(
                |file_info|
                {
                    let (hist, log) = file_info.get_logs_and_hists();

                    hist.into_iter()
                        .zip(log.into_iter())
                }
            ).unzip();
        let glued = match self.merge_type
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