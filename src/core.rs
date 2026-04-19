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
    let len = buffer.len();
    let is_negative = buffer[0] == b'-';
    let sign_multiplier = Temperature::from(!is_negative) * 2 - 1; // if negative -> -1; if positive -> 1
    let start_pos = usize::from(is_negative);
    let i1 = buffer[start_pos];

    dbg!(start_pos);
    dbg!(i1 as char);

    0
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
