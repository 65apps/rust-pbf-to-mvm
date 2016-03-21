extern crate hyper;

use hyper::client::Client;
use hyper::header::parsing::*;
use std::fs::File;
use std::io::{BufWriter, BufReader};
use std::io::prelude::*;



#[derive(Debug)]
struct Mvm {
    source: &'static str,
    file: 
}

trait Genetare {
	fn get_source(&self);
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
		self.file = vec[vec.len()-1];		

    // 	let mut file = File::create("temp.pbf").unwrap();
	   //  let mut buffer_write = BufWriter::new(file);
	   //  let mut buffer_read = BufReader::new(responce);
	   //  let mut download: u64 = 0;   	    	

	   //  while  download != size {
	   //  	let length = {	
	   //  		let buffer = buffer_read.fill_buf().unwrap();	
	    		
	   //  		buffer_write.write(buffer).unwrap();
				// buffer_write.flush().unwrap();

				// buffer.len()
	   //  	};

	   //  	buffer_read.consume(length);
	   //  	println!("{:?}", length);
	   //  	download += length as u64;
	   //  }
	}
}

static CRIMEAN: &'static str = "http://download.geofabrik.de/russia/crimean-fed-district-latest.osm.pbf";

fn main() {    
    let central = Mvm {
    	source: CRIMEAN
    };

    central.get_source();
}
