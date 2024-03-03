use clap::{builder::PossibleValue, Parser, ValueEnum};
use read_process_memory::{copy_address, Pid, ProcessHandle};

#[derive(Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
#[non_exhaustive]
enum DataType {
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
            DataType::CV_32FC3 => image::ColorType::Rgb32F,
            DataType::CV_32FC4 => image::ColorType::Rgba32F,
            // next types require conversion
            DataType::CV_32FC1 => image::ColorType::L16,
            DataType::CV_32FC2 => image::ColorType::La16,
            DataType::CV_64FC1 => image::ColorType::L16,
            DataType::CV_64FC2 => image::ColorType::La16,
            DataType::CV_64FC3 => image::ColorType::Rgb32F,
            DataType::CV_64FC4 => image::ColorType::Rgba32F,
        }
    }
}

impl DataType {
    pub fn channels(&self) -> u8 {
        match self {
            DataType::CV_8UC1
            | DataType::CV_16UC1
            | DataType::CV_32FC1
            | DataType::CV_64FC1 => 1,
            DataType::CV_8UC2
            | DataType::CV_16UC2
            | DataType::CV_32FC2
            | DataType::CV_64FC2 => 2,
            DataType::CV_8UC3
            | DataType::CV_16UC3
            | DataType::CV_32FC3
            | DataType::CV_64FC3 => 3,
            DataType::CV_8UC4
            | DataType::CV_16UC4
            | DataType::CV_32FC4
            | DataType::CV_64FC4 => 4,
        }
    }

    pub fn bytes_per_color(&self) -> u8 {
        match self {
            DataType::CV_8UC1
            | DataType::CV_8UC2
            | DataType::CV_8UC3
            | DataType::CV_8UC4 => 1,
            DataType::CV_16UC1
            | DataType::CV_16UC2
            | DataType::CV_16UC3
            | DataType::CV_16UC4 => 2,
            DataType::CV_32FC1
            | DataType::CV_32FC2
            | DataType::CV_32FC3
            | DataType::CV_32FC4 => 4,
            DataType::CV_64FC1
            | DataType::CV_64FC2
            | DataType::CV_64FC3
            | DataType::CV_64FC4 => 8,
        }
    }

    /// Convert usupported by `image` types like `CV_64FC2`
    /// to supported ones, like `La16`
    pub fn convert_to_supported(&self, bytes: Vec<u8>) -> Vec<u8> {
        match self {
            DataType::CV_32FC1
            | DataType::CV_32FC2 => {
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
            },
            DataType::CV_64FC1
            | DataType::CV_64FC2 => {
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
            DataType::CV_64FC3
            | DataType::CV_64FC4 => {
                bytes
                    .chunks(8)
                    .map(|c| {
                        assert!(c.len() == 8);
                        let f = f64::from_ne_bytes(c.try_into().unwrap()) as f32;
                        f.to_ne_bytes()
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
            | DataType::CV_16UC4
            | DataType::CV_32FC3
            | DataType::CV_32FC4 => bytes,
        }
    }

    pub fn extention(&self) -> &str {
        match self {
            DataType::CV_8UC1
            | DataType::CV_8UC2
            | DataType::CV_8UC3
            | DataType::CV_8UC4
            | DataType::CV_16UC1
            | DataType::CV_16UC2
            | DataType::CV_16UC3
            | DataType::CV_16UC4
            | DataType::CV_32FC1
            | DataType::CV_32FC2
            | DataType::CV_64FC1
            | DataType::CV_64FC2 => "png",
            DataType::CV_32FC3
            | DataType::CV_32FC4
            | DataType::CV_64FC3
            | DataType::CV_64FC4 => "exr",
        }
    }
}

/// Tool which allows to save image from cv::Mat from memory
#[derive(Parser)]
#[command(about, version)]
struct CLI {
    /// PID of target process
    pid: usize,

    /// Target memory address in process
    addr: String,

    /// Width of image
    width: u32,

    /// Height of image
    height: u32,

    /// Buffer type
    #[arg(value_enum)]
    buf_type: DataType,

    /// Out file name
    #[arg(short, long, default_value_t = String::from("out"))]
    out: String,
}

fn main() {
    let cli = CLI::parse();
    process(cli);
}

fn process(cli: CLI) {
    let handle = ProcessHandle::try_from(cli.pid as Pid).unwrap();
    let buff_size = cli.width as usize * cli.height as usize * cli.buf_type.channels() as usize * cli.buf_type.bytes_per_color() as usize;
    let addr = usize::from_str_radix(cli.addr.trim_start_matches("0x"), 16).unwrap();
    match copy_address(addr, buff_size, &handle) {
        Ok(bytes) => save_bytes(cli.buf_type.convert_to_supported(bytes), cli.buf_type, &(cli.out + "." + cli.buf_type.extention()), [cli.width, cli.height]),
        Err(e) => println!("Could not read from memory: {:?}", e),
    }
}

fn save_bytes(bytes: Vec<u8>, color_type: impl Into<image::ColorType>, filename: &str, dims: [u32; 2]) {
    match image::save_buffer(filename, &bytes, dims[0], dims[1], color_type.into()) {
        Ok(_) => println!("Image saved!"),
        Err(e) => println!("Could not save an image: {:?}", e),
    }
}