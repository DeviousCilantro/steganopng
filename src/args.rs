use std::str::FromStr;
use crate::chunk_type::ChunkType;
use crate::chunk::Chunk;
use crate::png::Png;
use optional_field::Field;
use std::fs::File;
use std::io::{self, Read, Write};
use generic_array::GenericArray;
use generic_array::typenum::U32;
use deoxys::{
    aead::{Aead, KeyInit},
    DeoxysII256,
};

pub enum Command {
    Encode {
        file_path: String,
        chunk_type: ChunkType,
        message: String,
        output_file: Field<String>,
    },
    Decode {
        file_path: String,
        chunk_type: ChunkType,
    },
    Remove {
        file_path: String,
        chunk_type: ChunkType,
    },
    Print {
        file_path: String,
    },
}

impl Command {
    pub fn new(command: &str, args: &[String]) -> Self {
        match command {
            "encode" => {
                Self::Encode {
                    file_path: args[0].clone(),
                    chunk_type: ChunkType::from_str(&args[1]).unwrap(),
                    message: args[2].clone(),
                    output_file: {
                        if args.len() > 3 {
                            Field::Present(Some(args[3].clone()))
                        } else {
                            Field::Missing
                        }
                    },
                }
            },
            "decode" => {
                Self::Decode {
                    file_path: args[0].clone(),
                    chunk_type: ChunkType::from_str(&args[1]).unwrap(),
                }
            },
            "remove" => {
                Self::Remove {
                    file_path: args[0].clone(),
                    chunk_type: ChunkType::from_str(&args[1]).unwrap(),
                }
            },
            "print" => {
                Self::Print {
                    file_path: args[0].clone(),
                }
            },
            _ => panic!("Invalid command"),
        }
    }

    pub fn read_file(file_path: String) -> Result<Png, Box<dyn std::error::Error>> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let png = Png::try_from(&buffer[..]).unwrap();
        Ok(png)
    }

    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Encode { file_path, chunk_type, message, output_file }  => {
                let mut png = Self::read_file(file_path.clone())?;
                png.append_chunk(Chunk::new(chunk_type.clone(), message.bytes().collect()));
                let output = png.as_bytes();
                if output_file.clone().is_present() {
                    let mut file = File::create(output_file.clone().unwrap())?;
                    file.write_all(&output)?;
                } else {
                    assert!(png.chunk_by_type(&String::from_utf8(chunk_type.bytes().to_vec()).unwrap()).is_none(), "Chunk already exists");
                    let mut file = File::create(file_path)?;
                    file.write_all(&output)?;
                }
            },
            Self::Decode { file_path, chunk_type } => {
                let png = Self::read_file(file_path.clone())?;
                let Some(chunk) = png.chunk_by_type(&String::from_utf8(chunk_type.bytes().to_vec()).unwrap()) else {
                    panic!("Chunk not found");
                };
                let data = chunk.data_as_string().unwrap();
                print!("Decrypt? [Y/n] ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                match input.trim() {
                    "Y" | "y" | "" => {
                        print!("Enter key: ");
                        io::stdout().flush()?;
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        let key: GenericArray<u8, U32> = *GenericArray::from_slice(&hex::decode(input.trim()).unwrap());
                        print!("Enter nonce: ");
                        io::stdout().flush()?;
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        let nonce: [u8; 15] = Self::convert_to_fixed_slice(hex::decode(input.trim()).unwrap().as_slice());
                        let nonce: GenericArray<u8, _> =  *GenericArray::from_slice(&nonce);
                        let cipher = DeoxysII256::new(&key);
                        let plaintext = cipher.decrypt(&nonce, hex::decode(data).unwrap().as_ref()).unwrap();
                        println!("Retrieved message: {}", String::from_utf8(plaintext).unwrap());
                    },
                    "N" | "n" => println!("Retrieved message: {}", &chunk.data_as_string().unwrap()),
                    _ => panic!("Invalid input"),
                };
            },
            Self::Remove { file_path, chunk_type } => {
                let mut png = Self::read_file(file_path.clone())?;
                let chunk = String::from_utf8(chunk_type.bytes().to_vec()).unwrap();
                png.remove_chunk(&chunk)?;
                let output = png.as_bytes();
                let mut file = File::create(file_path)?;
                file.write_all(&output)?;
                println!("Removed chunk: {chunk}");
            },
            Self::Print { file_path } => {
                let png = Self::read_file(file_path.clone())?;
                println!("{png:?}");
            },
        };
        Ok(())
    }

    pub fn convert_to_fixed_slice(v: &[u8]) -> [u8; 15] {
        v.try_into().unwrap()
    }

}

