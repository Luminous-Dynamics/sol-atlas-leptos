use energy_dispatch_mip_reference::build_model_set;
use serde_json::to_vec_pretty;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn sha256(bytes:&[u8])->String { hex::encode(Sha256::digest(bytes)) }

fn run()->Result<ExitCode,String>{
    let mut args=env::args_os();
    let program=args.next().unwrap_or_default();
    let Some(request_path)=args.next() else {
        eprintln!("usage: {} <request.json> <output-directory>",program.to_string_lossy());
        return Ok(ExitCode::from(64));
    };
    let Some(output_dir)=args.next() else {
        eprintln!("usage: {} <request.json> <output-directory>",program.to_string_lossy());
        return Ok(ExitCode::from(64));
    };
    if args.next().is_some(){
        eprintln!("expected exactly two arguments");
        return Ok(ExitCode::from(64));
    }
    let metadata=fs::metadata(&request_path).map_err(|e|format!("request metadata: {e}"))?;
    if !metadata.file_type().is_file() || metadata.len()>1_048_576 {
        return Err("request must be a regular file no larger than 1 MiB".to_owned());
    }
    let bytes=fs::read(&request_path).map_err(|e|format!("request read: {e}"))?;
    if bytes.len()>1_048_576 { return Err("request exceeded 1 MiB while reading".to_owned()); }
    let (manifest,files)=build_model_set(&bytes).map_err(|e|e.to_string())?;
    let output_dir=PathBuf::from(output_dir);
    fs::create_dir_all(&output_dir).map_err(|e|format!("output directory: {e}"))?;
    for (name,contents) in files {
        fs::write(output_dir.join(name),contents).map_err(|e|format!("model write: {e}"))?;
    }
    let manifest_bytes=to_vec_pretty(&manifest).map_err(|e|format!("manifest serialization: {e}"))?;
    fs::write(output_dir.join("manifest.json"),&manifest_bytes).map_err(|e|format!("manifest write: {e}"))?;
    println!("source_sha256={}",manifest.source_sha256);
    println!("manifest_sha256={}",sha256(&manifest_bytes));
    println!("model_count={}",manifest.model_count);
    println!("{}",String::from_utf8(manifest_bytes).map_err(|e|e.to_string())?);
    Ok(ExitCode::SUCCESS)
}

fn main()->ExitCode{
    match run(){
        Ok(code)=>code,
        Err(error)=>{eprintln!("{error}");ExitCode::from(66)}
    }
}
