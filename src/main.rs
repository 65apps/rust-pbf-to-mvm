extern crate hyper;

use hyper::client::Client;
use hyper::header::parsing::*;
use std::fs::File;
use std::io::{BufWriter, BufReader};
use std::io::prelude::*;
use std::process::Command;
use std::env;

struct Target {
	omim: String,
	files: String,
	graph: String
}

struct District<'a> {
	url: &'a str,
	name: &'a str,
	poly: &'a str,
}

trait Genetare<'a> {
	fn get_osm(&self);

	fn get_poly(&self);

	fn convert_mvm_and_graph(&self);

	fn read_env(&self) -> Target;
}

impl<'a> District<'a> {
	fn new(url: &'a str, name: &'a str, poly: &'a str) -> District<'a> {
		District {
			url: url,
			name: name,
			poly: poly,
		}
	}
}

impl<'a> Genetare<'a> for District<'a> {

	fn get_osm(&self) {
		println!("start convert {:?}", self.url);		

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

	fn get_poly(&self) {
		let client = Client::new();	
		let responce = client.get(self.poly).send().unwrap();
		
		assert_eq!(hyper::Ok, responce.status);		

		let size: u64;

		{
			let length: &[Vec<u8>] = responce.headers.get_raw("content-length").unwrap();

		 	size = match from_one_raw_str(&length) {
	    		Err(_) => panic!("cannot read header"),
	    		Ok(size) => size,
	    	};
		}

		let file = File::create(self.name.replace("pbf", "poly")).unwrap();
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
		self.get_osm();
		self.get_poly();			

		let whitespace: &str = " ";
		let poly = self.name.replace("pbf", "poly");	
		println!("{:?}", poly);					

		let mvm_proc = Command::new(env.omim)							
							.env("COASTS", "WorldCoasts.geom")
							.env("BORDER", &poly)
							.env("TARGET", &env.files)
							.arg(self.name).output().unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

		println!("status: {}", mvm_proc.status);
		println!("stdout: {}", String::from_utf8_lossy(&mvm_proc.stdout));
		println!("stderr: {}", String::from_utf8_lossy(&mvm_proc.stderr));

		let old_file = self.name.replace("pbf", "mwm");
        let new_mwm_file = old_file.replace("-", whitespace);        

        match std::fs::rename(env.files.clone() + &old_file, env.files.clone()+ &new_mwm_file) {
            Err(e) => panic!("error rename mwm file - {:?}", e),
            Ok(_) => (),
        }

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

		let graph_file = origin_file.replace(".pbf", "-gh");		
		let mut new_graph_file = self.name.replace("-", whitespace);
		new_graph_file = new_graph_file.replace(".pbf", "-gh");
		new_graph_file = env.files + &new_graph_file;

		let mv_proc = Command::new("mv")							
							.arg(graph_file).arg(new_graph_file).output().unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
		
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
	let crimea = District::new("http://download.geofabrik.de/russia/crimean-fed-district-latest.osm.pbf", "Crimea.pbf", "http://download.geofabrik.de/russia/crimean-fed-district.poly");
	// let northcaucasus = District::new("http://download.geofabrik.de/russia/north-caucasus-fed-district-latest.osm.pbf", "Russia_North-Caucasian.pbf");
	// let central = District::new("http://download.geofabrik.de/russia/central-fed-district-latest.osm.pbf", "Russia_Central.pbf");
	// let fareastern = District::new("http://download.geofabrik.de/russia/far-eastern-fed-district-latest.osm.pbf", "Russia_Far-Eastern.pbf");
	// let northwestern = District::new("http://download.geofabrik.de/russia/northwestern-fed-district-latest.osm.pbf", "Russia_Northwestern.pbf");
	// let siberian = District::new("http://download.geofabrik.de/russia/siberian-fed-district-latest.osm.pbf", "Russia_Siberian.pbf");
	// let south = District::new("http://download.geofabrik.de/russia/south-fed-district-latest.osm.pbf", "Russia_Southern.pbf");
	// let ural = District::new("http://download.geofabrik.de/russia/ural-fed-district-latest.osm.pbf", "Russia_Urals.pbf");
	// let volga = District::new("http://download.geofabrik.de/russia/volga-fed-district-latest.osm.pbf", "Russia_Volga.pbf");

	let array = [crimea];
	for x in array.iter() {				
	    x.convert_mvm_and_graph(); 
	}
}
