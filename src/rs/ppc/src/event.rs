use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event(u32);

#[allow(dead_code)]
impl Event {
    pub const NONE: Self = Self(0);

    // External events
    pub const RESIZE: Self = Self(1 << 0);
    pub const DATA_UPDATE: Self = Self(1 << 1);
    pub const BACKGROUND_COLOR_CHANGE: Self = Self(1 << 2);
    pub const BRUSH_COLOR_CHANGE: Self = Self(1 << 3);
    pub const UNSELECTED_COLOR_CHANGE: Self = Self(1 << 4);
    pub const COLOR_SCALE_CHANGE: Self = Self(1 << 5);
    pub const DATA_COLOR_MODE_CHANGE: Self = Self(1 << 6);
    pub const COLOR_BAR_VISIBILITY_CHANGE: Self = Self(1 << 7);
    pub const LABEL_ADDITION: Self = Self(1 << 8);
    pub const LABEL_REMOVAL: Self = Self(1 << 9);
    pub const ACTIVE_LABEL_CHANGE: Self = Self(1 << 10);
    pub const LABEL_COLOR_CHANGE: Self = Self(1 << 11);
    pub const LABEL_SELECTION_BOUNDS_CHANGE: Self = Self(1 << 12);
    pub const LABEL_EASING_CHANGE: Self = Self(1 << 13);
    pub const DEBUG_OPTIONS_CHANGE: Self = Self(1 << 14);

    // Internal events
    pub const AXIS_STATE_CHANGE: Self = Self(1 << 20);
    pub const AXIS_POSITION_CHANGE: Self = Self(1 << 21);
    pub const AXIS_ORDER_CHANGE: Self = Self(1 << 22);
    pub const SELECTIONS_CHANGE: Self = Self(1 << 23);

    pub fn is_empty(&self) -> bool {
        *self == Self::NONE
    }

    pub fn has_events(&self) -> bool {
        *self != Self::NONE
    }

    pub fn clear(&mut self) -> Self {
        let e = *self;
        *self = Self::NONE;
        e
    }

    pub fn signal(&mut self, event: Self) {
        *self |= event;
    }

    pub fn signal_many(&mut self, events: &[Self]) {
        let events = events.iter().fold(Self::NONE, |acc, &e| acc | e);
        *self |= events;
    }

    pub fn signaled(&self, event: Self) -> bool {
        (*self & event).has_events()
    }

    pub fn signaled_any(&self, events: &[Self]) -> bool {
        events.iter().copied().any(|e| (*self & e).has_events())
    }

    pub fn signaled_all(&self, events: &[Self]) -> bool {
        events.iter().copied().all(|e| (*self & e).has_events())
    }
}

impl BitAnd for Event {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Event {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Event {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Event {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor for Event {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Event {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Not for Event {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
