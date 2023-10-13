use std::{fmt::Debug, mem::MaybeUninit};

/// Marker for types that can be shared between the host and the gpu.
///
/// # Safety
///
/// See the wgsl specification to check if a type is host-sharable.
pub unsafe trait HostSharable: Copy {}

unsafe impl HostSharable for i32 {}
unsafe impl HostSharable for u32 {}
unsafe impl HostSharable for f32 {}
unsafe impl<T: HostSharable, const N: usize> HostSharable for [T; N] {}

/// Wrapper for an atomic type.
#[repr(C, align(4))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Atomic<T: atomic::AtomicScalar>(pub T);

unsafe impl HostSharable for Atomic<i32> {}
unsafe impl HostSharable for Atomic<u32> {}

mod atomic {
    pub trait AtomicScalar: Copy {}

    impl AtomicScalar for i32 {}
    impl AtomicScalar for u32 {}
}

/// A vector of two elements.
#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vec2<T: vector::VectorScalar>(pub [T; 2]);

unsafe impl HostSharable for Vec2<i32> {}
unsafe impl HostSharable for Vec2<u32> {}
unsafe impl HostSharable for Vec2<f32> {}

/// A vector of three elements.
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vec3<T: vector::VectorScalar>(pub [T; 3]);

unsafe impl HostSharable for Vec3<i32> {}
unsafe impl HostSharable for Vec3<u32> {}
unsafe impl HostSharable for Vec3<f32> {}

/// A vector of four elements.
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vec4<T: vector::VectorScalar>(pub [T; 4]);

unsafe impl HostSharable for Vec4<i32> {}
unsafe impl HostSharable for Vec4<u32> {}
unsafe impl HostSharable for Vec4<f32> {}

mod vector {
    pub trait VectorScalar: Copy {}

    impl VectorScalar for i32 {}
    impl VectorScalar for u32 {}
    impl VectorScalar for f32 {}

    pub trait VectorType<const N: usize> {
        type Type<T: VectorScalar>;

        fn unpack<T: VectorScalar>(value: Self::Type<T>) -> [T; N];

        fn pack<T: VectorScalar>(value: [T; N]) -> Self::Type<T>;
    }

    impl VectorType<2> for () {
        type Type<T: VectorScalar> = super::Vec2<T>;

        fn unpack<T: VectorScalar>(value: Self::Type<T>) -> [T; 2] {
            value.0
        }

        fn pack<T: VectorScalar>(value: [T; 2]) -> Self::Type<T> {
            super::Vec2(value)
        }
    }

    impl VectorType<3> for () {
        type Type<T: VectorScalar> = super::Vec3<T>;

        fn unpack<T: VectorScalar>(value: Self::Type<T>) -> [T; 3] {
            value.0
        }

        fn pack<T: VectorScalar>(value: [T; 3]) -> Self::Type<T> {
            super::Vec3(value)
        }
    }

    impl VectorType<4> for () {
        type Type<T: VectorScalar> = super::Vec4<T>;

        fn unpack<T: VectorScalar>(value: Self::Type<T>) -> [T; 4] {
            value.0
        }

        fn pack<T: VectorScalar>(value: [T; 4]) -> Self::Type<T> {
            super::Vec4(value)
        }
    }
}

/// A matrix type.
#[repr(C)]
pub struct Matrix<T: matrix::MatrixScalar, const C: usize, const R: usize>
where
    (): vector::VectorType<R>,
    (): vector::VectorType<C>,
{
    data: [<() as vector::VectorType<R>>::Type<T>; C],
}

impl<T, const C: usize, const R: usize> Matrix<T, C, R>
where
    T: matrix::MatrixScalar,
    (): vector::VectorType<R>,
    (): vector::VectorType<C>,
{
    /// Instantiates a new Matrix.
    pub fn new(data: [<() as vector::VectorType<R>>::Type<T>; C]) -> Self {
        Self { data }
    }

    /// Instantiates a new matrix from the given row vectors.
    pub fn from_rows(rows: [<() as vector::VectorType<C>>::Type<T>; R]) -> Self {
        let rows = rows.map(<() as vector::VectorType<C>>::unpack);
        Self::from_rows_array(rows)
    }

    /// Instantiates a new matrix from the given row arrays.
    pub fn from_rows_array(rows: [[T; C]; R]) -> Self {
        let mut columns = [[MaybeUninit::<T>::uninit(); R]; C];
        for (r, row) in rows.iter().enumerate() {
            for (c, v) in row.iter().enumerate() {
                columns[c][r] = MaybeUninit::new(*v);
            }
        }

        // Safety: We initialized every element.
        let columns = unsafe { std::mem::transmute_copy::<_, [[T; R]; C]>(&columns) };
        Self::from_columns_array(columns)
    }

    /// Instantiates a new matrix from the given column vectors.
    pub fn from_columns(columns: [<() as vector::VectorType<R>>::Type<T>; C]) -> Self {
        Self::new(columns)
    }

    /// Instantiates a new matrix from the given column vectors.
    pub fn from_columns_array(columns: [[T; R]; C]) -> Self {
        let columns = columns.map(<() as vector::VectorType<R>>::pack);
        Self::from_columns(columns)
    }
}

impl<T, const C: usize, const R: usize> Debug for Matrix<T, C, R>
where
    T: matrix::MatrixScalar,
    (): vector::VectorType<R>,
    (): vector::VectorType<C>,
    <() as vector::VectorType<R>>::Type<T>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Matrix").field("data", &self.data).finish()
    }
}

impl<T, const C: usize, const R: usize> Clone for Matrix<T, C, R>
where
    T: matrix::MatrixScalar,
    (): vector::VectorType<R>,
    (): vector::VectorType<C>,
    <() as vector::VectorType<R>>::Type<T>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<T, const C: usize, const R: usize> Copy for Matrix<T, C, R>
where
    T: matrix::MatrixScalar,
    (): vector::VectorType<R>,
    (): vector::VectorType<C>,
    <() as vector::VectorType<R>>::Type<T>: Copy,
{
}

impl<T, const C: usize, const R: usize> PartialEq for Matrix<T, C, R>
where
    T: matrix::MatrixScalar,
    (): vector::VectorType<R>,
    (): vector::VectorType<C>,
    <() as vector::VectorType<R>>::Type<T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T, const C: usize, const R: usize> Eq for Matrix<T, C, R>
where
    T: matrix::MatrixScalar,
    (): vector::VectorType<R>,
    (): vector::VectorType<C>,
    <() as vector::VectorType<R>>::Type<T>: Eq,
{
}

impl<T, const C: usize, const R: usize> PartialOrd for Matrix<T, C, R>
where
    T: matrix::MatrixScalar,
    (): vector::VectorType<R>,
    (): vector::VectorType<C>,
    <() as vector::VectorType<R>>::Type<T>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

impl<T, const C: usize, const R: usize> Ord for Matrix<T, C, R>
where
    T: matrix::MatrixScalar,
    (): vector::VectorType<R>,
    (): vector::VectorType<C>,
    <() as vector::VectorType<R>>::Type<T>: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}

/// A 2x2 Matrix.
pub type Matrix2x2<T> = Matrix<T, 2, 2>;

/// A 2x3 Matrix.
pub type Matrix2x3<T> = Matrix<T, 2, 3>;

/// A 2x4 Matrix.
pub type Matrix2x4<T> = Matrix<T, 2, 4>;

/// A 3x2 Matrix.
pub type Matrix3x2<T> = Matrix<T, 3, 2>;

/// A 3x3 Matrix.
pub type Matrix3x3<T> = Matrix<T, 3, 3>;

/// A 3x4 Matrix.
pub type Matrix3x4<T> = Matrix<T, 3, 4>;

/// A 4x2 Matrix.
pub type Matrix4x2<T> = Matrix<T, 4, 2>;

/// A 4x3 Matrix.
pub type Matrix4x3<T> = Matrix<T, 4, 3>;

/// A 4x4 Matrix.
pub type Matrix4x4<T> = Matrix<T, 4, 4>;

unsafe impl HostSharable for Matrix2x2<f32> {}
unsafe impl HostSharable for Matrix2x3<f32> {}
unsafe impl HostSharable for Matrix2x4<f32> {}

unsafe impl HostSharable for Matrix3x2<f32> {}
unsafe impl HostSharable for Matrix3x3<f32> {}
unsafe impl HostSharable for Matrix3x4<f32> {}

unsafe impl HostSharable for Matrix4x2<f32> {}
unsafe impl HostSharable for Matrix4x3<f32> {}
unsafe impl HostSharable for Matrix4x4<f32> {}

mod matrix {
    pub trait MatrixScalar: super::vector::VectorScalar {}

    impl MatrixScalar for f32 {}
}
