use std::{borrow::Cow, collections::BTreeMap};

use once_cell::sync::OnceCell;

use crate::colors::{
    CieLab, CieLch, Color, ColorQuery, ColorSpace, ColorSpaceTransform, ColorTransparent, SRgb, Xyz,
};
use crate::lerp::Lerp;

/// A descriptor for how to construct a color scale.
pub enum ColorScaleDescriptor<'a> {
    Named(Cow<'a, str>),
    Constant(ColorQuery<'a>),
    Gradient(Vec<(f32, ColorQuery<'a>)>),
}

impl ColorScaleDescriptor<'_> {
    pub fn to_color_scale<T>(&self) -> ColorScale<T>
    where
        T: ColorSpace,
        SRgb: ColorSpaceTransform<T>,
        Xyz: ColorSpaceTransform<T>,
        CieLab: ColorSpaceTransform<T>,
        CieLch: ColorSpaceTransform<T>,
    {
        match self {
            ColorScaleDescriptor::Named(name) => {
                let scales = Self::get_named_color_scales();
                match scales.get(&**name) {
                    Some(descriptor) => descriptor.to_color_scale(),
                    None => panic!("named color scale {name:?} does not exist"),
                }
            }
            ColorScaleDescriptor::Constant(constant) => {
                let constant = constant.resolve_with_alpha::<T>();
                ColorScale {
                    scale: vec![(0.0, constant), (1.0, constant)],
                }
            }
            ColorScaleDescriptor::Gradient(gradient) => {
                let gradient = gradient
                    .iter()
                    .map(|(t, query)| (*t, query.resolve_with_alpha::<T>()))
                    .collect::<Vec<_>>();

                if gradient.windows(2).any(|w| w[0].0 >= w[1].0) {
                    panic!("The provided gradient is not sorted in strictly ascending order");
                }

                ColorScale { scale: gradient }
            }
        }
    }

    fn get_named_color_scales() -> &'static BTreeMap<String, ColorScaleDescriptor<'static>> {
        static NAMED_SCALES: OnceCell<BTreeMap<String, ColorScaleDescriptor<'static>>> =
            OnceCell::new();

        NAMED_SCALES.get_or_init(BTreeMap::new)
    }
}

/// A color scale that maps each value between `0` and `1` to a color value.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ColorScale<T: ColorSpace> {
    scale: Vec<(f32, ColorTransparent<T>)>,
}

impl<T: ColorSpace> ColorScale<T> {
    /// Returns the unsampled representation of the scale.
    ///
    /// The scale is sorted in strictly ascending order of the `t` value and is
    /// guaranteed to have at least the entries at `t=0.0` and `t=1.0`.
    pub fn get_scale(&self) -> &[(f32, ColorTransparent<T>)] {
        &self.scale
    }

    /// Samples the color scale at a specific `t` value.
    ///
    /// # Panics
    ///
    /// Panics if `t` is outside of the expected range of `[0.0, 1.0]`.
    pub fn sample(&self, t: f32) -> ColorTransparent<T> {
        if !(0.0..=1.0).contains(&t) {
            panic!("t value must lie between 0 and 1, t value: {t}")
        }

        let end_color_idx = self.scale.partition_point(|(x, _)| *x <= t);
        let start_color_idx = end_color_idx - 1;

        let (start_t, start_color) = self.scale[start_color_idx];
        if start_t == t {
            start_color
        } else {
            let (end_t, end_color) = self.scale[end_color_idx];
            let t = (t - start_t) / (end_t - start_t);

            let start_color = start_color.to_f32_with_alpha();
            let end_color = end_color.to_f32_with_alpha();
            let color = start_color.lerp(end_color, t);
            ColorTransparent::from_f32_with_alpha(color)
        }
    }
}
