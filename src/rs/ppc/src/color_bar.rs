use std::rc::Rc;

use crate::{
    axis::Axis,
    coordinates::{Aabb, Length, Position, ScreenSpace, ScreenViewTransformer, ViewSpace},
};

const OUTER_PADDING_REM: f32 = 2.0;
const TICKS_PADDING_REM: f32 = 0.0;
const COLOR_BAR_PADDING_REM: f32 = 0.5;
const COLOR_BAR_WIDTH_REM: f32 = 2.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColorBarColorMode {
    Color,
    Probability,
}

#[allow(clippy::type_complexity)]
pub struct ColorBar {
    visible: bool,
    color_mode: ColorBarColorMode,
    label: Rc<str>,
    screen_size: (f32, f32),
    ticks: Vec<(f32, Rc<str>)>,
    max_ticks_width: Length<ViewSpace>,
    max_ticks_height: Length<ViewSpace>,
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
        let get_text_length = Rc::new(
            move |text: &str| -> (Length<ViewSpace>, Length<ViewSpace>) {
                let (width, height) = get_text_length(text);
                (Length::new(width.0), Length::new(height.0))
            },
        );

        let ticks = default_ticks();
        let max_ticks_width = ticks
            .iter()
            .map(|(_, tick)| get_text_length(tick).0)
            .max_by(|&l, &r| l.0.total_cmp(&r.0))
            .unwrap_or(Length::new(0.0));
        let max_ticks_height = ticks
            .iter()
            .map(|(_, tick)| get_text_length(tick).1)
            .max_by(|&l, &r| l.0.total_cmp(&r.0))
            .unwrap_or(Length::new(0.0));

        Self {
            visible: false,
            label: "".into(),
            color_mode: ColorBarColorMode::Color,
            screen_size: (width, height),
            ticks,
            max_ticks_width,
            max_ticks_height,
            get_rem_length,
            get_text_length,
        }
    }

    pub fn color_mode(&self) -> ColorBarColorMode {
        self.color_mode
    }

    pub fn label(&self) -> Rc<str> {
        self.label.clone()
    }

    pub fn ticks(&self) -> &[(f32, Rc<str>)] {
        &self.ticks
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn set_to_empty(&mut self) {
        self.label = "".into();
        self.color_mode = ColorBarColorMode::Color;
        self.ticks = default_ticks();
        self.max_ticks_width = self
            .ticks
            .iter()
            .map(|(_, tick)| (self.get_text_length)(tick).0)
            .max_by(|&l, &r| l.0.total_cmp(&r.0))
            .unwrap_or(Length::new(0.0));
        self.max_ticks_height = self
            .ticks
            .iter()
            .map(|(_, tick)| (self.get_text_length)(tick).1)
            .max_by(|&l, &r| l.0.total_cmp(&r.0))
            .unwrap_or(Length::new(0.0));
    }

    pub fn set_to_label_probability(&mut self, label: &str) {
        self.label = format!("Probability {label}").into();
        self.color_mode = ColorBarColorMode::Probability;
        self.ticks = default_ticks();
        self.max_ticks_width = self
            .ticks
            .iter()
            .map(|(_, tick)| (self.get_text_length)(tick).0)
            .max_by(|&l, &r| l.0.total_cmp(&r.0))
            .unwrap_or(Length::new(0.0));
        self.max_ticks_height = self
            .ticks
            .iter()
            .map(|(_, tick)| (self.get_text_length)(tick).1)
            .max_by(|&l, &r| l.0.total_cmp(&r.0))
            .unwrap_or(Length::new(0.0));
    }

    pub fn set_to_axis(&mut self, axis: &Axis) {
        self.label = axis.label();
        self.color_mode = ColorBarColorMode::Color;
        self.ticks = axis.ticks().into();
        self.max_ticks_width = self
            .ticks
            .iter()
            .map(|(_, tick)| (self.get_text_length)(tick).0)
            .max_by(|&l, &r| l.0.total_cmp(&r.0))
            .unwrap_or(Length::new(0.0));
        self.max_ticks_height = self
            .ticks
            .iter()
            .map(|(_, tick)| (self.get_text_length)(tick).1)
            .max_by(|&l, &r| l.0.total_cmp(&r.0))
            .unwrap_or(Length::new(0.0));
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

    pub fn ticks_range(&self) -> (Position<ScreenSpace>, Position<ScreenSpace>) {
        let outer_padding = (self.get_rem_length)(OUTER_PADDING_REM);
        let ticks_padding = (self.get_rem_length)(TICKS_PADDING_REM);
        let bar_padding = (self.get_rem_length)(COLOR_BAR_PADDING_REM);
        let color_bar_width = (self.get_rem_length)(COLOR_BAR_WIDTH_REM);
        let (label_width, label_height) = if self.label.is_empty() {
            (self.get_text_length)("empty")
        } else {
            (self.get_text_length)(&self.label)
        };

        let width = color_bar_width.0.max(label_width.0);
        let (screen_width, screen_height) = self.screen_size;

        let start_x = screen_width
            - outer_padding.0
            - ticks_padding.0
            - bar_padding.0
            - bar_padding.0
            - width;

        let start_y = outer_padding.0 - (self.max_ticks_height.0 / 2.0);
        let end_y = screen_height
            - outer_padding.0
            - label_height.0
            - bar_padding.0
            - (self.max_ticks_height.0 / 2.0);

        let start = Position::<ViewSpace>::new((start_x, start_y));
        let end = Position::<ViewSpace>::new((start_x, end_y));

        let transformer = ScreenViewTransformer::new(screen_height);
        let start = start.transform(&transformer);
        let end = end.transform(&transformer);

        (start, end)
    }

    pub fn bounding_box(&self) -> Aabb<ScreenSpace> {
        let outer_padding = (self.get_rem_length)(OUTER_PADDING_REM);
        let ticks_padding = (self.get_rem_length)(TICKS_PADDING_REM);
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
            - ticks_padding.0
            - ticks_padding.0
            - bar_padding.0
            - bar_padding.0
            - self.max_ticks_width.0
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

fn default_ticks() -> Vec<(f32, Rc<str>)> {
    vec![
        (0.0, "0.0".into()),
        (0.1, "0.1".into()),
        (0.2, "0.2".into()),
        (0.3, "0.3".into()),
        (0.4, "0.4".into()),
        (0.5, "0.5".into()),
        (0.6, "0.6".into()),
        (0.7, "0.7".into()),
        (0.8, "0.8".into()),
        (0.9, "0.9".into()),
        (1.0, "1.0".into()),
    ]
}
