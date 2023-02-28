
// Each timer has an internal counter that is decremented on each input clock.
// When the counter becomes zero, it is reloaded with the period and an output clock is generated.
#[derive(Default)]
pub struct Clock {
    pub period: u32,
    pub n:      u32,
}

impl Clock {
    
    pub fn new(period: u32) -> Self {
        Self { period, n: 0 }
    }

    pub fn tick(&mut self, cycles: u32) -> u32 {
        self.n += cycles;
        let c = self.n / self.period;
        self.n %= self.period;
        c
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn clock() {
        let mut c = Clock::new(12);
        assert_eq!(c.tick(6), 0);
        assert_eq!(c.tick(6), 1);
        assert_eq!(c.tick(36), 3);
        assert_eq!(c.tick(18), 1);
        assert_eq!(c.tick(4), 0);
        assert_eq!(c.tick(2), 1);
    }
}