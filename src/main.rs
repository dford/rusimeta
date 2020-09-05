use std::process;
use rusimeta;

fn main() {
    let cfg_result = rusimeta::Config::new( std::env::args() );

    if let Ok(cfg) = cfg_result {
        if cfg.print_help() {
            println!("\
Provide paths (absolute or relative) to image files whose metadata should be read.
The read metadata will be written to JSON files in the same directory as the matching images.

Example usage:
rusimeta images/my_image1.jpg images/my_image2.tiff
");
            process::exit(0);
        }

        let run_result = rusimeta::run( cfg );
        match run_result {
            Ok(_) => process::exit(0),
            Err(some_err) => {
                eprintln!("rusimeta encountered a fatal error: {:?}",some_err);
                process::exit(1);
            },
        }
    } else {
        eprintln!("Error parsing arguments: {:?}",cfg_result.err())
    }
}
