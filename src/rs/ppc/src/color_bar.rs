use std::rc::Rc;

use crate::{
    axis::Axis,
    coordinates::{Aabb, Length, Position, ScreenSpace, ScreenViewTransformer, ViewSpace},
};

const OUTER_PADDING_REM: f32 = 2.0;
const COLOR_BAR_PADDING_REM: f32 = 1.0;
const COLOR_BAR_WIDTH_REM: f32 = 2.5;

#[allow(clippy::type_complexity)]
pub struct ColorBar {
    visible: bool,
    label: Rc<str>,
    screen_size: (f32, f32),
    get_rem_length: Rc<dyn Fn(f32) -> Length<ViewSpace>>,
    get_text_length: Rc<dyn Fn(&str) -> (Length<ViewSpace>, Length<ViewSpace>)>,
}

impl ColorBar {
    #[allow(clippy::type_complexity)]
    pub fn new(
        width: f32,
        height: f32,
        get_rem_length: Rc<dyn Fn(f32) -> Length<ScreenSpace>>,
        get_text_length: Rc<dyn Fn(&str) -> (Length<ScreenSpace>, Length<ScreenSpace>)>,
    ) -> Self {
        let get_rem_length = Rc::new(move |rem| {
            let length = get_rem_length(rem);
            Length::new(length.0)
        });
        let get_text_length = Rc::new(move |text: &str| {
            let (width, height) = get_text_length(text);
            (Length::new(width.0), Length::new(height.0))
        });

        Self {
            visible: false,
            label: "".into(),
            screen_size: (width, height),
            get_rem_length,
            get_text_length,
        }
    }

    pub fn label(&self) -> Rc<str> {
        self.label.clone()
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn set_to_empty(&mut self) {
        self.label = "".into();
    }

    pub fn set_to_label_probability(&mut self, label: &str) {
        self.label = format!("Probability {label}").into();
    }

    pub fn set_to_axis(&mut self, axis: &Axis) {
        self.label = axis.label();
    }

    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.screen_size = (width, height);
    }

    pub fn label_position(&self) -> Position<ScreenSpace> {
        let outer_padding = (self.get_rem_length)(OUTER_PADDING_REM);
        let bar_padding = (self.get_rem_length)(COLOR_BAR_PADDING_REM);
        let color_bar_width = (self.get_rem_length)(COLOR_BAR_WIDTH_REM);
        let (label_width, label_height) = if self.label.is_empty() {
            (self.get_text_length)("empty")
        } else {
            (self.get_text_length)(&self.label)
        };

        let width = color_bar_width.0.max(label_width.0);
        let half_width = width / 2.0;
        let (screen_width, screen_height) = self.screen_size;

        let x = screen_width - outer_padding.0 - bar_padding.0 - half_width;
        let y = screen_height - outer_padding.0 - label_height.0;
        let position = Position::<ViewSpace>::new((x, y));
        position.transform(&ScreenViewTransformer::new(screen_height))
    }

    pub fn bounding_box(&self) -> Aabb<ScreenSpace> {
        let outer_padding = (self.get_rem_length)(OUTER_PADDING_REM);
        let bar_padding = (self.get_rem_length)(COLOR_BAR_PADDING_REM);
        let color_bar_width = (self.get_rem_length)(COLOR_BAR_WIDTH_REM);
        let (label_width, _) = if self.label.is_empty() {
            (self.get_text_length)("empty")
        } else {
            (self.get_text_length)(&self.label)
        };

        let width = color_bar_width.0.max(label_width.0);
        let (screen_width, screen_height) = self.screen_size;

        let start_x = screen_width
            - outer_padding.0
            - outer_padding.0
            - bar_padding.0
            - bar_padding.0
            - width;
        let start_y = 0.0;
        let start = Position::<ViewSpace>::new((start_x, start_y));

        let end_x = screen_width;
        let end_y = screen_height - 1.0;
        let end = Position::<ViewSpace>::new((end_x, end_y));

        let transformer = ScreenViewTransformer::new(screen_height);
        let start = start.transform(&transformer);
        let end = end.transform(&transformer);
        Aabb::new(start, end)
    }

    pub fn bar_viewport(&self, pixel_ratio: f32) -> ((f32, f32), (f32, f32)) {
        let outer_padding = (self.get_rem_length)(OUTER_PADDING_REM);
        let bar_padding = (self.get_rem_length)(COLOR_BAR_PADDING_REM);
        let color_bar_width = (self.get_rem_length)(COLOR_BAR_WIDTH_REM);
        let (label_width, label_height) = if self.label.is_empty() {
            (self.get_text_length)("empty")
        } else {
            (self.get_text_length)(&self.label)
        };

        let full_width = color_bar_width.0.max(label_width.0);
        let width = color_bar_width.0;

        let half_full_width = full_width / 2.0;
        let half_width = width / 2.0;

        let (screen_width, screen_height) = self.screen_size;

        let start_x = screen_width - outer_padding.0 - bar_padding.0 - half_full_width - half_width;
        let start_y = outer_padding.0;

        let end_y = screen_height - outer_padding.0 - label_height.0 - bar_padding.0;
        let height = end_y - start_y;

        let start = (
            (start_x * pixel_ratio).floor(),
            ((screen_height - end_y) * pixel_ratio).floor(),
        );
        let size = (
            (width * pixel_ratio).floor(),
            (height * pixel_ratio).floor(),
        );
        (start, size)
    }
}
