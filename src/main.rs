#![deny(clippy::all)]

use std::env::current_dir;

/// each line inside the measurements.txt file is in the following format `<string: station_name>;<f64: measurement>`
///
/// where each measurement has exactly one single fractional digit
///
/// ```
/// let measurements = "
///     Hamburg;12.0
///     Bulawayo;8.9
///     Palembang;38.8
///     St. John's;15.2
///     Cracow;12.6
///     Bridgetown;26.9
///     Istanbul;6.2
///     Roseau;34.4
///     Conakry;31.2
///     Istanbul;23.0
/// "
/// ```
///
/// the task is to read the whole file, and:
/// - calculate the min temperature;
/// - calculate the mean temperature;
/// - calculate the max temperature.
///
/// for each weather station, and emit the result in the stdout:
/// sorted alphabetically by station name, and the result values per station in the format `<min>/<mean>/<max>`, rounded to one fractional digit.
///
/// ```
/// let result = "{Abha=-23.0/18.0/59.2, Abidjan=-16.2/26.0/67.3, Abéché=-10.0/29.4/69.0, ...}"
/// ```

fn main() {
    let path = current_dir()
        .and_then(|dir| Ok(dir.join("data/measurements.txt")))
        .unwrap();
    dbg!(path);
    // let measurements = File::open(path)

    println!("Hello, world!");
}
