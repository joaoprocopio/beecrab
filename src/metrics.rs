use std::collections::HashMap;

pub type Temperature = i64;
pub type TemperatureCount = usize;

pub type MetricsMap<'a> = HashMap<&'a [u8], Metrics>;

#[derive(Debug)]
pub struct Metrics {
    pub min: Temperature,
    pub max: Temperature,
    pub sum: Temperature,
    pub count: TemperatureCount,
}

impl Metrics {
    pub fn new(temperature: Temperature) -> Self {
        Self {
            max: temperature,
            min: temperature,
            sum: temperature,
            count: 1,
        }
    }

    pub fn update(&mut self, temperature: Temperature) {
        self.max = temperature.max(self.max);
        self.min = temperature.min(self.min);
        self.sum += temperature;
        self.count += 1;
    }
}
