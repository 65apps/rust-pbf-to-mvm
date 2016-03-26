extern crate hyper;

use hyper::client::Client;
use hyper::header::parsing::*;
use std::fs::File;
use std::io::{BufWriter, BufReader};
use std::io::prelude::*;
use std::process::Command;
use std::env;

enum Src<'a> {
    None,
    Path(&'a str),
}

struct Mvm<'a> {
    source: &'a str,
    file: Src<'a>
}

trait Genetare<'a> {
	fn get_source(&mut self);

	fn convert_mvm(&self);

	fn read_env(&self) -> Target;
}

struct Target {
	omim: String,
	files: String
}

impl<'a> Genetare<'a> for Mvm<'a> {

	fn get_source(&mut self) {
		let client = Client::new();	
		let responce = client.get(self.source).send().unwrap();
		
		assert_eq!(hyper::Ok, responce.status);		

		let size: u64;

		{
			let length: &[Vec<u8>] = responce.headers.get_raw("content-length").unwrap();

		 	size = match from_one_raw_str(&length) {
	    		Err(_) => panic!("cannot read header"),
	    		Ok(size) => size,
	    	};
		}


		let vec: Vec<&str> = self.source.split('/').collect();
		let name: &str = vec[vec.len()-1];	

		self.file = Src::Path(name);		

    	let file = File::create(name).unwrap();
	    let mut buffer_write = BufWriter::new(file);
	    let mut buffer_read = BufReader::new(responce);
	    let mut download: u64 = 0;   	    	

	    while  download != size {
	    	let length = {	
	    		let buffer = buffer_read.fill_buf().unwrap();	
	    		
	    		buffer_write.write(buffer).unwrap();
				buffer_write.flush().unwrap();

				buffer.len()
	    	};

	    	buffer_read.consume(length);
	    	println!("{:?}", length);
	    	download += length as u64;
	    }
	}

	fn convert_mvm(&self) {
		let file = match self.file {
			Src::None => panic!("no file found"),
			Src::Path(file) => file,
		};

		let env = self.read_env();				
		let output = Command::new(env.omim)
							.env("TARGET", env.files)
							.arg(file).output().unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

		println!("status: {}", output.status);
		println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
		println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

	}

	fn read_env(&self) -> Target {
		let omim_var: &str = "OMIM_DIR";
		let files_var: &str = "FILES_DIR";

		let omim = match env::var(omim_var) {
			Err(_) => panic!("error read env OMIM_DIR"),
		    Ok(val) => val,		    
		};

		let files = match env::var(files_var) {
			Err(_) => panic!("error read env FILES_DIR"),
		    Ok(val) => val,		    
		};

		Target {
			omim: omim,
			files: files
		}
	}
}

fn main() {    
	let crimean: &str = "http://download.geofabrik.de/russia/crimean-fed-district-latest.osm.pbf";

    let mut central = Mvm {
    	source: crimean,
    	file: Src::None
    };

    central.get_source();
    central.convert_mvm(); 

}
