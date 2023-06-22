use std::env;
use crate::args::Args;
use crate::chunk_type::ChunkType;
use deoxys::{
    aead::{Aead, KeyInit, OsRng},
    DeoxysII256,
    Nonce,
};
use std::io::{self, Write};
use rand::Rng;
use generic_array::GenericArray;
use generic_array::typenum::U32;
use std::str::FromStr;
use std::process::Command;
use std::path::Path;

mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut action = String::new();
    let mut application_args: Vec<String> = Vec::new();

    for (index, argument) in env::args().enumerate() {
        if index == 0 { continue; }
        if index == 1 { action = argument.clone() };
        if index == 3 { assert!(ChunkType::from_str(&argument)?.is_valid()) };
        if index > 1 {
            application_args.push(argument.clone());
        }
    }

    if application_args.len() < 2 || application_args.len() > 3 {
        return Err("Invalid number of arguments supplied".into());
    }

    let input = application_args[0].clone();
    

    if input.contains("://") {
        let file_name;
        let delimiter = "/";
        if let Some(index) = input.rfind(delimiter) {
            file_name = input[(index + delimiter.len())..].to_string();
        } else {
            return Err("Invalid URL path".into());
        }
        if Command::new("wget")
        .arg("-q")
        .arg("-O")
        .arg(&file_name)
        .arg(input)
        .output()
        .is_err() {
            return Err("Failed to fetch image from URL".into());
        };
        application_args[0] = file_name;
    }

    if !(Path::new(&application_args[0]).is_file()) {
        return Err("File not found".into());
    }

    if action == "encode" {
        print!("Encrypt? [Y/n] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "Y" | "y" | "" => {
                let key: GenericArray<u8, U32> = DeoxysII256::generate_key(&mut OsRng);
                let cipher = DeoxysII256::new(&key);
                let mut rng = rand::thread_rng();
                let random_bytes: [u8; 15] = rng.gen();
                let nonce = Nonce::from_slice(&random_bytes);
                let ciphertext = cipher.encrypt(nonce, application_args[2].as_bytes()).unwrap();
                application_args[2] = hex::encode(ciphertext);
                println!("Store these safely...");
                println!("Secret key: {}", hex::encode(key));
                println!("Nonce: {}", hex::encode(nonce));
            },
            "N" | "n" => {
            },
            _ => return Err("Invalid input".into()),
        };
    }

    Args::new(&action, &application_args).execute()?;

    Ok(())
}
