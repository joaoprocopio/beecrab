# rowgobrr

1 billion row challenge in rust.

## profilling

tools:

- perf linux
- callgrind
- valgrind
- cachegrind

record:

<!-- https://nnethercote.github.io/perf-book/profiling.html -->
<!-- https://rustc-dev-guide.rust-lang.org/profiling/with-perf.html -->

```sh
perf record -F99 --call-graph dwarf ./target/release/rowgobrrr
```

visualize:

```sh
perf script > perf.txt
```

paste on: https://profiler.firefox.com/
