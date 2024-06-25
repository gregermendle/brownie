pub struct LowPassFilter {
    cutoff: f32,
    sample_rate: f32,
    prev_output: f32,
}

impl LowPassFilter {
    pub fn new(cutoff: f32, sample_rate: f32) -> Self {
        LowPassFilter {
            cutoff,
            sample_rate,
            prev_output: 0.0,
        }
    }

    pub fn apply(&mut self, input: f32) -> f32 {
        let rc = 1.0 / (self.cutoff * 2.0 * std::f32::consts::PI);
        let dt = 1.0 / self.sample_rate;
        let alpha = dt / (rc + dt);

        let output = self.prev_output + alpha * (input - self.prev_output);

        self.prev_output = output;

        output
    }
}
