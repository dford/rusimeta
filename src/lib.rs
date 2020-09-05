use std::error::Error;
use std::path::PathBuf;

use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};

// TODO: structs and members need to get marked up with traits for JSON ser/des
#[derive(Debug,Clone,PartialEq)]
pub struct FileMetadataOfInterest {
    pub filename: String,
    pub size: u64, // in bytes
    pub created_time: chrono::DateTime<Utc>,
    pub modified_time: chrono::DateTime<Utc>,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Orientation {
    Normal = 1,
    Mirrored = 2,
    UpsideDown = 3,
    UpsideDownMirrored = 4,
    QuarterRotationCCWMirrored = 5,
    QuarterRotationCCW = 6,
    QuarterRotationCWMirrored = 7,
    QuarterRotationCW = 8,
}

#[derive(Debug,Clone,PartialEq)]
pub struct ImageMetadataOfInterest {
    pub orientation: Option<Orientation>,
    pub capture_time: Option<chrono::NaiveDateTime>,
    pub camera_model: Option<String>,
    pub camera_serial: Option<String>,
}

#[derive(Debug,Clone,PartialEq)]
pub struct MetadataOfInterest {
    pub file_metadata: FileMetadataOfInterest,
    pub image_metadata: ImageMetadataOfInterest,
}

pub fn write_json_metadata( metadata: MetadataOfInterest ) -> Result<(), ()> {
    // TODO
    Ok(())
}

pub fn read_json_metadata( path : &str ) -> Result<MetadataOfInterest, ()> {
    // TODO
    let file_metadata = FileMetadataOfInterest {
        filename: "".to_string(),
        size: 0,
        created_time: chrono::DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
        modified_time: chrono::DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
    };

    let image_metadata = ImageMetadataOfInterest {
        orientation: None,
        capture_time: None,
        camera_model: None,
        camera_serial: None,
    };

    Ok(MetadataOfInterest {
        file_metadata,
        image_metadata
    })
}

pub struct Config {
    image_paths: Vec<PathBuf>,
    print_help: bool,
}

impl Config {
    pub fn new(args: std::env::Args) -> Result<Config, &'static str> {
        // First arg is skipped because it is the executable name
        let raw_args : Vec<String> = args.collect();
        if let Some(first_arg_string) = raw_args.get(1) {
            if first_arg_string == "-h" || first_arg_string == "--help" {
                return Ok(Config {
                    image_paths: vec![],
                    print_help: true,
                });
            }
        } else {
            return Err("Not enough arguments.  Provide at least one path to an image file to read.")
        }
        let paths : Vec<PathBuf> = raw_args.iter().skip(1).map(|arg| { PathBuf::from(&arg) }).collect();

        Ok(Config { 
            image_paths: paths,
            print_help: false,
        })
    }

    pub fn from_strings(strings: Vec<String>) -> Config {
        Config {
            image_paths: strings.iter().map(|arg| { PathBuf::from(&arg) }).collect(),
            print_help: false,
        }
    }

    pub fn image_paths(&self) -> &Vec<PathBuf> {
        &self.image_paths
    }

    pub fn print_help(&self) -> bool {
        self.print_help
    }
}

pub fn run( config : Config ) -> Result<(), Box<dyn Error>> {
    // TODO
    Ok(())
}
