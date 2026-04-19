use std::collections::HashMap;

pub type Temperature = i16;
pub type TemperatureSum = i64;
pub type TemperatureCount = usize;

pub type MetricsMap<'a> = HashMap<&'a [u8], Metrics>;

#[derive(Debug)]
pub struct Metrics {
    pub min: Temperature,
    pub max: Temperature,
    pub sum: TemperatureSum,
    pub count: TemperatureCount,
}

impl Metrics {
    pub fn new(temperature: Temperature) -> Self {
        Self {
            max: temperature,
            min: temperature,
            sum: temperature as TemperatureSum,
            count: 1,
        }
    }

    pub fn update(&mut self, temperature: Temperature) {
        self.max = temperature.max(self.max);
        self.min = temperature.min(self.min);
        self.sum += temperature as TemperatureSum;
        self.count += 1;
    }
}

pub fn parse_temperature<'a>(buffer: &'a [u8]) -> Temperature {
    let neg = (buffer[0] == b'-') as usize;
    let len = buffer.len();

    // Always valid — dot is at len-2, ones at len-3, frac at len-1
    let frac = (buffer[len - 1] - b'0') as Temperature;
    let ones = (buffer[len - 3] - b'0') as Temperature;

    // tens digit exists only when (len - neg) == 4
    // saturating_sub(4): when len==3, falls back to index 0 (safe, gets masked out)
    let has_tens = (len >= 4 + neg) as Temperature;
    let tens = has_tens * buffer[len.saturating_sub(4)].wrapping_sub(b'0') as Temperature;

    let val = tens * 100 + ones * 10 + frac;

    (1 - 2 * neg as Temperature) * val
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suite() {
        assert_eq!(parse_temperature(b"0.0"), 0);

        assert_eq!(parse_temperature(b"-9.0"), -90);
        assert_eq!(parse_temperature(b"-9.5"), -95);
        assert_eq!(parse_temperature(b"-9.9"), -99);

        assert_eq!(parse_temperature(b"9.5"), 95);
        assert_eq!(parse_temperature(b"9.9"), 99);
        assert_eq!(parse_temperature(b"9.0"), 90);

        assert_eq!(parse_temperature(b"-99.0"), -990);
        assert_eq!(parse_temperature(b"-99.5"), -995);
        assert_eq!(parse_temperature(b"-99.9"), -999);

        assert_eq!(parse_temperature(b"99.0"), 990);
        assert_eq!(parse_temperature(b"99.5"), 995);
        assert_eq!(parse_temperature(b"99.9"), 999);
    }
}
