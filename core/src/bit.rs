// Helper methods on u8 for bit manipulation.

pub trait Bit {
    fn get(&self, n: usize) -> bool;
    fn set(&mut self, n: usize);
}

impl Bit for u8 {
    fn get(&self, n: usize) -> bool {
        self & (1 << n) == (1 << n)
    }

    fn set(&mut self, n: usize) {
        *self |= 1 << n;
    }
}

#[cfg(test)]
mod test {
    use super::Bit;

    #[test]
    fn get() {
        let num: u8 = 50;
        assert!(num.get(1));
        assert!(!num.get(0));
        assert!(!num.get(2));
        assert!(!num.get(3));
        assert!(num.get(4));
        assert!(num.get(5));
    }

    #[test]
    fn set() {
        let mut num: u8 = 0;
        num.set(3);
        assert_eq!(num, 8);
        num.set(0);
        assert_eq!(num, 9);
    }
}
