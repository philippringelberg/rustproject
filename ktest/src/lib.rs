use std::fs::File;
use std::io::prelude::*; // provide io traits
use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub struct KTEST {
    pub version: i32,
    pub args: Vec<String>,
    pub objects: Vec<(String, Vec<u8>)>,
}

pub fn read_ktest(file_name: &str) -> std::io::Result<KTEST> {
    let mut file: File = File::open(file_name)?;
    let mut hdr = [0u8; 5];
    file.read_exact(&mut hdr)?;
    if &hdr != b"KTEST" {
        return Err(Error::new(ErrorKind::Other, "not a KTEST file"));
    }

    let version = read_i32(&mut file)?;
    println!("version : {}", version);
    if version > 3 {
        return Err(Error::new(ErrorKind::Other, "non support KTEST version"));
    }

    let num_args = read_i32(&mut file)?;

    // info regarding the KTEST file
    let mut args = vec![];
    for _ in 0..num_args {
        let arg = read_sized(&mut file)?;
        let str = String::from_utf8(arg).unwrap();
        args.push(str);
    }

    // metadata not used here
    let _sym_argvs = read_i32(&mut file)?;
    let _sym_argv_len = read_i32(&mut file)?;

    // read the objects
    let num_objects = read_i32(&mut file)?;
    let mut objects = vec![];
    for _ in 0..num_objects {
        let name = read_string(&mut file)?;
        let data = read_sized(&mut file)?;
        objects.push((name, data))
    }

    Ok(KTEST {
        version,
        args,
        objects,
    })
}

fn read_i32(file: &mut File) -> std::io::Result<i32> {
    let mut str = [0u8; 4];
    file.read_exact(&mut str)?;
    Ok(i32::from_be_bytes(str)) // big endian
}

fn read_string(file: &mut File) -> std::io::Result<String> {
    let str = read_sized(file)?;
    Ok(String::from_utf8(str).unwrap())
}

fn read_sized(file: &mut File) -> std::io::Result<Vec<u8>> {
    let size = read_i32(file)?;
    let mut buf = vec![0u8; size as usize];
    file.read_exact(&mut buf)?;
    Ok(buf)
}
