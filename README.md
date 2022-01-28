# large_deviation_merger

Used to Merge Large deviation WangLandau and Entropic sampling simulations.


## Example json

```json
{
  "out": "output.dat",`<-- Created output file
  "files": [ <-- array of files you want to read in
    {
      "path": "RELATIVE_PATH_FROM_WHERE_YOU_ARE/file1.dat", <- path of first file
      "index_hist_left": 0, <- in the file the left most column corresponds to the histogram bins
      "index_hist_right": null, <- we do not have/need a second column
      "log_cols": [ <- indices of the logarithmic probability
        {
          "index": 3, <-- the forth colum is the first one we wish to use
          "trim_right": null, <-- We do not which to trim the interval from the right - note: NaNs are automatically trimed
          "trim_left": null <-- We do not which to trim the interval from the left - note: NaNs are automatically trimed
        },
        {
          "index": 4,
          "trim_right": null,
          "trim_left": null
        }
      ],
      "comment": null, <-- Comments are specified by the global comment, here "#"
      "sep": null <-- No seperator is specified, all whitespace characters will do
    },
    {
      "path": "ABSOLUTE_PATH/file2.dat", <- path of second file
      "index_hist_left": 0,
      "index_hist_right": 1,
      "log_cols": [
        {
          "index": 3,
          "trim_right": null,
          "trim_left": null
        },
        {
          "index": 4,
          "trim_right": 14, <-- we want to remove 14 numbers from the right - after the NaNs are already removed
          "trim_left": null
        }
      ],
      "comment": "%", <-- In this file, comments are specified by "%"
      "sep": "," <-- also, the seperator is "%"
    }
  ],
  "hist": "HistIsizeFast", <--- No other Histogram is implemented yet
  "merge": "Average", <--- alternative mode: Derivative
  "global_comment": "#", <-- Specify what a line must start with to be ignored
  "bin_size": null, <--- you can specify a bin size, to normalize the integral instead of the sum
  "bin_starting_point": null <-- if you do specify a bin_size, what were should the merged interval start? 
}
```
