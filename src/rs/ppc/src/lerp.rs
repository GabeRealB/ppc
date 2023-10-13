use std::mem::{ManuallyDrop, MaybeUninit};

/// Trait for linearly interpolating between two values.
pub trait Lerp {
    /// Creates a value between `self` and `other` by linearly interpolating
    /// them, according to a t value.
    fn lerp(self, other: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(self, other: Self, t: f32) -> Self {
        ((1.0 - t) * self) + (t * other)
    }
}

impl<T: Lerp, const N: usize> Lerp for [T; N] {
    fn lerp(self, other: Self, t: f32) -> Self {
        let mut a = PartialArrayReader::new(self);
        let mut b = PartialArrayReader::new(other);
        let mut result = PartialArrayWriter::new();

        for _ in 0..N {
            let first = a.take_one();
            let second = b.take_one();
            let interpolated = first.lerp(second, t);
            result.push(interpolated);
        }

        result.unwrap()
    }
}

/// Trait for types that can invert a linear interpolation.
pub trait InverseLerp {
    /// Inverse of [`Lerp::lerp`].
    fn inv_lerp(self, start: Self, end: Self) -> f32;
}

impl InverseLerp for f32 {
    fn inv_lerp(self, start: Self, end: Self) -> f32 {
        (self - start) / (end - start)
    }
}

struct PartialArrayReader<T, const N: usize> {
    array: [MaybeUninit<T>; N],
    values_left: usize,
}

impl<T, const N: usize> PartialArrayReader<T, N> {
    fn new(array: [T; N]) -> Self {
        // Safety: Is totally safe, as the individual entries are still `MaybeUninit`
        // and we know that the sizes match.
        let uninit = unsafe { std::mem::transmute_copy(&array) };
        std::mem::forget(array);

        Self {
            array: uninit,
            values_left: N,
        }
    }

    fn take_one(&mut self) -> T {
        if self.values_left == 0 {
            panic!("there are no more values available")
        }

        let idx = N - self.values_left;
        self.values_left -= 1;

        // Safety: We just checked that the element was not consumed.
        unsafe { self.array[idx].assume_init_read() }
    }
}

impl<T, const N: usize> Drop for PartialArrayReader<T, N> {
    fn drop(&mut self) {
        let valid_start = N - self.values_left;
        for i in valid_start..N {
            // Safety: We know that the values `valid_start..N` are initialized.
            unsafe {
                self.array[i].assume_init_drop();
            }
        }
    }
}

struct PartialArrayWriter<T, const N: usize> {
    array: [MaybeUninit<T>; N],
    init: usize,
}

impl<T, const N: usize> PartialArrayWriter<T, N> {
    fn new() -> Self {
        // Safety: Is totally safe, as the individual entries are still `MaybeUninit`.
        let array = unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() };
        Self { array, init: 0 }
    }

    fn push(&mut self, value: T) {
        self.array[self.init].write(value);
        self.init += 1;
    }

    fn unwrap(self) -> [T; N] {
        if self.init != N {
            panic!("the array has not been fully initialized")
        }

        let this = ManuallyDrop::new(self);

        // Safety: We know that the read is valid, as it was properly initialized.
        let array = unsafe { std::ptr::read(&this.array) };

        // Safety all entries have been initialized.
        unsafe { array.map(|x| x.assume_init()) }
    }
}

impl<T, const N: usize> Drop for PartialArrayWriter<T, N> {
    fn drop(&mut self) {
        for i in 0..self.init {
            // Safety: We know that the values `0..self.init` are initialized.
            unsafe {
                self.array[i].assume_init_drop();
            }
        }
    }
}
