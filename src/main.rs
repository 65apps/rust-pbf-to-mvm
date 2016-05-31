extern crate hyper;

use hyper::client::Client;
use hyper::header::parsing::*;
use std::fs::{File, read_dir};
use std::io::{BufWriter, BufReader};
use std::io::prelude::*;
use std::process::Command;
use std::env;

struct Target {	
	files: String,	
}

struct Russia<'a> {
	url: &'a str,
}

trait Genetare<'a, 'b> {
	fn get_osm(&self);

	fn split_to_region(&self) -> std::io::Result<()>;	

	fn convert_mvm(&self, polygon: &'b str);

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

	    while  download != size {
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
		}		
	 	Ok(())				
	}


	
	fn convert_mvm(&self, polygon: &'b str) {
		let env = self.read_env();		
		
		// let whitespace: &str = " ";
		// let poly = self.name.replace("pbf", "poly");							
		
		// let path = env::current_dir().unwrap();
  //       let current_dir = match path.to_str()  {
  //           Some(v) => v.to_string() + "/",
  //           None => String::from(""),
  //       };

  		env::set_var("BORDERS_PATH", "polygons");
        env::set_var("COASTS", "WorldCoasts.geom");
        env::set_var("BORDER", polygon);
        env::set_var("TARGET", &env.files);

        let mut pbf_file = polygon.replace("polygons/", "");
        pbf_file = pbf_file.replace("poly", "pbf");
        
        let arguments = [pbf_file, String::from("asd")];
        
        let mvm_proc = Command::new("omim/tools/unix/generate_mwm.sh")	        
	        .args(&arguments).output().unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

		println!("status: {}", mvm_proc.status);
		println!("stdout: {}", String::from_utf8_lossy(&mvm_proc.stdout));
		println!("stderr: {}", String::from_utf8_lossy(&mvm_proc.stderr));

		// let old_file = self.name.replace("pbf", "mwm");
  //       let new_mwm_file = old_file.replace("-", whitespace);        

  //       match std::fs::rename(env.files.clone() + &old_file, env.files.clone()+ &new_mwm_file) {
  //           Err(e) => panic!("error rename mwm file - {:?}", e),
  //           Ok(_) => (),
  //       }

		// let mut dir = env::current_dir().unwrap();
		// dir.push(self.name);		
		// let graph_proc = Command::new("./graphhopper.sh")
		// 	.current_dir(&env.graph)							
		// 	.arg("import").arg(&dir).output().unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

		// println!("status: {}", graph_proc.status);
		// println!("stdout: {}", String::from_utf8_lossy(&graph_proc.stdout));
		// println!("stderr: {}", String::from_utf8_lossy(&graph_proc.stderr));		
		
		// let origin_file: &str = match dir.to_str() {
		// 	Some(val) => val,
		// 	None => panic!("not found origin file"),
		// };

		// let graph_file = origin_file.replace(".pbf", "-gh");		
		// let mut new_graph_file = self.name.replace("-", whitespace);
		// new_graph_file = new_graph_file.replace(".pbf", "-gh");
		// new_graph_file = env.files + &new_graph_file;

		// let mv_proc = Command::new("mv")							
		// 	.arg(graph_file).arg(new_graph_file).output().unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });		
		
		// println!("status: {}", mv_proc.status);
		// println!("stdout: {}", String::from_utf8_lossy(&mv_proc.stdout));
		// println!("stderr: {}", String::from_utf8_lossy(&mv_proc.stderr));	
	}

	fn read_env(&self) -> Target {		
		let files_var: &str = "FILES_DIR";		

		let files = match env::var(files_var) {
			Err(_) => panic!("error read env FILES_DIR"),
		    Ok(val) => val,		    
		};

		Target {			
			files: files,			
		}
	}	
}



fn main() {    					
	let russia = Russia::new("http://data.gis-lab.info/osm_dump/dump/latest/RU.osm.pbf");
	// russia.get_osm();
	russia.split_to_region();

	// let crimea = District::new("http://download.geofabrik.de/russia/crimean-fed-district-latest.osm.pbf", "Crimea.pbf", "http://download.geofabrik.de/russia/crimean-fed-district.poly");

	// let altai_krai = "Russia_Altai Krai.poly";
	// let altai_repub = "Russia_Altai Republic.poly";
	// let amur_obl = "Russia_Amur Oblast.poly";
	// let arkhangelsk_obl_central = "Russia_Arkhangelsk Oblast_Central.poly";
	// let arkhangelsk_obl_north = "Russia_Arkhangelsk Oblast_North.poly";
	// let astrakhan_obl = "Russia_Astrakhan Oblast.poly";
	// let bashkortostan = "Russia_Bashkortostan.poly";
	// let belgorod_obl = "Russia_Belgorod Oblast.poly";
	// let bryansk_obl = "Russia_Bryansk Oblast.poly";
	// let buryatia = "Russia_Buryatia.poly";
	// let chechen_rebub = "Russia_Chechen Republic.poly";
	// let chelyabinsk_obl = "Russia_Chelyabinsk Oblast.poly";
	// let chukotka_autonomous_okrug = "Russia_Chukotka Autonomous Okrug.poly";
	// let chuvashia = "Russia_Chuvashia.poly";
	// let ingushetia = "Russia_Ingushetia.poly";
	// let irkutsk_obl = "Russia_Irkutsk Oblast.poly";
	// let ivanovo_obl = "Russia_Ivanovo Oblast.poly";
	// let jewish_autonomous_okrug = "Russia_Ivanovo Oblast.poly";
	// let kabardino_balkaria = "Russia_Kabardino-Balkaria.poly";
	// let kaliningrad_obl = "Russia_Kaliningrad Oblast.poly";
	// let kaluga_obl = "Russia_Kaluga Oblast.poly";
	// let kamchatka_krai = "Russia_Kamchatka Krai.poly";
	// let karachay_cherkessia = "Russia_Karachay-Cherkessia.poly";
	// let kemerov_obl = "Russia_Kemerov Oblast.poly";
	// let khabarovsk_krai = "Russia_Khabarovsk Krai.poly";
	// let khakassia = "Russia_Khakassia.poly";
	// let kirov_obl = "Russia_Kirov Oblast.poly";
	// let komi_repub = "Russia_Komi Republic.poly";
	// let kostroma_obl = "Russia_Kostroma Oblast.poly";
	// let krasnodar_krai = "Russia_Krasnodar Krai.poly";
	// let krasnodar_krai_adygeya = "Russia_Krasnodar Krai_Adygeya.poly";
	// let krasnoyarsk_krai_north = "Russia_Krasnoyarsk Krai_North.poly";
	// let krasnoyarsk_krai_south = "Russia_Krasnoyarsk Krai_South.poly";
	// let kurgan_obl = "Russia_Kurgan Oblast.poly";
	// let kursk_obl = "Russia_Kursk Oblast.poly";
	// let leningrad_obl_karelsky = "Russia_Leningradskaya Oblast_Karelsky.poly";
	// let leningrad_obl_south = "Russia_Leningradskaya Oblast_Southeast.poly";
	// let lipetsk_obl = "Russia_Lipetsk Oblast.poly";
	// let magadan_obl = "Russia_Magadan Oblast.poly";
	// let mari_el = "Russia_Mari El.poly";
	// let moscow_obl_east = "Russia_Moscow Oblast_East.poly";
	// let moscow_obl_west = "Russia_Moscow Oblast_West.poly";
	// let moscow = "Russia_Moscow.poly";

	// let array = [crimea, northcaucasus, central, fareastern, northwestern, siberian, south, ural, volga];
	// for x in array.iter() {				
	//     x.convert_mvm_and_graph(); 
	// }
}
