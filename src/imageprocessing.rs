use std::path::Path;

use clap::{builder::PossibleValue, ValueEnum};
use eframe::egui::ColorImage;
use image::ColorType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[allow(non_camel_case_types)]
#[non_exhaustive]
pub enum DataType {
    CV_8UC1,
    CV_8UC2,
    #[default]
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

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum ChannelOrder {
    #[default]
    Rgb,
    Bgr,
}

impl ValueEnum for DataType {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::CV_8UC1,
            // Self::CV_8UC2,
            Self::CV_8UC3,
            Self::CV_8UC4,
            Self::CV_16UC1,
            // Self::CV_16UC2,
            Self::CV_16UC3,
            Self::CV_16UC4,
            Self::CV_32FC1,
            // Self::CV_32FC2,
            Self::CV_32FC3,
            Self::CV_32FC4,
            Self::CV_64FC1,
            // Self::CV_64FC2,
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

impl From<DataType> for ColorType {
    fn from(value: DataType) -> Self {
        match value.channels() {
            1 => ColorType::L8,
            2 => ColorType::La8,
            3 => ColorType::Rgb8,
            4 => ColorType::Rgba8,
            _ => unreachable!(),
        }
    }
}

impl DataType {
    pub fn channels(&self) -> u8 {
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

    /// Convert all images to u8 ones to be able to display them
    fn convert_to_supported(&self, bytes: Vec<u8>, channel_order: ChannelOrder) -> Vec<u8> {
        let type_converted = match self {
            DataType::CV_32FC1 | DataType::CV_32FC2 | DataType::CV_32FC3 | DataType::CV_32FC4 => {
                bytes
                    .chunks(4)
                    .flat_map(|c| {
                        assert!(c.len() == 4);
                        let f = f32::from_ne_bytes(c.try_into().unwrap());
                        let u = (f * u8::MAX as f32) as u8;
                        u.to_ne_bytes()
                    })
                    .collect()
            }
            DataType::CV_64FC1 | DataType::CV_64FC2 | DataType::CV_64FC3 | DataType::CV_64FC4 => {
                bytes
                    .chunks(8)
                    .flat_map(|c| {
                        assert!(c.len() == 8);
                        let f = f64::from_ne_bytes(c.try_into().unwrap());
                        let u = (f * u8::MAX as f64) as u8;
                        u.to_ne_bytes()
                    })
                    .collect()
            }
            DataType::CV_16UC1 | DataType::CV_16UC2 | DataType::CV_16UC3 | DataType::CV_16UC4 => {
                bytes
                    .chunks(2)
                    .flat_map(|c| {
                        assert!(c.len() == 2);
                        let u = u16::from_ne_bytes(c.try_into().unwrap());
                        let k = u16::MAX as f32 / u as f32;
                        let u = (k * u8::MAX as f32) as u8;
                        u.to_ne_bytes()
                    })
                    .collect()
            }
            // supported ones
            DataType::CV_8UC1 | DataType::CV_8UC2 | DataType::CV_8UC3 | DataType::CV_8UC4 => bytes,
        };
        let order_converted = match self.channels() {
            // no conversion required for rgb
            _ if channel_order == ChannelOrder::Rgb => type_converted,
            // three and for channelled data needs to be converted
            4 if channel_order == ChannelOrder::Bgr => type_converted
                .chunks(4)
                .flat_map(|c| {
                    assert!(c.len() == 4);
                    [c[2], c[1], c[0], c[3]]
                })
                .collect(),
            3 if channel_order == ChannelOrder::Bgr => type_converted
                .chunks(3)
                .flat_map(|c| {
                    assert!(c.len() == 3);
                    [c[2], c[1], c[0]]
                })
                .collect(),
            // others have no such conversion
            1 | 2 => type_converted,
            _ => unreachable!(),
        };
        order_converted
    }

    /// Creates `ImageData` based on `DataType` with all required
    /// conversions.
    pub fn init_image_data(
        &self,
        bytes: Vec<u8>,
        width: u32,
        height: u32,
        channel_order: ChannelOrder,
    ) -> ImageData {
        let bytes = self.convert_to_supported(bytes, channel_order);
        ImageData {
            data: bytes,
            color_type: (*self).into(),
            width,
            height,
        }
    }
}

#[derive(Clone)]
pub struct ImageData {
    pub data: Vec<u8>,
    pub color_type: ColorType,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub enum ImageProcessingError {
    IoError(std::io::Error),
    ImageError(image::ImageError),
}

pub type ImageProcessingResult = Result<(), ImageProcessingError>;

impl ImageData {
    /// initializes appropriate egui image
    pub fn init_egui_image(&self) -> ColorImage {
        match self.color_type.channel_count() {
            1 => ColorImage::from_gray([self.width as usize, self.height as usize], &self.data),
            3 => ColorImage::from_rgb([self.width as usize, self.height as usize], &self.data),
            4 => ColorImage::from_rgba_unmultiplied(
                [self.width as usize, self.height as usize],
                &self.data,
            ),
            _ => unimplemented!(),
        }
    }

    /// saves image on disk
    pub fn save(&self, filename: &Path) -> ImageProcessingResult {
        if let Some(filedir) = filename.parent() {
            std::fs::create_dir_all(filedir).map_err(ImageProcessingError::IoError)?;
        }
        image::save_buffer(
            filename,
            &self.data,
            self.width,
            self.height,
            self.color_type,
        )
        .map_err(ImageProcessingError::ImageError)
    }
}
