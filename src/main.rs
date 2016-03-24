extern crate hyper;

use hyper::client::Client;
use hyper::header::parsing::*;
use std::fs::File;
use std::io::{BufWriter, BufReader};
use std::io::prelude::*;
use std::process::Command;
use std::env;
use std::env::VarError;

enum Src {
    None,
    Path(&'static str),
}

struct Mvm {
    source: &'static str,
    file: Src
}

trait Genetare {
	fn get_source(&mut self);

	fn convert(&self);

	fn read_env(&self) -> String;
}

impl Genetare for Mvm {
	fn get_source(&mut self) {
		let client = Client::new();	
		let mut responce = client.get(self.source).send().unwrap();
		
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

    	let mut file = File::create(name).unwrap();
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

	fn convert(&self) {
		let file = match self.file {
			Src::None => panic!("no file found"),
			Src::Path(file) => file,
		};

		let path = self.read_env();		
		let output = Command::new(path)
							.arg(file)
							.output().unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

		println!("status: {}", output.status);
		println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
		println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
	}

	fn read_env(&self) -> String {
		static OMIM: &'static str = "OMIM_DIR";
		match env::var(OMIM) {
			Err(e) => panic!("error read env {:?}", e),
		    Ok(val) => val,		    
		}
	}
}

static CRIMEAN: &'static str = "http://download.geofabrik.de/russia/crimean-fed-district-latest.osm.pbf";

fn main() {    
    let mut central = Mvm {
    	source: CRIMEAN,
    	file: Src::None
    };

    central.get_source();
    central.convert(); 
}
