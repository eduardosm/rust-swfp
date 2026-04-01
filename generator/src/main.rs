#![warn(
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_qualifications
)]
#![forbid(unsafe_code)]
#![allow(clippy::too_many_arguments)]

use std::path::Path;
use std::process::ExitCode;

mod rt_data;
mod test_data;

struct RunError;

fn main() -> ExitCode {
    let mut args = std::env::args_os();
    let arg0 = args.next().unwrap();

    let Some(arg1) = args.next() else {
        eprintln!("Usage: {} <MODE>", arg0.to_string_lossy());
        return ExitCode::FAILURE;
    };

    match arg1.to_str() {
        Some("rt-data") => {
            if args.len() < 1 {
                eprintln!(
                    "Usage: {} {} <paths...>",
                    arg1.to_string_lossy(),
                    arg0.to_string_lossy(),
                );
                return ExitCode::FAILURE;
            }

            for arg in args {
                if let Err(RunError) = rt_data::proc_path(Path::new(&arg)) {
                    return ExitCode::FAILURE;
                }
            }

            ExitCode::SUCCESS
        }
        Some("test-data") => {
            if args.len() != 1 {
                eprintln!(
                    "Usage: {} {} <function>",
                    arg0.to_string_lossy(),
                    arg1.to_string_lossy(),
                );
                return ExitCode::FAILURE;
            }

            let arg2 = args.next().unwrap();

            match test_data::run(&arg2) {
                Ok(()) => ExitCode::SUCCESS,
                Err(RunError) => ExitCode::FAILURE,
            }
        }
        _ => {
            eprintln!("Invalid mode: {arg1:?}");
            ExitCode::FAILURE
        }
    }
}
