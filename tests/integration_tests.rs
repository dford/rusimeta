use rusimeta::*;
use std::path::Path;
use std::fs;
use serial_test::serial;
use chrono;

#[derive(Debug)]
struct TestFile<'a> {
    path: &'a str,
    expected_json_path: &'a str,
    pub expected_metadata: rusimeta::MetadataOfInterest,
}

impl<'a> TestFile<'a> {
    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn expected_json_path(&self) -> &str {
        &self.expected_json_path
    }

    pub fn expected_metadata(&self) -> rusimeta::MetadataOfInterest {
        self.expected_metadata.clone()
    }
}

struct AllTestData<'a> {
    pub COMPLETE_METADATA_1 : TestFile<'a>,
    pub COMPLETE_METADATA_2 : TestFile<'a>,
    pub COMPLETE_METADATA_3 : TestFile<'a>,
    pub INCOMPLETE_METADATA : TestFile<'a>,
}

impl<'a> AllTestData<'a> {}

const CAPTURE_TIME_EXPECTATION_FORMAT : &str = "%Y:%m:%d %H:%M:%S";

fn get_expected_date_time( str: &str ) -> chrono::NaiveDateTime {
    chrono::NaiveDateTime::parse_from_str(str, CAPTURE_TIME_EXPECTATION_FORMAT).unwrap()
}

// Note for this expected data that created_time and modified_time
// are going to be dependent on when and how the resource files are copied,
// and it does not make sense to hard-code them in the expectation.
//
// They should be filled by test code if and when tested, by reading the created and modified
// times from the file under test.
//
// This function has to be run at the start of each test because the timestamps can't
// be set through const methods.
// There is probably a better way to do this initialization statically and once with a proper test framework, but
// I'm not sure how, and the runtime cost would only matter if this test suite was run frequently.
//
fn init_test_data() -> AllTestData<'static> {
    AllTestData {
        COMPLETE_METADATA_1: TestFile {
            path: "tests/resource/images1/JAM19896.jpg",
            expected_json_path: "tests/resource/images1/JAM19896.json",
            expected_metadata: rusimeta::MetadataOfInterest {
                file_metadata: rusimeta::FileMetadataOfInterest {
                    filename: "JAM19896.jpg".to_string(),
                    size: 953_458,
                    created_time: None,
                    modified_time: None,
                },
                image_metadata: rusimeta::ImageMetadataOfInterest {
                    orientation: Some(Orientation::Normal),
                    capture_time: Some(get_expected_date_time("2019:07:26 13:25:33")),
                    camera_model: Some("Canon EOS 5D Mark IV".to_string()),
                    camera_serial: Some("025021000537".to_string()),
                },
            },
        },
        COMPLETE_METADATA_2: TestFile {
            path: "tests/resource/images1/JAM26284.jpg",
            expected_json_path: "tests/resource/images1/JAM26284.json",
            expected_metadata: rusimeta::MetadataOfInterest {
                file_metadata: rusimeta::FileMetadataOfInterest {
                    filename: "JAM26284.jpg".to_string(),
                    size: 574_207,
                    created_time: None,
                    modified_time: None,
                },
                image_metadata: rusimeta::ImageMetadataOfInterest {
                    orientation: Some(Orientation::Normal),
                    capture_time: Some(get_expected_date_time("2020:01:30 09:28:07")),
                    camera_model: Some("Canon EOS 5D Mark IV".to_string()),
                    camera_serial: Some("025021000535".to_string()),
                },
            },
        },
        COMPLETE_METADATA_3: TestFile {
            path: "tests/resource/images2/JAM26496.jpg",
            expected_json_path: "tests/resource/images2/JAM26496.json",
            expected_metadata: rusimeta::MetadataOfInterest {
                file_metadata: rusimeta::FileMetadataOfInterest {
                    filename: "JAM26496.jpg".to_string(),
                    size: 353_914,
                    created_time: None,
                    modified_time: None,
                },
                image_metadata: rusimeta::ImageMetadataOfInterest {
                    orientation: Some(Orientation::Normal),
                    capture_time: Some(get_expected_date_time("2020:01:30 09:44:56")),
                    camera_model: Some("Canon EOS 5D Mark IV".to_string()),
                    camera_serial: Some("025021000535".to_string()),
                },
            },
        },
        INCOMPLETE_METADATA: TestFile {
            path: "tests/resource/images2/rotated_CCW90.jpg",
            expected_json_path: "tests/resource/images2/rotated_CCW90.json",
            expected_metadata: rusimeta::MetadataOfInterest {
                file_metadata: rusimeta::FileMetadataOfInterest {
                    filename: "rotated_CCW90.jpg".to_string(),
                    size: 327_616,
                    created_time: None,
                    modified_time: None,
                },
                image_metadata: rusimeta::ImageMetadataOfInterest {
                    orientation: Some(Orientation::QuarterRotationCCW),
                    capture_time: None,
                    camera_model: None,
                    camera_serial: None,
                },
            },
        },
    }
}

const UNSUPPORTED_FILE_PATH_TEXT : &str = "tests/resource/unsupported_files/notanimage.txt";
const UNSUPPORTED_FILE_PATH_MUSIC : &str = "tests/resource/unsupported_files/Edenal.mp3";

// Tests are run serially because they may touch the same files and the result would be
// a race condition on the filesystem if multiple tests tried to write to the same JSON file.
#[test]
#[serial]
fn single_file_with_complete_metadata_can_be_read_and_serialized_to_json()
{
    // GIVEN a path to an image file containing all of the metadata of interest ("complete")
    let td = init_test_data();
    let cfg = rusimeta::Config::from_strings( vec![td.COMPLETE_METADATA_1.path().to_string()] );

    // WHEN the metadata is requested
    let result = rusimeta::run( cfg );
    assert!(result.is_ok(),"{:?}",result.err());

    // THEN the JSON output is written and as expected; all fields are populated
    assert!(Path::new(td.COMPLETE_METADATA_1.expected_json_path()).exists());
    assert_eq!(
        td.COMPLETE_METADATA_1.expected_metadata(),
        rusimeta::read_json_metadata(td.COMPLETE_METADATA_1.expected_json_path()).unwrap()
    );
}

#[test]
#[serial]
fn single_file_with_incomplete_metadata_can_be_read_and_serialized_to_json()
{
    // GIVEN a path to an image file containing only some of the metadata of interest ("incomplete")
    let td = init_test_data();
    let cfg = rusimeta::Config::from_strings( vec![td.INCOMPLETE_METADATA.path().to_string()] );

    // WHEN the metadata is requested
    let result = rusimeta::run( cfg );
    assert!(result.is_ok(),"{:?}",result.err());

    // THEN the JSON output is written and as expected; only the fields corresponding to the present input metadata are present
    assert!(Path::new(td.INCOMPLETE_METADATA.expected_json_path()).exists());
    assert_eq!(
        td.INCOMPLETE_METADATA.expected_metadata(),
        rusimeta::read_json_metadata(td.INCOMPLETE_METADATA.expected_json_path()).unwrap()
    );
}

#[test]
#[serial]
fn multiple_files_metadata_can_be_read_and_dumped_and_serialized_to_json()
{
    // GIVEN paths to multiple image files with a mix of metadata
    let td = init_test_data();
    let test_files : Vec<&TestFile> = vec![
        &td.COMPLETE_METADATA_1,
        &td.COMPLETE_METADATA_2,
        &td.COMPLETE_METADATA_3,
        &td.INCOMPLETE_METADATA,
    ];

    let cfg = rusimeta::Config::from_strings( test_files.iter().map(|tf| tf.path().to_string()).collect() );

    // WHEN the metadata is requested for all of these files
    let result = rusimeta::run( cfg );
    assert!(result.is_ok(),"{:?}",result.err());

    // THEN the expected JSON output is written for each of the inputs
    for test_file in test_files {
        assert!(Path::new(test_file.expected_json_path()).exists());
        assert_eq!(
            test_file.expected_metadata(),
            rusimeta::read_json_metadata(test_file.expected_json_path()).unwrap()
        );
    }
}

#[test]
#[serial]
fn duplicate_files_metadata_can_be_read_and_paths_can_be_absolute_or_relative()
{
    // GIVEN a set of input image file paths containing duplicate relative paths
    let td = init_test_data();
    let test_files : Vec<&TestFile> = vec![
        &td.COMPLETE_METADATA_1,
        &td.COMPLETE_METADATA_2,
        &td.COMPLETE_METADATA_1,
    ];

    // AND GIVEN an additional path which corresponds to the same duplicate file, but which is absolute instead of relative
    let mut test_paths : Vec<String> = test_files.iter().map(|tf| tf.path().to_string()).collect();
    test_paths.push(fs::canonicalize(td.COMPLETE_METADATA_1.path()).unwrap().to_str().unwrap().to_string()); // make it absolute

    let cfg = rusimeta::Config::from_strings( test_paths );

    // WHEN the metadata is requested for all of these files
    let result = rusimeta::run( cfg );
    assert!(result.is_ok(),"{:?}",result.err());

    // THEN the expected JSON output is written for each of the inputs, the same as if no duplicate paths had been provided
    assert!(Path::new(td.COMPLETE_METADATA_1.expected_json_path()).exists());
    assert_eq!(
        td.COMPLETE_METADATA_1.expected_metadata(),
        rusimeta::read_json_metadata(td.COMPLETE_METADATA_1.expected_json_path()).unwrap()
    );

    assert!(Path::new(td.COMPLETE_METADATA_2.expected_json_path()).exists());
    assert_eq!(
        td.COMPLETE_METADATA_2.expected_metadata(),
        rusimeta::read_json_metadata(td.COMPLETE_METADATA_2.expected_json_path()).unwrap()
    );
}

// This test is simply expected to correctly process the supported files and ignore the unsupported files without panicking.
#[test]
#[serial]
fn wrong_file_types_are_ignored()
{
    // GIVEN a set of input file paths which include unsupported files (not images)
    let td = init_test_data();
    let test_paths : Vec<String> = vec![
        td.COMPLETE_METADATA_1.path().to_string(),
        td.INCOMPLETE_METADATA.path().to_string(),
        UNSUPPORTED_FILE_PATH_TEXT.to_string(),
        UNSUPPORTED_FILE_PATH_MUSIC.to_string(),
    ];
    
    let cfg = rusimeta::Config::from_strings( test_paths );

    // WHEN the metadata is requested for all of these files
    let result = rusimeta::run( cfg );
    assert!(result.is_ok(),"{:?}",result.err());

    // THEN the expected JSON output is written for each of the supported inputs, and the program does not panic
    assert!(Path::new(td.COMPLETE_METADATA_1.expected_json_path()).exists());
    assert_eq!(
        td.COMPLETE_METADATA_1.expected_metadata(),
        rusimeta::read_json_metadata(td.COMPLETE_METADATA_1.expected_json_path()).unwrap()
    );

    assert!(Path::new(td.INCOMPLETE_METADATA.expected_json_path()).exists());
    assert_eq!(
        td.INCOMPLETE_METADATA.expected_metadata(),
        rusimeta::read_json_metadata(td.INCOMPLETE_METADATA.expected_json_path()).unwrap()
    );

    // DoMore - this should ideally also check the number of json files in the output directory
    // and make sure that none were created for the unsupported files (i.e. the expected number should be present)
}
