//! Extensions for things not in vizia

use std::ops::RangeInclusive;

use num_traits::Num;

pub trait RangeExt<T>
where
    T: Num + Copy,
{
    fn width(&self) -> T;
    // Normalize `T`
    fn map(&self, value: T) -> T;
}

impl<T> RangeExt<T> for RangeInclusive<T>
where
    T: Num + Copy,
{
    fn width(&self) -> T {
        *self.end() - *self.start()
    }

    fn map(&self, value: T) -> T {
        (value - *self.start()) / self.width()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn positive_range_width() {
        assert_approx_eq!((0.2f32..=0.8).width(), 0.6);
    }

    #[test]
    fn negative_range_width() {
        assert_approx_eq!((-0.2f32..=0.2).width(), 0.4);
    }

    #[test]
    fn get_mapped() {
        let tests = [
            (-1f32..=1f32, 0.0),
            (0f32..=1f32, 0.5),
            (-1f32..=0f32, -0.5),
            (0f32..=5f32, 2.5),
            (-5f32..=0f32, -2.5),
            (2.5f32..=-2.5f32, 0f32),
        ];
        for (range, value) in tests {
            let mapped = range.map(value);
            assert_approx_eq!(mapped, 0.5f32);
        }
    }
}
