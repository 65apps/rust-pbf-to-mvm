extern crate hyper;
extern crate zip;

use hyper::client::Client;
use hyper::header::parsing::*;
use std::fs::{File, read_dir};
use std::io::{BufWriter, BufReader};
use std::io::prelude::*;
use std::process::Command;
use std::env;

struct Target {	
	files: String,	
	dir: String,
	generator: String,
}

struct Russia<'a> {
	url: &'a str,
}

trait Genetare<'a, 'b> {
	fn get_osm(&self);

	fn split_to_region(&self) -> std::io::Result<()>;	

	fn convert_mvm(&self, polygon: &'b str);

	fn zip_file(&self, polygon: &'b str) -> zip::result::ZipResult<()>;

	fn read_env(&self) -> Target;
}

impl<'a> Russia<'a> {
	fn new(url: &'a str) -> Russia<'a> {
		Russia {
			url: url,			
		}
	}
}


impl<'a, 'b> Genetare<'a, 'b> for Russia<'a> {

	fn get_osm(&self) {		
		println!("start download {:?}", self.url);		

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

		let file = File::create("Russia.osm.pbf").unwrap();
	    let mut buffer_write = BufWriter::new(file);
	    let mut buffer_read = BufReader::new(responce);
	    let mut download: u64 = 0;   	    	

	    while download != size {
	    	let length = {	
	    		let buffer = buffer_read.fill_buf().unwrap();	
	    		
	    		buffer_write.write(buffer).unwrap();
				buffer_write.flush().unwrap();

				buffer.len()
	    	};

	    	buffer_read.consume(length);
	    	println!("downloaded bytes {:?} from {:?})", download, size);
	    	download += length as u64;
	    }
	}

	fn split_to_region(&self) -> std::io::Result<()> {
		for entry in try!(read_dir("polygons")) {
    		let dir = try!(entry);

    		let polygon_dir_path = dir.path();
    		let polygon_dir_str = match polygon_dir_path.to_str() {
    			Some(r) => r,			    
			    None => "",
    		};

    		let polygon_dir_arg = String::from("file=") + polygon_dir_str;
    		let mut file_name_arg = polygon_dir_arg.replace("polygons/", "");
    		file_name_arg = file_name_arg.replace("poly", "pbf");

    		let arguments = ["--read-pbf", "Russia.osm.pbf", "--bounding-polygon", polygon_dir_arg.as_str(), "--write-pbf", file_name_arg.as_str()];

			let split_proc = Command::new("osmosis/bin/osmosis")	        
	        	.args(&arguments).output().unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

        	println!("status: {}", split_proc.status);
			println!("stdout: {}", String::from_utf8_lossy(&split_proc.stdout));
			println!("stderr: {}", String::from_utf8_lossy(&split_proc.stderr));	  

			self.convert_mvm(polygon_dir_str);
			self.zip_file(polygon_dir_str);			
		}		
	 	Ok(())				
	}
	
	fn convert_mvm(&self, polygon: &'b str) {
		let var = self.read_env();	

		let polygons_path = var.generator.to_owned() + "polygons";
		let coasts_path = var.generator.to_owned() + "WorldCoasts.geom";
		let border_path = var.generator.to_owned() + polygon;

  		env::set_var("BORDERS_PATH", &polygons_path);
        env::set_var("COASTS", &coasts_path);
        env::set_var("BORDER", &border_path);        

        let mut pbf_file = polygon.replace("polygons/", "");
        pbf_file = pbf_file.replace("poly", "pbf");
        pbf_file = var.generator.to_owned() + &pbf_file;
        
        let arguments = [pbf_file, String::from("asd")];
        
        let mvm_proc = Command::new("omim/tools/unix/generate_mwm.sh").current_dir(var.dir)	        
	        .args(&arguments).output().unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

		println!("status: {}", mvm_proc.status);
		println!("stdout: {}", String::from_utf8_lossy(&mvm_proc.stdout));
		println!("stderr: {}", String::from_utf8_lossy(&mvm_proc.stderr));
	}

	fn zip_file(&self, polygon: &'b str) -> zip::result::ZipResult<()> {
		let mut zip_name = polygon.replace("polygons/", "");
		zip_name = zip_name.replace("poly", "zip");
		let mwm_file = zip_name.replace("zip", "mwm");		
		
		let file = File::create(zip_name.clone()).unwrap();

		let mut zip = zip::ZipWriter::new(file);
		try!(zip.start_file(mwm_file.clone(), zip::CompressionMethod::Deflated));	

		let mut mwm = try!(File::open(&mwm_file));
		let mut buffer_read = BufReader::new(mwm);	

		let mut condition: usize = 1;		

		while condition != 0 {
			let length = {	
	    		let buffer = buffer_read.fill_buf().unwrap();	

	    		zip.write_all(buffer);
				buffer.len()
	    	};

	    	buffer_read.consume(length);    		
    		condition = length;
		}
	    
		try!(zip.finish());	

		let var = self.read_env();	
		let arguments = [zip_name, var.files];		
		let mv_proc = Command::new("mv")	        
	        .args(&arguments).output().unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

        println!("status: {}", mv_proc.status);
		println!("stdout: {}", String::from_utf8_lossy(&mv_proc.stdout));
		println!("stderr: {}", String::from_utf8_lossy(&mv_proc.stderr));
        
		Ok(())
	}

	fn read_env(&self) -> Target {		
		let files_var: &str = "FILES_DIR";		
		let dir_var: &str = "DIR";
		let generator_var: &str = "GENERATOR";

		let files = match env::var(files_var) {
			Err(_) => panic!("error read env FILES_DIR"),
		    Ok(val) => val,		    
		};

		let dir = match env::var(dir_var) {
			Err(_) => panic!("error read env DIR"),
		    Ok(val) => val,		    
		};

		let generator = match env::var(generator_var) {
			Err(_) => panic!("error read env GENERATOR"),
		    Ok(val) => val,		    
		};

		Target {			
			files: files,	
			dir: dir,	
			generator: generator,
		}
	}	
}


fn main() {    					
	let russia = Russia::new("http://data.gis-lab.info/osm_dump/dump/latest/RU.osm.pbf");	
	russia.get_osm();
	russia.split_to_region();	
}