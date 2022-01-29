# Example

Here I have two files in two folders which contain simulation results.

To merge them I now create the required json file via:

```bash
large_deviation_merger create-job -g '*/*.dat' --hist-col-left 0 --log-col-left 3 --log-col-right 27 -j job.json --global-comment '#'
```

Now I start the merging process

```bash
large_deviation_merger merge -j job.json
```

Since I did not specify the output name it has been written to `merged.out`

Now I look at the result with gnuplot

```gnuplot
p "merged.out"
```

The result looks a bit weird. Lets look at the aligned intervals.

```gnuplot
p for[i=3:50]"merged.out" u 1:i
```
It looks like the last interval had problems sampling.
Now I manually edit the job to exclude the last interval.
I save the new job at jobv2.json.

I just changed `out` to `mergedv2.out`
and removed the interval that gave me problems.

```bash 
large_deviation_merger merge -j jobv2.json
```

```gnuplot
p "mergedv2.out"
```

That looks better!

Lets say I also want to include my simple sampling results.
I edit the job to also use my simple sampling results jobv3.json.
Note that I had to use a shift value, because I changed the origin of my energyaxis between large deviation 
and simple sampling.

I change the json one last time to jobv4.json
I noticed, that for the first few values I should only use simple sampling, so I trim the first large deviation interval from the left.
Also the statistics of the simple sampling are not that good for the righter most values, so I trim that from the right.

Note that I also show of another posibility for the histogram (For the SimpleSampling file): 
If you do not specify a column for the histogram,
it will use the line number as bin (comments are not counted)

comparing the intervals
```gnuplot
p for[i=3:4] "mergedv3.out" u 1:i, for[i=3:4] "mergedv4.out" u 1:i
p "mergedv3.out" u 1:2, "mergedv4.out" u 1:2
```

And to compare everything linearly
```gnuplot
p "mergedv3.out" u 1:(10**$2), "mergedv4.out" u 1:(10**$2), "SimpleSample_v0.18.6-alpha_M_N3200_t0.1_r0.14_Sw_0.1_ES7485_MS52312_S10000000j24.dat" u ($3-1):(10**$2*1.0/3200)
```

