use crc64fast_nvme::Digest;
/// Generates CRC-64/NVME checksums, using SIMD-accelerated
/// carryless-multiplication, from a file on disk.
use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::process::ExitCode;

const CRC_NVME: crc::Algorithm<u64> = crc::Algorithm {
    width: 64,
    poly: 0xAD93D23594C93659,
    init: 0xFFFFFFFFFFFFFFFF,
    refin: true,
    refout: true,
    xorout: 0xFFFFFFFFFFFFFFFF,
    check: 0xae8b14860a799888,
    residue: 0x0000000000000000,
};

// Define a chunk size for reading files, e.g., 100MB.
const CHUNK_SIZE: usize = 100 * 1024 * 1024;

/// Calculates the CRC-64/NVME checksum for a file by reading it in chunks.
/// This version uses the SIMD-accelerated implementation.
fn calculate_crc_64_simd_from_file(file_path: &str) -> io::Result<u64> {
    let mut c = Digest::new();

    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = vec![0; CHUNK_SIZE];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        c.write(&buffer[..bytes_read]);
}

    Ok(c.sum64())
}

/// Calculates the CRC-64/NVME checksum for a file by reading it in chunks.
/// This version is for validation and is typically slower.
fn calculate_crc_64_validate_from_file(file_path: &str) -> io::Result<u64> {
    let crc = crc::Crc::<u64>::new(&CRC_NVME);
    let mut digest = crc.digest();

    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = vec![0; CHUNK_SIZE];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        digest.update(&buffer[..bytes_read]);
    }

    Ok(digest.finalize())
}

fn calculate_crc_64_simd_from_string(input: &str) -> u64 {
    let mut c = Digest::new();

    c.write(input.as_bytes());

    c.sum64()
}

fn calculate_crc_64_validate_from_string(input: &str) -> u64 {
    let crc = crc::Crc::<u64>::new(&CRC_NVME);

    let mut digest = crc.digest();

    digest.update(input.as_bytes());

    digest.finalize()
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: crc_64_nvm_checksum [--inputType] [inputString] [--validate-slow]");
        println!("Example for a file: crc_64_nvm_checksum --file /path/to/file");
        println!("Example for a string: crc_64_nvm_checksum --string 123456789");
        println!("Optionally including '--validate-slow' in the argument list will skip SIMD calculation, typically just for testing.");

        return ExitCode::from(1);
    }

    let input_type = &args[1];
    let input = &args[2];

    match input_type.as_str() {
        "--file" => {
            if fs::metadata(input).is_err() {
                println!("Couldn't open file {}", input);
            return ExitCode::from(1);
        }

            let use_slow_validation = args.len() == 4 && args[3] == "--validate-slow";

            let result = if use_slow_validation {
                calculate_crc_64_validate_from_file(input)
            } else {
                calculate_crc_64_simd_from_file(input)
            };

            match result {
                Ok(checksum) => {
                    println!("{}", checksum);
                    ExitCode::SUCCESS
    }
                Err(e) => {
                    println!("Error processing file {}: {}", input, e);
                    ExitCode::from(1)
        }
        }
    }
        "--string" => {
            let use_slow_validation = args.len() == 4 && args[3] == "--validate-slow";

            let checksum = if use_slow_validation {
                calculate_crc_64_validate_from_string(input)
            } else {
                calculate_crc_64_simd_from_string(input)
            };
            println!("{}", checksum);
            ExitCode::SUCCESS
        }

        _ => {
            println!("Invalid input type. Use --file or --string.");
    ExitCode::from(1)
}
    }
}

