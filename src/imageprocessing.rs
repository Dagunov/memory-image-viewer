use clap::{builder::PossibleValue, ValueEnum};

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_camel_case_types)]
#[non_exhaustive]
pub enum DataType {
    CV_8UC1,
    CV_8UC2,
    CV_8UC3,
    CV_8UC4,
    CV_16UC1,
    CV_16UC2,
    CV_16UC3,
    CV_16UC4,
    CV_32FC1,
    CV_32FC2,
    CV_32FC3,
    CV_32FC4,
    CV_64FC1,
    CV_64FC2,
    CV_64FC3,
    CV_64FC4,
}

impl ValueEnum for DataType {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::CV_8UC1,
            Self::CV_8UC2,
            Self::CV_8UC3,
            Self::CV_8UC4,
            Self::CV_16UC1,
            Self::CV_16UC2,
            Self::CV_16UC3,
            Self::CV_16UC4,
            Self::CV_32FC1,
            Self::CV_32FC2,
            Self::CV_32FC3,
            Self::CV_32FC4,
            Self::CV_64FC1,
            Self::CV_64FC2,
            Self::CV_64FC3,
            Self::CV_64FC4,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            DataType::CV_8UC1 => PossibleValue::new("CV_8UC1").alias("8UC1"),
            DataType::CV_8UC2 => PossibleValue::new("CV_8UC2").alias("8UC2"),
            DataType::CV_8UC3 => PossibleValue::new("CV_8UC3").alias("8UC3"),
            DataType::CV_8UC4 => PossibleValue::new("CV_8UC4").alias("8UC4"),
            DataType::CV_16UC1 => PossibleValue::new("CV_16UC1").alias("16UC1"),
            DataType::CV_16UC2 => PossibleValue::new("CV_16UC2").alias("16UC2"),
            DataType::CV_16UC3 => PossibleValue::new("CV_16UC3").alias("16UC3"),
            DataType::CV_16UC4 => PossibleValue::new("CV_16UC4").alias("16UC4"),
            DataType::CV_32FC1 => PossibleValue::new("CV_32FC1").alias("32FC1"),
            DataType::CV_32FC2 => PossibleValue::new("CV_32FC2").alias("32FC2"),
            DataType::CV_32FC3 => PossibleValue::new("CV_32FC3").alias("32FC3"),
            DataType::CV_32FC4 => PossibleValue::new("CV_32FC4").alias("32FC4"),
            DataType::CV_64FC1 => PossibleValue::new("CV_64FC1").alias("64FC1"),
            DataType::CV_64FC2 => PossibleValue::new("CV_64FC2").alias("64FC2"),
            DataType::CV_64FC3 => PossibleValue::new("CV_64FC3").alias("64FC3"),
            DataType::CV_64FC4 => PossibleValue::new("CV_64FC4").alias("64FC4"),
        })
    }
}

impl From<DataType> for image::ColorType {
    fn from(value: DataType) -> Self {
        match value {
            DataType::CV_8UC1 => image::ColorType::L8,
            DataType::CV_8UC2 => image::ColorType::La8,
            DataType::CV_8UC3 => image::ColorType::Rgb8,
            DataType::CV_8UC4 => image::ColorType::Rgba8,
            DataType::CV_16UC1 => image::ColorType::L16,
            DataType::CV_16UC2 => image::ColorType::La16,
            DataType::CV_16UC3 => image::ColorType::Rgb16,
            DataType::CV_16UC4 => image::ColorType::Rgba16,
            // next types require conversion
            DataType::CV_32FC1 => image::ColorType::L16,
            DataType::CV_32FC2 => image::ColorType::La16,
            DataType::CV_32FC3 => image::ColorType::Rgb16,
            DataType::CV_32FC4 => image::ColorType::Rgba16,
            DataType::CV_64FC1 => image::ColorType::L16,
            DataType::CV_64FC2 => image::ColorType::La16,
            DataType::CV_64FC3 => image::ColorType::Rgb16,
            DataType::CV_64FC4 => image::ColorType::Rgba16,
        }
    }
}

impl DataType {
    fn channels(&self) -> u8 {
        match self {
            DataType::CV_8UC1 | DataType::CV_16UC1 | DataType::CV_32FC1 | DataType::CV_64FC1 => 1,
            DataType::CV_8UC2 | DataType::CV_16UC2 | DataType::CV_32FC2 | DataType::CV_64FC2 => 2,
            DataType::CV_8UC3 | DataType::CV_16UC3 | DataType::CV_32FC3 | DataType::CV_64FC3 => 3,
            DataType::CV_8UC4 | DataType::CV_16UC4 | DataType::CV_32FC4 | DataType::CV_64FC4 => 4,
        }
    }

    fn bytes_per_color(&self) -> u8 {
        match self {
            DataType::CV_8UC1 | DataType::CV_8UC2 | DataType::CV_8UC3 | DataType::CV_8UC4 => 1,
            DataType::CV_16UC1 | DataType::CV_16UC2 | DataType::CV_16UC3 | DataType::CV_16UC4 => 2,
            DataType::CV_32FC1 | DataType::CV_32FC2 | DataType::CV_32FC3 | DataType::CV_32FC4 => 4,
            DataType::CV_64FC1 | DataType::CV_64FC2 | DataType::CV_64FC3 | DataType::CV_64FC4 => 8,
        }
    }

    pub fn bytes_per_pixel(&self) -> u8 {
        self.channels() * self.bytes_per_color()
    }

    /// Convert float image types like `CV_64FC2`
    /// to u16 ones, like `La16`.
    pub fn convert_to_supported(&self, bytes: Vec<u8>) -> Vec<u8> {
        match self {
            DataType::CV_32FC1 | DataType::CV_32FC2 | DataType::CV_32FC3 | DataType::CV_32FC4 => {
                bytes
                    .chunks(4)
                    .map(|c| {
                        assert!(c.len() == 4);
                        let f = f32::from_ne_bytes(c.try_into().unwrap());
                        let u = (f * u16::MAX as f32) as u16;
                        u.to_ne_bytes()
                    })
                    .flatten()
                    .collect()
            }
            DataType::CV_64FC1 | DataType::CV_64FC2 | DataType::CV_64FC3 | DataType::CV_64FC4 => {
                bytes
                    .chunks(8)
                    .map(|c| {
                        assert!(c.len() == 8);
                        let f = f64::from_ne_bytes(c.try_into().unwrap());
                        let u = (f * u16::MAX as f64) as u16;
                        u.to_ne_bytes()
                    })
                    .flatten()
                    .collect()
            }
            // supported ones
            DataType::CV_8UC1
            | DataType::CV_8UC2
            | DataType::CV_8UC3
            | DataType::CV_8UC4
            | DataType::CV_16UC1
            | DataType::CV_16UC2
            | DataType::CV_16UC3
            | DataType::CV_16UC4 => bytes,
        }
    }
}

pub fn save_bytes(
    bytes: Vec<u8>,
    color_type: impl Into<image::ColorType>,
    filename: &str,
    dims: [u32; 2],
) -> image::ImageResult<()> {
    image::save_buffer(filename, &bytes, dims[0], dims[1], color_type.into())
}
