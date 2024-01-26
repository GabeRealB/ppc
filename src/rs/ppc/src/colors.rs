// Color space conversions are adapted from `https://mina86.com/2021/srgb-lab-lchab-conversions/`.
#![allow(clippy::excessive_precision)]

use std::borrow::Cow;

use once_cell::sync::OnceCell;
use regex::Regex;

/// A trait for representing different color spaces.
pub trait ColorSpace: Clone + Copy {
    /// Returns the color represented as an array of floats.
    fn to_f32(self) -> [f32; 3];

    /// Constructs the color from an array of floats.
    fn from_f32(values: [f32; 3]) -> Self;
}

/// A trait for expressing the transformation from one color space to another.
pub trait ColorSpaceTransform<Output: ColorSpace>: ColorSpace {
    /// Transforms the color space of the values.
    fn transform(self) -> Output;
}

impl<T: ColorSpace> ColorSpaceTransform<T> for T {
    fn transform(self) -> T {
        self
    }
}

/// A generalization of a color.
pub trait Color<T: ColorSpace> {
    type Value<U: ColorSpace>;
    type OpaqueValue;
    type TransparentValue;

    /// Removes the alpha from the color.
    fn without_alpha(self) -> Self::OpaqueValue;

    /// Adds an alpha component to the color.
    fn with_alpha(self, alpha: f32) -> Self::TransparentValue;

    /// Transforms the color space of the color.
    fn transform<U: ColorSpace>(self) -> Self::Value<U>
    where
        T: ColorSpaceTransform<U>;

    /// Returns the color represented as an array of floats without the alpha component.
    fn to_f32(self) -> [f32; 3];

    /// Returns the color represented as an array of floats with the alpha component.
    fn to_f32_with_alpha(self) -> [f32; 4];

    /// Constructs the color from an array of `f32` without the alpha component.
    fn from_f32(values: [f32; 3]) -> Self;

    /// Constructs the color from an array of `f32` with the alpha component.
    fn from_f32_with_alpha(values: [f32; 4]) -> Self;
}

/// An opaque color of a color space.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ColorOpaque<T: ColorSpace> {
    pub values: T,
}

impl<T: ColorSpace> Color<T> for ColorOpaque<T> {
    type Value<U: ColorSpace> = ColorOpaque<U>;
    type OpaqueValue = Self;
    type TransparentValue = ColorTransparent<T>;

    fn without_alpha(self) -> Self::OpaqueValue {
        self
    }

    fn with_alpha(self, alpha: f32) -> Self::TransparentValue {
        ColorTransparent { color: self, alpha }
    }

    fn transform<U: ColorSpace>(self) -> Self::Value<U>
    where
        T: ColorSpaceTransform<U>,
    {
        let values = self.values.transform();
        ColorOpaque { values }
    }

    fn to_f32(self) -> [f32; 3] {
        self.values.to_f32()
    }

    fn to_f32_with_alpha(self) -> [f32; 4] {
        self.with_alpha(1.0).to_f32_with_alpha()
    }

    fn from_f32(values: [f32; 3]) -> Self {
        Self {
            values: T::from_f32(values),
        }
    }

    fn from_f32_with_alpha(values: [f32; 4]) -> Self {
        let values = [values[0], values[1], values[2]];
        Self::from_f32(values)
    }
}

/// A color of a color space with a transparency component.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ColorTransparent<T: ColorSpace> {
    pub color: ColorOpaque<T>,
    pub alpha: f32,
}

impl<T: ColorSpace> Color<T> for ColorTransparent<T> {
    type Value<U: ColorSpace> = ColorTransparent<U>;
    type OpaqueValue = ColorOpaque<T>;
    type TransparentValue = Self;

    fn without_alpha(self) -> Self::OpaqueValue {
        self.color
    }

    fn with_alpha(mut self, alpha: f32) -> Self::TransparentValue {
        self.alpha = alpha;
        self
    }

    fn transform<U: ColorSpace>(self) -> Self::Value<U>
    where
        T: ColorSpaceTransform<U>,
    {
        ColorTransparent {
            color: self.color.transform(),
            alpha: self.alpha,
        }
    }

    fn to_f32(self) -> [f32; 3] {
        self.color.to_f32()
    }

    fn to_f32_with_alpha(self) -> [f32; 4] {
        let normalized = self.color.to_f32();
        [normalized[0], normalized[1], normalized[2], self.alpha]
    }

    fn from_f32(values: [f32; 3]) -> Self {
        Self {
            color: ColorOpaque::from_f32(values),
            alpha: 1.0,
        }
    }

    fn from_f32_with_alpha(values: [f32; 4]) -> Self {
        Self {
            color: ColorOpaque::from_f32_with_alpha(values),
            alpha: values[3],
        }
    }
}

macro_rules! indirect_transform {
    ($T:ty, $U:ty, $Res:ty) => {
        impl ColorSpaceTransform<$Res> for $T {
            fn transform(self) -> $Res {
                let tmp: $U = self.transform();
                tmp.transform()
            }
        }
    };
}

macro_rules! to_unknown {
    ($T:ty) => {
        impl ColorSpaceTransform<UnknownColorSpace> for $T {
            fn transform(self) -> UnknownColorSpace {
                let values = <$T as ColorSpace>::to_f32(self);
                <UnknownColorSpace as ColorSpace>::from_f32(values)
            }
        }
    };
}

/// An unknown color space.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct UnknownColorSpace {
    pub values: [f32; 3],
}

impl ColorSpace for UnknownColorSpace {
    fn to_f32(self) -> [f32; 3] {
        self.values
    }

    fn from_f32(values: [f32; 3]) -> Self {
        Self { values }
    }
}

/// The sRGB color space with a D65 white point and 8-bit values.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorSpace for SRgb {
    fn to_f32(self) -> [f32; 3] {
        [
            (self.r as f32) / 255.0,
            (self.g as f32) / 255.0,
            (self.b as f32) / 255.0,
        ]
    }

    fn from_f32(values: [f32; 3]) -> Self {
        let [r, g, b] = values.map(|v| (v * 255.0).round() as u8);
        Self { r, g, b }
    }
}

impl ColorSpaceTransform<SRgbLinear> for SRgb {
    fn transform(self) -> SRgbLinear {
        fn f(v: u8) -> f32 {
            if v <= 10 {
                (v as f32) / 3294.6
            } else {
                let exp = ((v as f32) + 14.025) / 269.025;
                exp.powf(2.4)
            }
        }

        SRgbLinear {
            r: f(self.r),
            g: f(self.g),
            b: f(self.b),
        }
    }
}

to_unknown! {SRgb}
indirect_transform! {SRgb, SRgbLinear, Xyz}
indirect_transform! {SRgb, SRgbLinear, CieLab}
indirect_transform! {SRgb, SRgbLinear, CieLch}

/// The sRGB color space with a D65 white point and linear values in the range [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SRgbLinear {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl ColorSpace for SRgbLinear {
    fn to_f32(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }

    fn from_f32(values: [f32; 3]) -> Self {
        let [r, g, b] = values;
        Self { r, g, b }
    }
}

impl ColorSpaceTransform<SRgb> for SRgbLinear {
    fn transform(self) -> SRgb {
        // Gamma compress the channels.
        fn f(v: f32) -> u8 {
            let non_linear = if v <= 0.00313066844250060782371 {
                3294.6 * v
            } else {
                269.025 * v.powf(5.0 / 12.0) - 14.025
            };

            non_linear.round() as u8
        }

        SRgb {
            r: f(self.r),
            g: f(self.g),
            b: f(self.b),
        }
    }
}

impl ColorSpaceTransform<Xyz> for SRgbLinear {
    fn transform(self) -> Xyz {
        const SRGB_LINEAR_TO_XYZ_MATRIX: [[f32; 3]; 3] = [
            [0.4124108464885388, 0.3575845678529519, 0.18045380393360833],
            [0.21264934272065283, 0.7151691357059038, 0.07218152157344333],
            [
                0.019331758429150258,
                0.11919485595098397,
                0.9503900340503373,
            ],
        ];

        let row = self.to_f32();
        let xyz = matrix_multiply(SRGB_LINEAR_TO_XYZ_MATRIX, row);
        Xyz::from_f32(xyz)
    }
}

to_unknown! {SRgbLinear}
indirect_transform! {SRgbLinear, Xyz, CieLab}
indirect_transform! {SRgbLinear, Xyz, CieLch}

/// The XYZ color space with a D65 white point.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Xyz {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl ColorSpace for Xyz {
    fn to_f32(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    fn from_f32(values: [f32; 3]) -> Self {
        let [x, y, z] = values;
        Self { x, y, z }
    }
}

to_unknown! {Xyz}
indirect_transform! {Xyz, SRgbLinear, SRgb}

impl ColorSpaceTransform<SRgbLinear> for Xyz {
    fn transform(self) -> SRgbLinear {
        const XYZ_TO_SRGB_LINEAR_MATRIX: [[f32; 3]; 3] = [
            [3.240812398895283, -1.5373084456298136, -0.4985865229069666],
            [-0.9692430170086407, 1.8759663029085742, 0.04155503085668564],
            [
                0.055638398436112804,
                -0.20400746093241362,
                1.0571295702861434,
            ],
        ];

        let xyz: [f32; 3] = self.to_f32();
        let srgb = matrix_multiply(XYZ_TO_SRGB_LINEAR_MATRIX, xyz);
        SRgbLinear::from_f32(srgb)
    }
}

impl ColorSpaceTransform<CieLab> for Xyz {
    fn transform(self) -> CieLab {
        const EPSILON: f32 = 216.0 / 24389.0;
        const KAPPA: f32 = 24389.0 / 27.0;

        fn f(v: f32) -> f32 {
            if v > EPSILON {
                v.powf(1.0 / 3.0)
            } else {
                (KAPPA * v + 16.0) / 116.0
            }
        }

        let fx = f(self.x / 0.9504492182750991);
        let fy = f(self.y);
        let fz = f(self.z / 1.0889166484304715);

        let lab = [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)];
        CieLab::from_f32(lab)
    }
}

indirect_transform! {Xyz, CieLab, CieLch}

/// The CIE L*a*b color space with a D65 white point.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CieLab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

impl ColorSpace for CieLab {
    fn to_f32(self) -> [f32; 3] {
        [self.l, self.a, self.b]
    }

    fn from_f32(values: [f32; 3]) -> Self {
        let [l, a, b] = values;
        Self { l, a, b }
    }
}

to_unknown! {CieLab}
indirect_transform! {CieLab, Xyz, SRgb}
indirect_transform! {CieLab, Xyz, SRgbLinear}

impl ColorSpaceTransform<Xyz> for CieLab {
    fn transform(self) -> Xyz {
        const CBRT_EPSILON: f32 = 6.0 / 29.0;
        const KAPPA: f32 = 24389.0 / 27.0;

        fn f_inv(v: f32) -> f32 {
            if v > CBRT_EPSILON {
                v.powi(3)
            } else {
                (v * 116.0 - 16.0) / KAPPA
            }
        }

        let fy = (self.l + 16.0) / 116.0;
        let fx = (self.a / 500.0) + fy;
        let fz = fy - (self.b / 200.0);

        let x = f_inv(fx) * 0.9504492182750991;
        let y = if self.l > 8.0 {
            fy.powi(3)
        } else {
            self.b / KAPPA
        };
        let z = f_inv(fz) * 1.0889166484304715;

        Xyz::from_f32([x, y, z])
    }
}

impl ColorSpaceTransform<CieLch> for CieLab {
    fn transform(self) -> CieLch {
        let l = self.l;
        let c = self.l.hypot(self.a);
        let h = self.b.atan2(self.a) * 360.0 / std::f32::consts::TAU;

        CieLch::from_f32([l, c, h])
    }
}

/// The CIE LCH color space with a D65 white point and hue expressed in degrees.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CieLch {
    pub l: f32,
    pub c: f32,
    pub h: f32,
}

impl ColorSpace for CieLch {
    fn to_f32(self) -> [f32; 3] {
        [self.l, self.c, self.h]
    }

    fn from_f32(values: [f32; 3]) -> Self {
        let [l, c, h] = values;
        Self { l, c, h }
    }
}

to_unknown! {CieLch}
indirect_transform! {CieLch, CieLab, SRgb}
indirect_transform! {CieLch, CieLab, SRgbLinear}
indirect_transform! {CieLch, CieLab, Xyz}

impl ColorSpaceTransform<CieLab> for CieLch {
    fn transform(self) -> CieLab {
        let h_rad = self.h * std::f32::consts::TAU / 360.0;

        let l = self.l;
        let a = self.c * h_rad.cos();
        let b = self.c * h_rad.sin();

        CieLab::from_f32([l, a, b])
    }
}

fn matrix_multiply<const N: usize, const M: usize>(matrix: [[f32; N]; M], v: [f32; N]) -> [f32; M] {
    matrix.map(|row| row.into_iter().zip(v).map(|(a, b)| a * b).sum())
}

/// A color query.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ColorQuery<'a> {
    Named(Cow<'a, str>),
    Css(Cow<'a, str>),
    SRgb([f32; 3], Option<f32>),
    Xyz([f32; 3], Option<f32>),
    Lab([f32; 3], Option<f32>),
    Lch([f32; 3], Option<f32>),
}

impl ColorQuery<'_> {
    pub fn resolve<T>(&self) -> ColorOpaque<T>
    where
        T: ColorSpace,
        SRgb: ColorSpaceTransform<T>,
        Xyz: ColorSpaceTransform<T>,
        CieLab: ColorSpaceTransform<T>,
        CieLch: ColorSpaceTransform<T>,
    {
        self.resolve_with_alpha::<T>().without_alpha()
    }

    pub fn resolve_with_alpha<T>(&self) -> ColorTransparent<T>
    where
        T: ColorSpace,
        SRgb: ColorSpaceTransform<T>,
        Xyz: ColorSpaceTransform<T>,
        CieLab: ColorSpaceTransform<T>,
        CieLch: ColorSpaceTransform<T>,
    {
        match self {
            ColorQuery::Named(name) => Self::resolve_named(name)
                .expect("named color does not exist")
                .transform(),
            ColorQuery::Css(css) => Self::resolve_css(css),
            ColorQuery::SRgb(values, alpha) => {
                let values = <SRgb as ColorSpace>::from_f32(*values);
                let alpha = alpha.unwrap_or(1.0);
                ColorTransparent::<SRgb> {
                    color: ColorOpaque { values },
                    alpha,
                }
                .transform()
            }
            ColorQuery::Xyz(values, alpha) => {
                let values = <Xyz as ColorSpace>::from_f32(*values);
                let alpha = alpha.unwrap_or(1.0);
                ColorTransparent::<Xyz> {
                    color: ColorOpaque { values },
                    alpha,
                }
                .transform()
            }
            ColorQuery::Lab(values, alpha) => {
                let values = <CieLab as ColorSpace>::from_f32(*values);
                let alpha = alpha.unwrap_or(1.0);
                ColorTransparent::<CieLab> {
                    color: ColorOpaque { values },
                    alpha,
                }
                .transform()
            }
            ColorQuery::Lch(values, alpha) => {
                let values = <CieLch as ColorSpace>::from_f32(*values);
                let alpha = alpha.unwrap_or(1.0);
                ColorTransparent::<CieLch> {
                    color: ColorOpaque { values },
                    alpha,
                }
                .transform()
            }
        }
    }

    fn resolve_named(name: &str) -> Option<ColorTransparent<SRgb>> {
        let [r, g, b] = match name {
            "aliceblue" => [240, 248, 255],
            "antiquewhite" => [250, 235, 215],
            "aqua" => [0, 255, 255],
            "aquamarine" => [127, 255, 212],
            "azure" => [240, 255, 255],
            "beige" => [245, 245, 220],
            "bisque" => [255, 228, 196],
            "black" => [0, 0, 0],
            "blanchedalmond" => [255, 235, 205],
            "blue" => [0, 0, 255],
            "blueviolet" => [138, 43, 226],
            "brown" => [165, 42, 42],
            "burlywood" => [222, 184, 135],
            "cadetblue" => [95, 158, 160],
            "chartreuse" => [127, 255, 0],
            "chocolate" => [210, 105, 30],
            "coral" => [255, 127, 80],
            "cornflowerblue" => [100, 149, 237],
            "cornsilk" => [255, 248, 220],
            "crimson" => [220, 20, 60],
            "cyan" => [0, 255, 255],
            "darkblue" => [0, 0, 139],
            "darkcyan" => [0, 139, 139],
            "darkgoldenrod" => [184, 134, 11],
            "darkgray" => [169, 169, 169],
            "darkgreen" => [0, 100, 0],
            "darkgrey" => [169, 169, 169],
            "darkkhaki" => [189, 183, 107],
            "darkmagenta" => [139, 0, 139],
            "darkolivegreen" => [85, 107, 47],
            "darkorange" => [255, 140, 0],
            "darkorchid" => [153, 50, 204],
            "darkred" => [139, 0, 0],
            "darksalmon" => [233, 150, 122],
            "darkseagreen" => [143, 188, 143],
            "darkslateblue" => [72, 61, 139],
            "darkslategray" => [47, 79, 79],
            "darkslategrey" => [47, 79, 79],
            "darkturquoise" => [0, 206, 209],
            "darkviolet" => [148, 0, 211],
            "deeppink" => [255, 20, 147],
            "deepskyblue" => [0, 191, 255],
            "dimgray" => [105, 105, 105],
            "dimgrey" => [105, 105, 105],
            "dodgerblue" => [30, 144, 255],
            "firebrick" => [178, 34, 34],
            "floralwhite" => [255, 250, 240],
            "forestgreen" => [34, 139, 34],
            "fuchsia" => [255, 0, 255],
            "gainsboro" => [220, 220, 220],
            "ghostwhite" => [248, 248, 255],
            "gold" => [255, 215, 0],
            "goldenrod" => [218, 165, 32],
            "gray" => [128, 128, 128],
            "green" => [0, 128, 0],
            "greenyellow" => [173, 255, 47],
            "grey" => [128, 128, 128],
            "honeydew" => [240, 255, 240],
            "hotpink" => [255, 105, 180],
            "indianred" => [205, 92, 92],
            "indigo" => [75, 0, 130],
            "ivory" => [255, 255, 240],
            "khaki" => [240, 230, 140],
            "lavender" => [230, 230, 250],
            "lavenderblush" => [255, 240, 245],
            "lawngreen" => [124, 252, 0],
            "lemonchiffon" => [255, 250, 205],
            "lightblue" => [173, 216, 230],
            "lightcoral" => [240, 128, 128],
            "lightcyan" => [224, 255, 255],
            "lightgoldenrodyellow" => [250, 250, 210],
            "lightgray" => [211, 211, 211],
            "lightgreen" => [144, 238, 144],
            "lightgrey" => [211, 211, 211],
            "lightpink" => [255, 182, 193],
            "lightsalmon" => [255, 160, 122],
            "lightseagreen" => [32, 178, 170],
            "lightskyblue" => [135, 206, 250],
            "lightslategray" => [119, 136, 153],
            "lightslategrey" => [119, 136, 153],
            "lightsteelblue" => [176, 196, 222],
            "lightyellow" => [255, 255, 224],
            "lime" => [0, 255, 0],
            "limegreen" => [50, 205, 50],
            "linen" => [250, 240, 230],
            "magenta" => [255, 0, 255],
            "maroon" => [128, 0, 0],
            "mediumaquamarine" => [102, 205, 170],
            "mediumblue" => [0, 0, 205],
            "mediumorchid" => [186, 85, 211],
            "mediumpurple" => [147, 112, 219],
            "mediumseagreen" => [60, 179, 113],
            "mediumslateblue" => [123, 104, 238],
            "mediumspringgreen" => [0, 250, 154],
            "mediumturquoise" => [72, 209, 204],
            "mediumvioletred" => [199, 21, 133],
            "midnightblue" => [25, 25, 112],
            "mintcream" => [245, 255, 250],
            "mistyrose" => [255, 228, 225],
            "moccasin" => [255, 228, 181],
            "navajowhite" => [255, 222, 173],
            "navy" => [0, 0, 128],
            "oldlace" => [253, 245, 230],
            "olive" => [128, 128, 0],
            "olivedrab" => [107, 142, 35],
            "orange" => [255, 165, 0],
            "orangered" => [255, 69, 0],
            "orchid" => [218, 112, 214],
            "palegoldenrod" => [238, 232, 170],
            "palegreen" => [152, 251, 152],
            "paleturquoise" => [175, 238, 238],
            "palevioletred" => [219, 112, 147],
            "papayawhip" => [255, 239, 213],
            "peachpuff" => [255, 218, 185],
            "peru" => [205, 133, 63],
            "pink" => [255, 192, 203],
            "plum" => [221, 160, 221],
            "powderblue" => [176, 224, 230],
            "purple" => [128, 0, 128],
            "rebeccapurple" => [102, 51, 153],
            "red" => [255, 0, 0],
            "rosybrown" => [188, 143, 143],
            "royalblue" => [65, 105, 225],
            "saddlebrown" => [139, 69, 19],
            "salmon" => [250, 128, 114],
            "sandybrown" => [244, 164, 96],
            "seagreen" => [46, 139, 87],
            "seashell" => [255, 245, 238],
            "sienna" => [160, 82, 45],
            "silver" => [192, 192, 192],
            "skyblue" => [135, 206, 235],
            "slateblue" => [106, 90, 205],
            "slategray" => [112, 128, 144],
            "slategrey" => [112, 128, 144],
            "snow" => [255, 250, 250],
            "springgreen" => [0, 255, 127],
            "steelblue" => [70, 130, 180],
            "tan" => [210, 180, 140],
            "teal" => [0, 128, 128],
            "thistle" => [216, 191, 216],
            "tomato" => [255, 99, 71],
            "turquoise" => [64, 224, 208],
            "violet" => [238, 130, 238],
            "wheat" => [245, 222, 179],
            "white" => [255, 255, 255],
            "whitesmoke" => [245, 245, 245],
            "yellow" => [255, 255, 0],
            "yellowgreen" => [154, 205, 50],
            _ => return None,
        };

        Some(ColorTransparent {
            color: ColorOpaque {
                values: SRgb { r, g, b },
            },
            alpha: 1.0,
        })
    }

    fn resolve_rgb(rgba: &str) -> ColorTransparent<SRgb> {
        static MATCHER: OnceCell<Regex> = OnceCell::new();
        let matcher = MATCHER.get_or_init(|| Regex::new("rgb\\((?<R>((25[0-5])|(2[0-4][0-9]{1})|([0-1]?[0-9]{1,2}))) (?<G>((25[0-5])|(2[0-4][0-9]{1})|([0-1]?[0-9]{1,2}))) (?<B>((25[0-5])|(2[0-4][0-9]{1})|([0-1]?[0-9]{1,2})))( (?<A>[+-]?([0-9]*[.])?[0-9]+))?\\)").unwrap());
        let captures = matcher.captures(rgba).expect("invalid rgb string");

        let r = captures
            .name("R")
            .unwrap()
            .as_str()
            .parse::<u8>()
            .expect("expected a value between 0 and 255");
        let g = captures
            .name("G")
            .unwrap()
            .as_str()
            .parse::<u8>()
            .expect("expected a value between 0 and 255");
        let b = captures
            .name("B")
            .unwrap()
            .as_str()
            .parse::<u8>()
            .expect("expected a value between 0 and 255");
        let a = captures
            .name("A")
            .map(|m| m.as_str().parse::<f32>().expect("expected a float value"))
            .unwrap_or(1.0);
        if !(0.0..=1.0).contains(&a) {
            panic!("invalid alpha range");
        }

        ColorTransparent {
            color: ColorOpaque {
                values: SRgb { r, g, b },
            },
            alpha: a,
        }
    }

    fn resolve_xyz(xyz: &str) -> ColorTransparent<Xyz> {
        static MATCHER: OnceCell<Regex> = OnceCell::new();
        let matcher =
            MATCHER.get_or_init(|| Regex::new("xyz\\((?<X>[+-]?([0-9]*[.])?[0-9]+) (?<Y>[+-]?([0-9]*[.])?[0-9]+) (?<Z>[+-]?([0-9]*[.])?[0-9]+)( (?<A>[+-]?([0-9]*[.])?[0-9]+))?\\)").unwrap());
        let captures = matcher.captures(xyz).expect("invalid xyz string");

        let x = captures
            .name("X")
            .unwrap()
            .as_str()
            .parse::<f32>()
            .expect("expected a float value");
        let y = captures
            .name("Y")
            .unwrap()
            .as_str()
            .parse::<f32>()
            .expect("expected a float value");
        let z = captures
            .name("Z")
            .unwrap()
            .as_str()
            .parse::<f32>()
            .expect("expected a float value");
        let a = captures
            .name("A")
            .map(|m| m.as_str().parse::<f32>().expect("expected a float value"))
            .unwrap_or(1.0);
        if !(0.0..=1.0).contains(&a) {
            panic!("invalid alpha range");
        }

        ColorTransparent {
            color: ColorOpaque {
                values: Xyz::from_f32([x, y, z]),
            },
            alpha: a,
        }
    }

    fn resolve_lab(lab: &str) -> ColorTransparent<CieLab> {
        static MATCHER: OnceCell<Regex> = OnceCell::new();
        let matcher =
            MATCHER.get_or_init(|| Regex::new("lab\\((?<L>[+-]?([0-9]*[.])?[0-9]+) (?<a>[+-]?([0-9]*[.])?[0-9]+) (?<b>[+-]?([0-9]*[.])?[0-9]+)( (?<A>[+-]?([0-9]*[.])?[0-9]+))?\\)").unwrap());
        let captures = matcher.captures(lab).expect("invalid lab string");

        let l = captures
            .name("L")
            .unwrap()
            .as_str()
            .parse::<f32>()
            .expect("expected a float value");
        let a_star = captures
            .name("a")
            .unwrap()
            .as_str()
            .parse::<f32>()
            .expect("expected a float value");
        let b_star = captures
            .name("b")
            .unwrap()
            .as_str()
            .parse::<f32>()
            .expect("expected a float value");
        let a = captures
            .name("A")
            .map(|m| m.as_str().parse::<f32>().expect("expected a float value"))
            .unwrap_or(1.0);
        if !(0.0..=1.0).contains(&a) {
            panic!("invalid alpha range");
        }

        ColorTransparent {
            color: ColorOpaque {
                values: CieLab::from_f32([l, a_star, b_star]),
            },
            alpha: a,
        }
    }

    fn resolve_lch(lch: &str) -> ColorTransparent<CieLch> {
        static MATCHER: OnceCell<Regex> = OnceCell::new();
        let matcher =
            MATCHER.get_or_init(|| Regex::new("lch\\((?<L>[+-]?([0-9]*[.])?[0-9]+) (?<C>[+-]?([0-9]*[.])?[0-9]+) (?<h>[+-]?([0-9]*[.])?[0-9]+)( (?<A>[+-]?([0-9]*[.])?[0-9]+))?\\)").unwrap());
        let captures = matcher.captures(lch).expect("invalid lch string");

        let l = captures
            .name("L")
            .unwrap()
            .as_str()
            .parse::<f32>()
            .expect("expected a float value");
        let c = captures
            .name("a")
            .unwrap()
            .as_str()
            .parse::<f32>()
            .expect("expected a float value");
        let h = captures
            .name("b")
            .unwrap()
            .as_str()
            .parse::<f32>()
            .expect("expected a float value");
        let a = captures
            .name("A")
            .map(|m| m.as_str().parse::<f32>().expect("expected a float value"))
            .unwrap_or(1.0);
        if !(0.0..=1.0).contains(&a) {
            panic!("invalid alpha range");
        }

        ColorTransparent {
            color: ColorOpaque {
                values: CieLch::from_f32([l, c, h]),
            },
            alpha: a,
        }
    }

    fn resolve_css<T>(css: &str) -> ColorTransparent<T>
    where
        T: ColorSpace,
        SRgb: ColorSpaceTransform<T>,
        Xyz: ColorSpaceTransform<T>,
        CieLab: ColorSpaceTransform<T>,
        CieLch: ColorSpaceTransform<T>,
    {
        if let Some(color) = Self::resolve_named(css) {
            color.transform()
        } else if css.starts_with("rgb") {
            Self::resolve_rgb(css).transform()
        } else if css.starts_with("xyz") {
            Self::resolve_xyz(css).transform()
        } else if css.starts_with("lab") {
            Self::resolve_lab(css).transform()
        } else if css.starts_with("lch") {
            Self::resolve_lch(css).transform()
        } else {
            panic!("unrecognized css string {css:?}")
        }
    }
}
