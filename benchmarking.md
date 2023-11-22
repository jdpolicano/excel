Reading in million line csv file, parsing it to a struct, and then writing it back out to a csv file.


With &str: 
1. $ ./target/release/excel_takehome test_data/large.csv  4.93s user 0.46s system 93% cpu 5.748 total
2. $ ./target/release/excel_takehome test_data/large.csv  4.93s user 0.44s system 98% cpu 5.431 total
3. $ ./target/release/excel_takehome test_data/large.csv  4.91s user 0.34s system 99% cpu 5.297 total

With &[char] references:
1. $ ./target/release/excel_takehome test_data/large.csv  4.61s user 0.39s system 93% cpu 5.377 total
2. $ ./target/release/excel_takehome test_data/large.csv  4.47s user 0.31s system 99% cpu 4.807 total
3. $ ./target/release/excel_takehome test_data/large.csv  4.36s user 0.30s system 99% cpu 4.685 total

With &[char] references and not cloning when writing to disk:
1. $ ./target/release/excel_takehome test_data/large.csv  4.61s user 0.39s system 93% cpu 5.377 total
2. $ ./target/release/excel_takehome test_data/large.csv  4.47s user 0.31s system 99% cpu 4.807 total
3. $ ./target/release/excel_takehome test_data/large.csv  4.36s user 0.30s system 99% cpu 4.685 total
