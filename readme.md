# beecrab

1 billion row challenge in rust.

## generating measurements

```sh
cd 1brc
./mvnw clean verify
./create_measurements.sh 1000000000
./calculate_average_baseline.sh
```

## profilling

tools:

- perf linux
- callgrind
- valgrind
- cachegrind

build:

```sh
cargo build --release
```

record:

<!-- https://nnethercote.github.io/perf-book/profiling.html -->
<!-- https://rustc-dev-guide.rust-lang.org/profiling/with-perf.html -->

```sh
perf record -F99 --call-graph dwarf ./target/release/beecrab
```

visualize (TUI):

```sh
perf report
```

visualize (GUI):

```sh
perf script > perf.txt
# paste the text file on: https://profiler.firefox.com/
```
