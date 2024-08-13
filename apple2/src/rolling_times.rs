
#[derive(Default)]
struct CpuRun {
    start: u128,
    end: u128,
    delta: u128,
    cycles: u128,
}

pub struct RollingTimes {
    /// Timestamp in ms, number of cycles
    times: Vec<CpuRun>,
}

impl RollingTimes {
    pub fn new() -> Self {
        Self {
            times: Vec::new()
        }
    }

    pub fn add(&mut self, start: u128, end: u128, cycles: u128) {
        self.times.push(CpuRun { start, end, cycles, delta: end - start });
        if self.times.len() > 10 {
            self.times.remove(0);
        }
    }

    /// Return the speed in Mhz
    pub fn average(&self) -> f32 {
        let mut total_cycles = 0_u128;
        let mut s = u128::MAX;
        let mut e = u128::MIN;
        let mut total_delta = 0.0;
        for CpuRun { start, end, cycles, delta } in &self.times {
            // println!("  - Averaging slice {start}-{end} ({}) cycles:{cycles}", end - start);
            if *start < s { s = *start; }
            if *end > e { e = *end; }
            total_delta += *delta as f32;
            total_cycles += cycles;
        }

        let result = total_cycles as f32 / total_delta / 1000.0;
        // println!("Average: {result} e-s: {}", e-s);
        result
    }
}

#[test]
fn test_rolling_times() {
    use float_eq::assert_float_eq;

    let mut rt = RollingTimes::new();
    rt.add(0, 200, 500_000);
    assert_float_eq!(rt.average(), 2.5, abs <= 0.1);
    rt.add(200, 400, 200_000);
    assert_float_eq!(rt.average(), 1.75, abs <= 0.1);
    rt.add(400, 600, 200_000);
    assert_float_eq!(rt.average(), 1.5, abs <= 0.1);
    rt.add(600, 1000, 100_000);
    assert_float_eq!(rt.average(), 1.0, abs <= 0.1);
}
