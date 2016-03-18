extern crate hyper;

use hyper::client::Client;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Debug)]
struct Mvm<'a> {
    source: &'a str
}

impl<'a> Mvm<'a> {
	fn get_source(&self) {
		let client = Client::new();	
		let mut res = client.get(self.source).send().unwrap();
		
		assert_eq!(hyper::Ok, res.status);

		let mut file = match File::create("temp.pbf") {
	        Err(_) => panic!("couldn't create"),
	        Ok(file) => file,
	    };

		let length = res.headers.get_raw("content-length");
	    println!("{:?}", length); 

	    // loop {
	    //     let mut buffer = [0; 1024];
	    // 	res.read(&mut buffer).unwrap();
	    // 	file.write(&buffer).unwrap();	    	
	    // }

		
		// for byte in res.bytes() {
		// 	let chunk = match byte {
		// 		Err(_) => panic!("error read"),
		// 		Ok(data) => data,
		// 	};

		// 	match file.write(&chunk) {
		// 		Err(_) => panic!("error write"),
		// 		Ok(size) => println!("write {} bytes", size),
		// 	}
		// }
	}
}

static CENTRAL: &'static str = "http://download.geofabrik.de/russia/crimean-fed-district-latest.osm.pbf";

fn main() {    
    let central = Mvm {
    	source: CENTRAL
    };

    central.get_source();
}
