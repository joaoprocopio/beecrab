# beecrab

1 billion row challenge in rust.

## development

### generating data

> to run this project you need to generate the base text file. read the [guide](1brc/README.md) here.

```sh
cd 1brc
./mvnw clean verify
./create_measurements.sh 1000000000
./calculate_average_baseline.sh > baseline.txt
mv measurements.txt ..
mv baseline.txt ..
```

### tools used for profilling

- perf linux
- callgrind
- valgrind
- cachegrind

### building the binary

```sh
cargo build --release
```

### sampling

<!-- https://nnethercote.github.io/perf-book/profiling.html -->
<!-- https://rustc-dev-guide.rust-lang.org/profiling/with-perf.html -->

```sh
perf record -F99 --call-graph dwarf ./target/release/beecrab measurements.txt
```

### visualize perf data on a tui

```sh
perf report
```

### visualize perf data on a gui

```sh
perf script > perf.txt
# paste the text file on: https://profiler.firefox.com/
```

## data insights

1. there is a total of `413` different stations
2. the longest station name is `Las Palmas de Gran Canaria` with `26` characters
3. the shortest station name is `Wau` and `Jos` with 3 characters
4. the distribution for station names looks something like:
   - mean: `7.8`
   - median: `7`
   - mode: `6`
5. temperatures are always in a inclusive range of `-99.9` to `99.9`
