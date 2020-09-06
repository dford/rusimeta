use std::convert::TryFrom;
use std::error;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path;
use std::path::Path;
use std::path::PathBuf;
use std::str;

use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
use exif;

use serde_json;
use serde::{Serialize, Deserialize};

const CAPTURE_TIME_FORMAT : &str = "%Y:%m:%d %H:%M:%S";

// TODO: structs and members need to get marked up with traits for JSON ser/des
#[derive(Debug,Clone,PartialEq)]
pub struct FileMetadataOfInterest {
    pub filename: String,
    pub size: u64, // in bytes
    pub created_time: Option<chrono::DateTime<Utc>>,
    pub modified_time: Option<chrono::DateTime<Utc>>,
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

impl TryFrom<u16> for Orientation {
    type Error = ();

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            x if x == Orientation::Normal as u16 => Ok(Orientation::Normal),
            x if x == Orientation::Mirrored as u16 => Ok(Orientation::Mirrored),
            x if x == Orientation::UpsideDown as u16 => Ok(Orientation::UpsideDown),
            x if x == Orientation::UpsideDownMirrored as u16 => Ok(Orientation::UpsideDownMirrored),
            x if x == Orientation::QuarterRotationCCWMirrored as u16 => Ok(Orientation::QuarterRotationCCWMirrored),
            x if x == Orientation::QuarterRotationCCW as u16 => Ok(Orientation::QuarterRotationCCW),
            x if x == Orientation::QuarterRotationCWMirrored as u16 => Ok(Orientation::QuarterRotationCWMirrored),
            x if x == Orientation::QuarterRotationCW as u16 => Ok(Orientation::QuarterRotationCW),
            _ => Err(()),
        }
    }
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

pub struct Config {
    image_paths: Vec<PathBuf>,
    print_help: bool,
}

pub fn run( config : Config ) -> Result<(), Box<dyn error::Error>> {
    // TODO: actually write out the metadata instead of printing it
    for image_path in config.image_paths.iter() {
        let metadata = read_metadata_of_interest( image_path )?;
        println!("{:?}",metadata);
    }

    Ok(())
}

pub fn write_json_metadata( metadata: MetadataOfInterest ) -> Result<(), Box<dyn error::Error>> {
    // TODO
    Ok(())
}

pub fn read_json_metadata( path : &str ) -> Result<MetadataOfInterest, Box<dyn error::Error>> {
    // TODO
    let file_metadata = FileMetadataOfInterest {
        filename: "".to_string(),
        size: 0,
        created_time: None,
        modified_time: None,
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

#[derive(Debug, Clone)]
struct NotAFileError {
    error_path: PathBuf,
}

impl fmt::Display for NotAFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Given path does not correspond to a file: {}", self.error_path.to_string_lossy())
    }
}

impl Error for NotAFileError {
    fn description(&self) -> &str {
        "A given file path does not correspond to a file"
    }
}

fn read_file_metadata( path: &Path ) -> Result<FileMetadataOfInterest, Box<dyn error::Error>> {
    let all_file_metadata = fs::metadata( path )?;

    if !all_file_metadata.is_file() {
        return Err(Box::new(NotAFileError{ error_path: PathBuf::from( path ) }));
    }

    let created_time_utc = if let Ok(created_time) = all_file_metadata.created() {
        Some(chrono::DateTime::<Utc>::from( created_time ))
    } else {
        println!( "Warning: reading the time created is not supported for file: {}",path.to_string_lossy());
        None
    };

    let modified_time_utc = if let Ok(modified_time) = all_file_metadata.modified() {
        Some(chrono::DateTime::<Utc>::from( modified_time ))
    } else {
        println!( "Warning: reading the time modified is not supported for file: {}",path.to_string_lossy());
        None
    };

    Ok( FileMetadataOfInterest {
        filename: path.file_name().unwrap().to_string_lossy().into_owned(),
        size: all_file_metadata.len(),
        created_time: created_time_utc,
        modified_time: modified_time_utc,
    } )
}

fn get_exif_fields( path: &Path ) -> Result<exif::Exif, Box<dyn error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();

    match exifreader.read_from_container(&mut bufreader) {
        Ok(good_exif) => Ok(good_exif),
        Err(unboxed_err) => Err(Box::new(unboxed_err))
    }
}

#[derive(Debug, Clone)]
struct WrongFieldTypeError {
    field_tag : exif::Tag
}

impl fmt::Display for WrongFieldTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EXIF field: {} had unexpected type", self.field_tag.description().unwrap_or("<unknown field>"))
    }
}

impl Error for WrongFieldTypeError {
    fn description(&self) -> &str {
        "An EXIF field had an unexpected type"
    }
}

fn read_exif_metadata( path: &Path ) -> Result<ImageMetadataOfInterest, Box<dyn error::Error>> {
    let exif_fields = get_exif_fields( path )?;

    let maybe_orientation_field = exif_fields.get_field( exif::Tag::Orientation, exif::In::PRIMARY );
    let orientation = if let Some(orientation_field) = maybe_orientation_field {
        if let exif::Value::Short(orientation_raw) = &orientation_field.value {
            if let Some(first_orientation) = orientation_raw.first() {
                if let Ok(valid_orientation) = Orientation::try_from( *first_orientation ) {
                    Some( valid_orientation )
                } else {
                    eprintln!("Invalid orientation value read: {}",first_orientation);
                    None
                }
            } else {
                None
            }
        } else {
            eprintln!("{}",WrongFieldTypeError{ field_tag: exif::Tag::Orientation });
            None
        }
    } else {
        None
    };

    let maybe_dto = exif_fields.get_field( exif::Tag::DateTimeOriginal, exif::In::PRIMARY );
    let capture_time = if let Some(dto_field) = maybe_dto {
        if let exif::Value::Ascii(dto_raw) = &dto_field.value {
            if let Some(first_dto) = dto_raw.first() {
                if let Ok(dto_string) = str::from_utf8( first_dto as &[u8] ) {
                    if let Ok(valid_dto) = chrono::NaiveDateTime::parse_from_str( dto_string, CAPTURE_TIME_FORMAT ) {
                        Some( valid_dto )
                    } else {
                        eprintln!("Date/time string: {} has wrong formatting, for file: {}",dto_string,path.to_string_lossy());
                        None
                    }
                } else {
                    eprintln!("Date/time original string is not valid UTF-8, for file: {}",path.to_string_lossy());
                    None
                }
            } else {
                None
            }
        } else {
            eprintln!("{}",WrongFieldTypeError{ field_tag: exif::Tag::DateTimeOriginal });
            None
        }
    } else {
        None
    };

    let maybe_cm = exif_fields.get_field( exif::Tag::Model, exif::In::PRIMARY );
    let camera_model = if let Some(cm_field) = maybe_cm {
        if let exif::Value::Ascii(cm_raw) = &cm_field.value {
            if let Some(first_cm) = cm_raw.first() {
                if let Ok(cm_string) = str::from_utf8( first_cm as &[u8] ) {
                    Some(String::from(cm_string))
                } else {
                    eprintln!("Camera model string is not valid UTF-8, for file: {}",path.to_string_lossy());
                    None
                }
            } else {
                None
            }
        } else {
            eprintln!("{}",WrongFieldTypeError{ field_tag: exif::Tag::Model });
            None
        }
    } else {
        None
    };

    let maybe_cs = exif_fields.get_field( exif::Tag::BodySerialNumber, exif::In::PRIMARY );
    let camera_serial = if let Some(cs_field) = maybe_cs {
        if let exif::Value::Ascii(cs_raw) = &cs_field.value {
            if let Some(first_cs) = cs_raw.first() {
                if let Ok(cs_string) = str::from_utf8( first_cs as &[u8] ) {
                    Some(String::from(cs_string))
                } else {
                    eprintln!("Camera serial string is not valid UTF-8, for file: {}",path.to_string_lossy());
                    None
                }
            } else {
                None
            }
        } else {
            eprintln!("{}",WrongFieldTypeError{ field_tag: exif::Tag::BodySerialNumber });
            None
        }
    } else {
        None
    };
    
    Ok( ImageMetadataOfInterest {
        orientation,
        capture_time,
        camera_model,
        camera_serial
    } )
}

fn read_metadata_of_interest( path: &Path ) -> Result<MetadataOfInterest, Box<dyn error::Error>> {
    let file_metadata = read_file_metadata( path )?;
    let image_metadata = read_exif_metadata( path )?;

    Ok( MetadataOfInterest {
        file_metadata,
        image_metadata
    } )
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
