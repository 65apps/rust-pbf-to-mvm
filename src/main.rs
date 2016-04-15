extern crate hyper;

use hyper::client::Client;
use hyper::header::parsing::*;
use std::fs::File;
use std::io::{BufWriter, BufReader};
use std::io::prelude::*;
use std::process::Command;
use std::env;
use std::str;

struct Target {
	omim: String,
	files: String,
	graph: String
}

struct District<'a, 'b> {
	url: &'a str,
	name: &'b str,
}

trait Genetare<'a, 'b> {
	fn get_osm(&self);

	fn convert_mvm_and_graph(&self);

	fn read_env(&self) -> Target;
}

impl<'a, 'b> District<'a, 'b> {
	fn new(url: &'a str, name: &'b str) -> District<'a, 'b> {
		District {
			url: url,
			name: name
		}
	}
}

impl<'a, 'b> Genetare<'a, 'b> for District<'a, 'b> {

	fn get_osm(&self) {
		println!("start convert {:?}", self.url);
		let temp = self.read_env();

		let client = Client::new();	
		let responce = client.get(self.url).send().unwrap();
		
		assert_eq!(hyper::Ok, responce.status);		

		let size: u64;

		{
			let length: &[Vec<u8>] = responce.headers.get_raw("content-length").unwrap();

		 	size = match from_one_raw_str(&length) {
	    		Err(_) => panic!("cannot read header"),
	    		Ok(size) => size,
	    	};
		}

		let file = File::create(self.name).unwrap();
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
	    	println!("downloaded bytes {:?} from {:?} (File - {:?})", download, size, self.name);
	    	download += length as u64;
	    }
	}

	fn convert_mvm_and_graph(&self) {
		let env = self.read_env();						
		let mvm_proc = Command::new(env.omim)
							.env("TARGET", &env.files)
							.arg(self.name).output().unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

		println!("status: {}", mvm_proc.status);
		println!("stdout: {}", String::from_utf8_lossy(&mvm_proc.stdout));
		println!("stderr: {}", String::from_utf8_lossy(&mvm_proc.stderr));

		let mut dir = env::current_dir().unwrap();
		dir.push(self.name);		
		let graph_proc = Command::new("./graphhopper.sh")
							.current_dir(&env.graph)							
							.arg("import").arg(&dir).output().unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

		println!("status: {}", graph_proc.status);
		println!("stdout: {}", String::from_utf8_lossy(&graph_proc.stdout));
		println!("stderr: {}", String::from_utf8_lossy(&graph_proc.stderr));		
		
		let origin_file: &str = match dir.to_str() {
			Some(val) => val,
			None => panic!("not found origin file"),
		};

		let graph_file = str::replace(origin_file, ".pbf", "-gh");		
		let mv_proc = Command::new("mv")							
							.arg(graph_file).arg(&env.files).output().unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
		
		println!("status: {}", mv_proc.status);
		println!("stdout: {}", String::from_utf8_lossy(&mv_proc.stdout));
		println!("stderr: {}", String::from_utf8_lossy(&mv_proc.stderr));		
	}

	fn read_env(&self) -> Target {
		let omim_var: &str = "OMIM_DIR";
		let files_var: &str = "FILES_DIR";
		let graph_var: &str = "GRAPH_DIR";

		let omim = match env::var(omim_var) {
			Err(_) => panic!("error read env OMIM_DIR"),
		    Ok(val) => val,		    
		};

		let files = match env::var(files_var) {
			Err(_) => panic!("error read env FILES_DIR"),
		    Ok(val) => val,		    
		};

		let graph = match env::var(graph_var) {
		    Err(_) => panic!("error read env GRAPH_DIR"),
		    Ok(val) => val,
		};

		Target {
			omim: omim,
			files: files,
			graph: graph
		}
	}
}



fn main() {    	
	
	let northwestern: &str = "http://download.geofabrik.de/russia/northwestern-fed-district-latest.osm.pbf";
	let siberian: &str = "http://download.geofabrik.de/russia/siberian-fed-district-latest.osm.pbf";
	let south: &str = "http://download.geofabrik.de/russia/south-fed-district-latest.osm.pbf";
	let ural: &str = "http://download.geofabrik.de/russia/ural-fed-district-latest.osm.pbf";
	let volga: &str = "http://download.geofabrik.de/russia/volga-fed-district-latest.osm.pbf";

	let crimea = District::new("http://download.geofabrik.de/russia/crimean-fed-district-latest.osm.pbf", "Crimea.pbf");
	let northcaucasus = District::new("http://download.geofabrik.de/russia/north-caucasus-fed-district-latest.osm.pbf", "Russia_North Caucasian.pbf");
	let central = District::new("http://download.geofabrik.de/russia/central-fed-district-latest.osm.pbf", "Russia_Central.pbf");
	let fareastern = District::new("http://download.geofabrik.de/russia/far-eastern-fed-district-latest.osm.pbf", "Russia_Far Eastern.pbf");


	let array = [northcaucasus];
	for x in array.iter() {		
		x.get_osm();	    
	    x.convert_mvm_and_graph(); 
	}
}
