use crate::lerp::{InverseLerp, Lerp};
use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

pub use cartesian::{CartesianLength, CartesianOffset, CartesianPosition};

pub trait CoordinateSystemTransformer<T: CoordinateSystem, U: CoordinateSystem> {
    /// Applies a coordinate system transform to a position.
    fn transform_position(&self, position: T::Position) -> U::Position;

    /// Applies a coordinate system transform to an offset.
    fn transform_offset(&self, offset: T::Offset) -> U::Offset;
}

/// Definition of a coordinate system.
pub trait CoordinateSystem {
    type Position: PositionType<Self::Offset, Self::Length>;
    type Offset: OffsetType<Self::Position, Self::Length>;
    type Length: LengthType<Self::Position, Self::Offset>;
}

/// Generalization of a position.
pub trait PositionType<Offset: OffsetType<Self, Length>, Length: LengthType<Self, Offset>>:
    Sized
    + Copy
    + ComponentAccessible
    + Lerp
    + Sub<Output = Offset>
    + Add<Offset, Output = Self>
    + AddAssign<Offset>
    + Sub<Offset, Output = Self>
    + SubAssign<Offset>
    + PartialEq
{
}

/// Generalization of an offset.
pub trait OffsetType<Position: PositionType<Self, Length>, Length: LengthType<Position, Self>>:
    Sized
    + Copy
    + ComponentAccessible
    + Lerp
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<Length, Output = Self>
    + MulAssign<Length>
    + Div<Length, Output = Self>
    + DivAssign<Length>
    + Add<Position, Output = Position>
    + Sub<Position, Output = Position>
    + PartialEq
{
}

/// Generalization of a length.
pub trait LengthType<Position: PositionType<Offset, Self>, Offset: OffsetType<Position, Self>>:
    Sized
    + Copy
    + Lerp
    + InverseLerp
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<Output = Self>
    + MulAssign
    + Div<Output = Self>
    + DivAssign
    + Mul<Offset, Output = Offset>
    + From<Offset>
    + PartialEq
    + PartialOrd
{
    /// Zero length.
    const ZERO: Self;

    /// Unit length.
    const UNIT: Self;
}

/// Types that allow accessing their individual components.
pub trait ComponentAccessible {
    /// Component type.
    type Component: Copy;

    /// Number of components.
    const COMPONENTS: usize;

    /// Zero constant.
    const ZERO: Self;

    /// Unit offset at the specified component.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is bigger or equal to [`ComponentAccessible::COMPONENTS`].
    fn unit_component(idx: usize) -> Self;

    /// Returns the component at `idx`.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is bigger or equal to [`ComponentAccessible::COMPONENTS`].
    fn get_component(&self, idx: usize) -> Self::Component;

    /// Sets the component at `idx`.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is bigger or equal to [`ComponentAccessible::COMPONENTS`].
    fn set_component(&mut self, idx: usize, component: Self::Component);

    /// Gets the ordering of the components at `idx`.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is bigger or equal to [`ComponentAccessible::COMPONENTS`].
    fn cmp_component(&self, rhs: &Self, idx: usize) -> Option<std::cmp::Ordering>;
}

/// A position in a coordinate system.
pub struct Position<T: CoordinateSystem> {
    value: T::Position,
}

impl<T: CoordinateSystem> Position<T> {
    /// Constructs a new position.
    pub fn new(position: impl Into<T::Position>) -> Self {
        Position {
            value: position.into(),
        }
    }

    /// Constructs the zero position.
    pub const fn zero() -> Self {
        Self::ZERO
    }

    /// Extracts the value representation of the position.
    pub fn extract<U>(self) -> U
    where
        U: From<T::Position>,
    {
        self.value.into()
    }

    /// Applies a coordinate system transform to the position.
    pub fn transform<U: CoordinateSystem>(
        self,
        transformer: &impl CoordinateSystemTransformer<T, U>,
    ) -> Position<U> {
        Position::new(transformer.transform_position(*self))
    }
}

impl<T: CoordinateSystem> Debug for Position<T>
where
    T::Position: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Position")
            .field("value", &self.value)
            .finish()
    }
}

impl<T: CoordinateSystem> Copy for Position<T> where T::Position: Copy {}

impl<T: CoordinateSystem> Clone for Position<T>
where
    T::Position: Clone,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: CoordinateSystem> PartialEq for Position<T>
where
    T::Position: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: CoordinateSystem> Lerp for Position<T> {
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            value: self.value.lerp(other.value, t),
        }
    }
}

impl<T: CoordinateSystem> Sub for Position<T> {
    type Output = Offset<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Offset::new(*self - *rhs)
    }
}

impl<T: CoordinateSystem> Add<Offset<T>> for Position<T> {
    type Output = Self;

    fn add(self, rhs: Offset<T>) -> Self::Output {
        Self::new(*self + *rhs)
    }
}

impl<T: CoordinateSystem> AddAssign<Offset<T>> for Position<T> {
    fn add_assign(&mut self, rhs: Offset<T>) {
        **self += *rhs;
    }
}

impl<T: CoordinateSystem> Sub<Offset<T>> for Position<T> {
    type Output = Self;

    fn sub(self, rhs: Offset<T>) -> Self::Output {
        Self::new(*self - *rhs)
    }
}

impl<T: CoordinateSystem> SubAssign<Offset<T>> for Position<T> {
    fn sub_assign(&mut self, rhs: Offset<T>) {
        **self -= *rhs;
    }
}

impl<T: CoordinateSystem> Deref for Position<T> {
    type Target = T::Position;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: CoordinateSystem> DerefMut for Position<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: CoordinateSystem> ComponentAccessible for Position<T> {
    type Component = <T::Position as ComponentAccessible>::Component;
    const COMPONENTS: usize = <T::Position as ComponentAccessible>::COMPONENTS;
    const ZERO: Self = Self {
        value: <T::Position as ComponentAccessible>::ZERO,
    };

    fn unit_component(idx: usize) -> Self {
        Self::new(T::Position::unit_component(idx))
    }

    fn get_component(&self, idx: usize) -> Self::Component {
        T::Position::get_component(&self.value, idx)
    }

    fn set_component(&mut self, idx: usize, component: Self::Component) {
        T::Position::set_component(&mut self.value, idx, component)
    }

    fn cmp_component(&self, rhs: &Self, idx: usize) -> Option<std::cmp::Ordering> {
        T::Position::cmp_component(&self.value, &rhs.value, idx)
    }
}

/// An offset in a coordinate system.
pub struct Offset<T: CoordinateSystem> {
    value: T::Offset,
}

impl<T: CoordinateSystem> Offset<T> {
    /// Constructs a new offset.
    pub fn new(offset: impl Into<T::Offset>) -> Self {
        Offset {
            value: offset.into(),
        }
    }

    /// Constructs the zero offset.
    pub const fn zero() -> Self {
        Self::ZERO
    }

    /// Constructs a new offset, where the value at axis `axis` is `length`.
    pub fn from_length_at_axis(axis: usize, length: Length<T>) -> Self {
        let value = <T::Offset as ComponentAccessible>::unit_component(axis);
        Offset { value } * length
    }

    /// Extracts the value representation of the offset.
    pub fn extract<U>(self) -> U
    where
        U: From<T::Offset>,
    {
        self.value.into()
    }

    /// Applies a coordinate system transform to the offset.
    pub fn transform<U: CoordinateSystem>(
        self,
        transformer: &impl CoordinateSystemTransformer<T, U>,
    ) -> Offset<U> {
        Offset::new(transformer.transform_offset(*self))
    }
}

impl<T: CoordinateSystem> Debug for Offset<T>
where
    T::Offset: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Offset")
            .field("value", &self.value)
            .finish()
    }
}

impl<T: CoordinateSystem> Copy for Offset<T> where T::Offset: Copy {}

impl<T: CoordinateSystem> Clone for Offset<T>
where
    T::Offset: Clone,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: CoordinateSystem> PartialEq for Offset<T>
where
    T::Offset: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: CoordinateSystem> Lerp for Offset<T> {
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            value: self.value.lerp(other.value, t),
        }
    }
}

impl<T: CoordinateSystem> Add for Offset<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(*self + *rhs)
    }
}

impl<T: CoordinateSystem> AddAssign for Offset<T> {
    fn add_assign(&mut self, rhs: Self) {
        **self += *rhs;
    }
}

impl<T: CoordinateSystem> Sub for Offset<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(*self - *rhs)
    }
}

impl<T: CoordinateSystem> SubAssign for Offset<T> {
    fn sub_assign(&mut self, rhs: Self) {
        **self -= *rhs;
    }
}

impl<T: CoordinateSystem> Mul<Length<T>> for Offset<T> {
    type Output = Self;

    fn mul(self, rhs: Length<T>) -> Self::Output {
        Self::new(*self * *rhs)
    }
}

impl<T: CoordinateSystem> MulAssign<Length<T>> for Offset<T> {
    fn mul_assign(&mut self, rhs: Length<T>) {
        **self *= *rhs;
    }
}

impl<T: CoordinateSystem> Div<Length<T>> for Offset<T> {
    type Output = Self;

    fn div(self, rhs: Length<T>) -> Self::Output {
        Self::new(*self / *rhs)
    }
}

impl<T: CoordinateSystem> DivAssign<Length<T>> for Offset<T> {
    fn div_assign(&mut self, rhs: Length<T>) {
        **self /= *rhs;
    }
}

impl<T: CoordinateSystem> Add<Position<T>> for Offset<T> {
    type Output = Position<T>;

    fn add(self, rhs: Position<T>) -> Self::Output {
        Position::new(*self + *rhs)
    }
}

impl<T: CoordinateSystem> Sub<Position<T>> for Offset<T> {
    type Output = Position<T>;

    fn sub(self, rhs: Position<T>) -> Self::Output {
        Position::new(*self - *rhs)
    }
}

impl<T: CoordinateSystem> Deref for Offset<T> {
    type Target = T::Offset;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: CoordinateSystem> DerefMut for Offset<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: CoordinateSystem> ComponentAccessible for Offset<T> {
    type Component = <T::Offset as ComponentAccessible>::Component;
    const COMPONENTS: usize = <T::Offset as ComponentAccessible>::COMPONENTS;
    const ZERO: Self = Self {
        value: <T::Offset as ComponentAccessible>::ZERO,
    };

    fn unit_component(idx: usize) -> Self {
        Self::new(T::Offset::unit_component(idx))
    }

    fn get_component(&self, idx: usize) -> Self::Component {
        T::Offset::get_component(&self.value, idx)
    }

    fn set_component(&mut self, idx: usize, component: Self::Component) {
        T::Offset::set_component(&mut self.value, idx, component)
    }

    fn cmp_component(&self, rhs: &Self, idx: usize) -> Option<std::cmp::Ordering> {
        T::Offset::cmp_component(&self.value, &rhs.value, idx)
    }
}

/// A length in a coordinate system.
pub struct Length<T: CoordinateSystem> {
    value: T::Length,
}

impl<T: CoordinateSystem> Length<T> {
    /// Constructs a new length.
    pub fn new(offset: impl Into<T::Length>) -> Self {
        Length {
            value: offset.into(),
        }
    }

    /// Constructs the zero length.
    pub fn zero() -> Self {
        Length::new(T::Length::ZERO)
    }

    /// Constructs the unit length.
    pub fn unit() -> Self {
        Length::new(T::Length::UNIT)
    }

    /// Extracts the value representation of the length.
    pub fn extract<U>(self) -> U
    where
        U: From<T::Length>,
    {
        self.value.into()
    }
}

impl<T: CoordinateSystem> Debug for Length<T>
where
    T::Length: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Length")
            .field("value", &self.value)
            .finish()
    }
}

impl<T: CoordinateSystem> Copy for Length<T> where T::Length: Copy {}

impl<T: CoordinateSystem> Clone for Length<T>
where
    T::Length: Clone,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: CoordinateSystem> PartialEq for Length<T>
where
    T::Length: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: CoordinateSystem> PartialOrd for Length<T>
where
    T::Length: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T: CoordinateSystem> Lerp for Length<T> {
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            value: self.value.lerp(other.value, t),
        }
    }
}

impl<T: CoordinateSystem> InverseLerp for Length<T> {
    fn inv_lerp(self, start: Self, end: Self) -> f32 {
        self.value.inv_lerp(start.value, end.value)
    }
}

impl<T: CoordinateSystem> Add for Length<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(*self + *rhs)
    }
}

impl<T: CoordinateSystem> Sub for Length<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(*self - *rhs)
    }
}

impl<T: CoordinateSystem> Mul for Length<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(*self * *rhs)
    }
}

impl<T: CoordinateSystem> Div for Length<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(*self / *rhs)
    }
}

impl<T: CoordinateSystem> Mul<Offset<T>> for Length<T> {
    type Output = Offset<T>;

    fn mul(self, rhs: Offset<T>) -> Self::Output {
        rhs * self
    }
}

impl<T: CoordinateSystem> From<Offset<T>> for Length<T> {
    fn from(value: Offset<T>) -> Self {
        Self::new(*value)
    }
}

impl<T: CoordinateSystem> Deref for Length<T> {
    type Target = T::Length;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: CoordinateSystem> DerefMut for Length<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

/// An axis-aligned bounding box.
#[derive(Clone, Copy, PartialEq)]
pub struct Aabb<T: CoordinateSystem> {
    start: Position<T>,
    end: Position<T>,
}

/// Relation between two bounding boxes.
pub enum AabbRelation {
    /// The two bounding boxes don't share any points.
    Disjoint,
    /// The two bounding boxes share all points among each other.
    Equal,
    /// The second bounding box contains some points of the first bounding box.
    Intersect,
    /// The second bounding box contains all the points of the first bounding box.
    Contained,
}

impl<T: CoordinateSystem> Aabb<T> {
    /// Constructs a new bounding box.
    pub fn new(start: Position<T>, end: Position<T>) -> Self {
        Self { start, end }
    }

    /// Applies a coordinate system transform to the bounding box.
    pub fn transform<U: CoordinateSystem>(
        self,
        transformer: &impl CoordinateSystemTransformer<T, U>,
    ) -> Aabb<U> {
        Aabb {
            start: self.start.transform(transformer),
            end: self.end.transform(transformer),
        }
    }

    /// Returns the size of the bounding box.
    pub fn size(&self) -> Offset<T> {
        self.end - self.start
    }

    /// Checks if the bounding box is degenerate.
    ///
    /// A degenerate bounding box is a bounding box, where at least one side
    /// has a length `<= 0`.
    pub fn is_degenerate(&self) -> bool {
        // Check start < end, for each component.
        for i in 0..Position::<T>::COMPONENTS {
            match self.start.cmp_component(&self.end, i) {
                None | Some(std::cmp::Ordering::Equal) | Some(std::cmp::Ordering::Greater) => {
                    return true
                }
                _ => {}
            }
        }

        false
    }

    /// Checks if a point is contained in the region inside the bounding box.
    pub fn contains_point(&self, p: &Position<T>) -> bool {
        // Check start <= p <= end, for each component.
        for i in 0..Position::<T>::COMPONENTS {
            match self.start.cmp_component(p, i) {
                Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {
                    match p.cmp_component(&self.end, i) {
                        Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {}
                        _ => return false,
                    }
                }
                _ => return false,
            }
        }

        true
    }

    /// Checks how the current bounding box stands, in relation to another
    /// bounding box.
    pub fn aabb_relation(&self, rhs: &Self) -> AabbRelation
    where
        Self: PartialEq,
    {
        if *self == *rhs {
            AabbRelation::Equal
        } else {
            let start_contained = rhs.contains_point(&self.start);
            let end_contained = rhs.contains_point(&self.end);

            if start_contained && end_contained {
                AabbRelation::Contained
            } else if start_contained || end_contained {
                AabbRelation::Intersect
            } else {
                AabbRelation::Disjoint
            }
        }
    }
}

impl<T: CoordinateSystem> Debug for Aabb<T>
where
    Position<T>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AaBb")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

/// The screen coordinate system.
///
/// Goes from `(0, height)` on the bottom left up to `(width, 0)` on the top right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScreenSpace;

impl CoordinateSystem for ScreenSpace {
    type Position = CartesianPosition<true>;
    type Offset = CartesianOffset;
    type Length = CartesianLength;
}

/// The view coordinate system.
///
/// Goes from `(0, 0)` on the bottom left up to `(width, height)` on the top right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewSpace;

impl CoordinateSystem for ViewSpace {
    type Position = CartesianPosition<false>;
    type Offset = CartesianOffset;
    type Length = CartesianLength;
}

/// The global coordinate system.
///
/// Goes from `(0, 0)` on the bottom left up to `(width, 1.0)` on the top right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorldSpace;

impl CoordinateSystem for WorldSpace {
    type Position = CartesianPosition<false>;
    type Offset = CartesianOffset;
    type Length = CartesianLength;
}

/// The local coordinate system.
///
/// Goes from `(0, 0)` on the bottom left up to `(width, 1.0)` on the top right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalSpace;

impl CoordinateSystem for LocalSpace {
    type Position = CartesianPosition<false>;
    type Offset = CartesianOffset;
    type Length = CartesianLength;
}

/// A type for transforming between screen and view space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenViewTransformer {
    max_y: f32,
}

impl ScreenViewTransformer {
    /// Constructs a new instance.
    pub fn new(height: f32) -> Self {
        Self {
            max_y: height - 1.0,
        }
    }
}

impl CoordinateSystemTransformer<ScreenSpace, ViewSpace> for ScreenViewTransformer {
    fn transform_position(
        &self,
        position: <ScreenSpace as CoordinateSystem>::Position,
    ) -> <ViewSpace as CoordinateSystem>::Position {
        CartesianPosition {
            x: position.x,
            y: self.max_y - position.y,
        }
    }

    fn transform_offset(
        &self,
        offset: <ScreenSpace as CoordinateSystem>::Offset,
    ) -> <ViewSpace as CoordinateSystem>::Offset {
        offset
    }
}

impl CoordinateSystemTransformer<ViewSpace, ScreenSpace> for ScreenViewTransformer {
    fn transform_position(
        &self,
        position: <ViewSpace as CoordinateSystem>::Position,
    ) -> <ScreenSpace as CoordinateSystem>::Position {
        CartesianPosition {
            x: position.x,
            y: self.max_y - position.y,
        }
    }

    fn transform_offset(
        &self,
        offset: <ViewSpace as CoordinateSystem>::Offset,
    ) -> <ScreenSpace as CoordinateSystem>::Offset {
        offset
    }
}

/// A type for transforming between view and world space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewWorldTransformer {
    view_max_y: f32,
    view_world_width_ratio: f32,
}

impl ViewWorldTransformer {
    /// Constructs a new instance.
    pub fn new(view_height: f32, view_width: f32, world_width: f32) -> Self {
        let view_world_width_ratio = (view_width - 1.0) / (world_width - 1.0);

        Self {
            view_max_y: view_height - 1.0,
            view_world_width_ratio,
        }
    }
}

impl CoordinateSystemTransformer<ViewSpace, WorldSpace> for ViewWorldTransformer {
    fn transform_position(
        &self,
        position: <ViewSpace as CoordinateSystem>::Position,
    ) -> <WorldSpace as CoordinateSystem>::Position {
        CartesianPosition {
            x: position.x / self.view_world_width_ratio,
            y: position.y / self.view_max_y,
        }
    }

    fn transform_offset(
        &self,
        offset: <ViewSpace as CoordinateSystem>::Offset,
    ) -> <WorldSpace as CoordinateSystem>::Offset {
        CartesianOffset {
            x: offset.x / self.view_world_width_ratio,
            y: offset.y / self.view_max_y,
        }
    }
}

impl CoordinateSystemTransformer<WorldSpace, ViewSpace> for ViewWorldTransformer {
    fn transform_position(
        &self,
        position: <WorldSpace as CoordinateSystem>::Position,
    ) -> <ViewSpace as CoordinateSystem>::Position {
        CartesianPosition {
            x: position.x * self.view_world_width_ratio,
            y: position.y * self.view_max_y,
        }
    }

    fn transform_offset(
        &self,
        offset: <WorldSpace as CoordinateSystem>::Offset,
    ) -> <ViewSpace as CoordinateSystem>::Offset {
        CartesianOffset {
            x: offset.x * self.view_world_width_ratio,
            y: offset.y * self.view_max_y,
        }
    }
}

/// A type for transforming between world and local space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldLocalTransformer {
    world_offset: f32,
    y_scaling: f32,
}

impl WorldLocalTransformer {
    /// Constructs a new instance.
    pub fn new(world_x_offset: f32, local_y_scaling: f32) -> Self {
        Self {
            world_offset: world_x_offset,
            y_scaling: local_y_scaling,
        }
    }
}

impl CoordinateSystemTransformer<WorldSpace, LocalSpace> for WorldLocalTransformer {
    fn transform_position(
        &self,
        mut position: <WorldSpace as CoordinateSystem>::Position,
    ) -> <LocalSpace as CoordinateSystem>::Position {
        let local_start = (1.0 - self.y_scaling) / 2.0;
        let local_end = 1.0 - ((1.0 - self.y_scaling) / 2.0);

        position.x -= self.world_offset;
        position.y = position.y.inv_lerp(local_start, local_end);
        position
    }

    fn transform_offset(
        &self,
        offset: <WorldSpace as CoordinateSystem>::Offset,
    ) -> <LocalSpace as CoordinateSystem>::Offset {
        offset / CartesianLength(self.y_scaling)
    }
}

impl CoordinateSystemTransformer<LocalSpace, WorldSpace> for WorldLocalTransformer {
    fn transform_position(
        &self,
        mut position: <LocalSpace as CoordinateSystem>::Position,
    ) -> <WorldSpace as CoordinateSystem>::Position {
        position.x += self.world_offset;
        position.y = (1.0 - self.y_scaling).lerp(self.y_scaling, position.y);
        position
    }

    fn transform_offset(
        &self,
        offset: <LocalSpace as CoordinateSystem>::Offset,
    ) -> <WorldSpace as CoordinateSystem>::Offset {
        offset * CartesianLength(self.y_scaling)
    }
}

mod cartesian {
    use crate::lerp::{InverseLerp, Lerp};

    use super::{ComponentAccessible, LengthType, OffsetType, PositionType};
    use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

    /// Position in the 2d cartesian coordinate system.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct CartesianPosition<const INVERSE_Y: bool> {
        pub x: f32,
        pub y: f32,
    }

    impl<const INVERSE_Y: bool> From<(f32, f32)> for CartesianPosition<INVERSE_Y> {
        fn from((x, y): (f32, f32)) -> Self {
            Self { x, y }
        }
    }

    impl<const INVERSE_Y: bool> From<CartesianPosition<INVERSE_Y>> for (f32, f32) {
        fn from(value: CartesianPosition<INVERSE_Y>) -> Self {
            (value.x, value.y)
        }
    }

    impl<const INVERSE_Y: bool> Lerp for CartesianPosition<INVERSE_Y> {
        fn lerp(self, other: Self, t: f32) -> Self {
            Self {
                x: self.x.lerp(other.x, t),
                y: self.y.lerp(other.y, t),
            }
        }
    }

    impl<const INVERSE_Y: bool> Sub for CartesianPosition<INVERSE_Y> {
        type Output = CartesianOffset;

        fn sub(self, rhs: Self) -> Self::Output {
            CartesianOffset {
                x: self.x - rhs.x,
                y: self.y - rhs.y,
            }
        }
    }

    impl<const INVERSE_Y: bool> Add<CartesianOffset> for CartesianPosition<INVERSE_Y> {
        type Output = Self;

        fn add(self, rhs: CartesianOffset) -> Self::Output {
            Self {
                x: self.x + rhs.x,
                y: self.y + rhs.y,
            }
        }
    }

    impl<const INVERSE_Y: bool> AddAssign<CartesianOffset> for CartesianPosition<INVERSE_Y> {
        fn add_assign(&mut self, rhs: CartesianOffset) {
            self.x += rhs.x;
            self.y += rhs.y;
        }
    }

    impl<const INVERSE_Y: bool> Sub<CartesianOffset> for CartesianPosition<INVERSE_Y> {
        type Output = Self;

        fn sub(self, rhs: CartesianOffset) -> Self::Output {
            Self {
                x: self.x - rhs.x,
                y: self.y - rhs.y,
            }
        }
    }

    impl<const INVERSE_Y: bool> SubAssign<CartesianOffset> for CartesianPosition<INVERSE_Y> {
        fn sub_assign(&mut self, rhs: CartesianOffset) {
            self.x -= rhs.x;
            self.y -= rhs.y;
        }
    }

    impl<const INVERSE_Y: bool> ComponentAccessible for CartesianPosition<INVERSE_Y> {
        type Component = f32;
        const COMPONENTS: usize = 2;
        const ZERO: Self = Self { x: 0.0, y: 0.0 };

        fn unit_component(idx: usize) -> Self {
            match idx {
                0 => Self { x: 1.0, y: 0.0 },
                1 => Self { x: 0.0, y: 1.0 },
                _ => panic!("index out of range"),
            }
        }

        fn get_component(&self, idx: usize) -> Self::Component {
            match idx {
                0 => self.x,
                1 => self.y,
                _ => panic!("index out of range"),
            }
        }

        fn set_component(&mut self, idx: usize, component: Self::Component) {
            match idx {
                0 => self.x = component,
                1 => self.y = component,
                _ => panic!("index out of range"),
            }
        }

        fn cmp_component(&self, rhs: &Self, idx: usize) -> Option<std::cmp::Ordering> {
            match idx {
                0 => self.x.partial_cmp(&rhs.x),
                1 => {
                    if INVERSE_Y {
                        rhs.y.partial_cmp(&self.y)
                    } else {
                        self.y.partial_cmp(&rhs.y)
                    }
                }
                _ => panic!("index out of range"),
            }
        }
    }

    impl<const INVERSE_Y: bool> PositionType<CartesianOffset, CartesianLength>
        for CartesianPosition<INVERSE_Y>
    {
    }

    /// Offset in the 2d cartesian coordinate system.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct CartesianOffset {
        pub x: f32,
        pub y: f32,
    }

    impl From<(f32, f32)> for CartesianOffset {
        fn from((x, y): (f32, f32)) -> Self {
            Self { x, y }
        }
    }

    impl From<CartesianOffset> for (f32, f32) {
        fn from(value: CartesianOffset) -> Self {
            (value.x, value.y)
        }
    }

    impl Lerp for CartesianOffset {
        fn lerp(self, other: Self, t: f32) -> Self {
            Self {
                x: self.x.lerp(other.x, t),
                y: self.y.lerp(other.y, t),
            }
        }
    }

    impl Add for CartesianOffset {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self {
                x: self.x + rhs.x,
                y: self.y + rhs.y,
            }
        }
    }

    impl AddAssign for CartesianOffset {
        fn add_assign(&mut self, rhs: Self) {
            self.x += rhs.x;
            self.y += rhs.y;
        }
    }

    impl Sub for CartesianOffset {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            Self {
                x: self.x - rhs.x,
                y: self.y - rhs.y,
            }
        }
    }

    impl SubAssign for CartesianOffset {
        fn sub_assign(&mut self, rhs: Self) {
            self.x -= rhs.x;
            self.y -= rhs.y;
        }
    }

    impl Mul<CartesianLength> for CartesianOffset {
        type Output = Self;

        fn mul(self, rhs: CartesianLength) -> Self::Output {
            Self {
                x: self.x * rhs.0,
                y: self.y * rhs.0,
            }
        }
    }

    impl MulAssign<CartesianLength> for CartesianOffset {
        fn mul_assign(&mut self, rhs: CartesianLength) {
            self.x *= rhs.0;
            self.y *= rhs.0;
        }
    }

    impl Div<CartesianLength> for CartesianOffset {
        type Output = Self;

        fn div(self, rhs: CartesianLength) -> Self::Output {
            Self {
                x: self.x / rhs.0,
                y: self.y / rhs.0,
            }
        }
    }

    impl DivAssign<CartesianLength> for CartesianOffset {
        fn div_assign(&mut self, rhs: CartesianLength) {
            self.x /= rhs.0;
            self.y /= rhs.0;
        }
    }

    impl<const INVERSE_Y: bool> Add<CartesianPosition<INVERSE_Y>> for CartesianOffset {
        type Output = CartesianPosition<INVERSE_Y>;

        fn add(self, rhs: CartesianPosition<INVERSE_Y>) -> Self::Output {
            CartesianPosition {
                x: self.x + rhs.x,
                y: self.y + rhs.y,
            }
        }
    }

    impl<const INVERSE_Y: bool> Sub<CartesianPosition<INVERSE_Y>> for CartesianOffset {
        type Output = CartesianPosition<INVERSE_Y>;

        fn sub(self, rhs: CartesianPosition<INVERSE_Y>) -> Self::Output {
            CartesianPosition {
                x: self.x - rhs.x,
                y: self.y - rhs.y,
            }
        }
    }

    impl ComponentAccessible for CartesianOffset {
        type Component = f32;
        const COMPONENTS: usize = 2;
        const ZERO: Self = Self { x: 0.0, y: 0.0 };

        fn unit_component(idx: usize) -> Self {
            match idx {
                0 => Self { x: 1.0, y: 0.0 },
                1 => Self { x: 0.0, y: 1.0 },
                _ => panic!("index out of range"),
            }
        }

        fn get_component(&self, idx: usize) -> Self::Component {
            match idx {
                0 => self.x,
                1 => self.y,
                _ => panic!("index out of range"),
            }
        }

        fn set_component(&mut self, idx: usize, component: Self::Component) {
            match idx {
                0 => self.x = component,
                1 => self.y = component,
                _ => panic!("index out of range"),
            }
        }

        fn cmp_component(&self, rhs: &Self, idx: usize) -> Option<std::cmp::Ordering> {
            match idx {
                0 => self.x.partial_cmp(&rhs.x),
                1 => self.y.partial_cmp(&rhs.y),
                _ => panic!("index out of range"),
            }
        }
    }

    impl<const INVERSE_Y: bool> OffsetType<CartesianPosition<INVERSE_Y>, CartesianLength>
        for CartesianOffset
    {
    }

    /// Length in the 2d cartesian coordinate system.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub struct CartesianLength(pub f32);

    impl From<f32> for CartesianLength {
        fn from(value: f32) -> Self {
            Self(value)
        }
    }

    impl From<CartesianLength> for f32 {
        fn from(value: CartesianLength) -> Self {
            value.0
        }
    }

    impl Lerp for CartesianLength {
        fn lerp(self, other: Self, t: f32) -> Self {
            Self(self.0.lerp(other.0, t))
        }
    }

    impl InverseLerp for CartesianLength {
        fn inv_lerp(self, start: Self, end: Self) -> f32 {
            self.0.inv_lerp(start.0, end.0)
        }
    }

    impl Add for CartesianLength {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self(self.0 + rhs.0)
        }
    }

    impl AddAssign for CartesianLength {
        fn add_assign(&mut self, rhs: Self) {
            self.0 += rhs.0
        }
    }

    impl Sub for CartesianLength {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            Self(self.0 - rhs.0)
        }
    }

    impl SubAssign for CartesianLength {
        fn sub_assign(&mut self, rhs: Self) {
            self.0 -= rhs.0;
        }
    }

    impl Mul for CartesianLength {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            Self(self.0 * rhs.0)
        }
    }

    impl MulAssign for CartesianLength {
        fn mul_assign(&mut self, rhs: Self) {
            self.0 *= rhs.0
        }
    }

    impl Div for CartesianLength {
        type Output = Self;

        fn div(self, rhs: Self) -> Self::Output {
            Self(self.0 / rhs.0)
        }
    }

    impl DivAssign for CartesianLength {
        fn div_assign(&mut self, rhs: Self) {
            self.0 /= rhs.0;
        }
    }

    impl Mul<CartesianOffset> for CartesianLength {
        type Output = CartesianOffset;

        fn mul(self, rhs: CartesianOffset) -> Self::Output {
            rhs * self
        }
    }

    impl From<CartesianOffset> for CartesianLength {
        fn from(value: CartesianOffset) -> Self {
            let CartesianOffset { x, y } = value;
            Self((x.powi(2) + y.powi(2)).sqrt())
        }
    }

    impl<const INVERSE_Y: bool> LengthType<CartesianPosition<INVERSE_Y>, CartesianOffset>
        for CartesianLength
    {
        const ZERO: Self = CartesianLength(0.0);
        const UNIT: Self = CartesianLength(1.0);
    }
}
