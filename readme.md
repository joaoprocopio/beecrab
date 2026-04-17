# rowgobrr

1 billion row challenge in rust.

## profilling

tools:

- perf linux
- callgrind
- valgrind
- cachegrind

record:

```sh
perf record --call-graph dwarf ./target/release/rowgobrrr
```

visualize:

```sh
perf script > perf.txt
```

paste on: https://profiler.firefox.com/
