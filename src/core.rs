pub type Temperature = f64;

#[derive(Debug)]
pub struct Status {
    pub min: Temperature,
    pub max: Temperature,
    pub sum: Temperature,
    pub count: usize,
}

impl Status {
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
