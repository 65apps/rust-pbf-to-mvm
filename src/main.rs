extern crate hyper;

use hyper::client::Client;
use hyper::header::parsing::*;
use std::fs::File;
use std::io::{BufWriter, BufReader};
use std::io::prelude::*;
use std::process::Command;
use std::env;

struct Target {	
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
		
		let path = env::current_dir().unwrap();
        let current_dir = match path.to_str()  {
            Some(v) => v.to_string() + "/",
            None => String::from(""),
        };

        env::set_var("COASTS", "WorldCoasts.geom");
        env::set_var("BORDER", &poly);
        env::set_var("TARGET", &env.files);

        let mvm_proc = Command::new("omim/tools/unix/generate_mwm.sh")	        
	        .arg(current_dir.clone() + &self.name).output().unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

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
		let files_var: &str = "FILES_DIR";
		let graph_var: &str = "GRAPH_DIR";

		let files = match env::var(files_var) {
			Err(_) => panic!("error read env FILES_DIR"),
		    Ok(val) => val,		    
		};

		let graph = match env::var(graph_var) {
		    Err(_) => panic!("error read env GRAPH_DIR"),
		    Ok(val) => val,
		};

		Target {			
			files: files,
			graph: graph
		}
	}
}



fn main() {    					


	// let crimea = District::new("http://download.geofabrik.de/russia/crimean-fed-district-latest.osm.pbf", "Crimea.pbf", "http://download.geofabrik.de/russia/crimean-fed-district.poly");

	// let altai_krai = District::new("", "Russia_Altai-Krai.pbf");
	// let altai_repub = District::new("", "Russia_Altai-Republic.pbf");
	// let amur_obl = District::new("", "Russia_Amur-Oblast.pbf");
	// let arkhangelsk_obl_central = District::new("", "Russia_Arkhangelsk Oblast_Central.pbf");
	// let arkhangelsk_obl_north = District::new("", "Russia_Arkhangelsk Oblast_North.pbf");
	// let astrakhan_obl = District::new("", "Russia_Astrakhan Oblast.pbf");
	// let bashkortostan = District::new("", "Russia_Bashkortostan.pbf");
	// let belgorod_obl = District::new("", "Russia_Belgorod Oblast.pbf");
	// let bryansk_obl = District::new("", "Russia_Bryansk Oblast.pbf");
	// let buryatia = District::new("", "Russia_Buryatia.pbf");
	// let chechen_rebub = District::new("", "Russia_Chechen Republic.pbf");
	// let chelyabinsk_obl = District::new("", "Russia_Chelyabinsk Oblast.pbf");
	// let chukotka_autonomous_okrug = District::new("", "Russia_Chukotka Autonomous Okrug.pbf");
	// let chuvashia = District::new("", "Russia_Chuvashia.pbf");
	// let ingushetia = District::new("", "Russia_Ingushetia.pbf");
	// let irkutsk_obl = District::new("", "Russia_Irkutsk Oblast.pbf");
	// let ivanovo_obl = District::new("", "Russia_Ivanovo Oblast.pbf");
	// let jewish_autonomous_okrug = District::new("", "Russia_Ivanovo Oblast.pbf");
	// let kabardino_balkaria = District::new("", "Russia_Kabardino-Balkaria.pbf");
	// let kaliningrad_obl = District::new("", "Russia_Kaliningrad Oblast.pbf");
	// let kaluga_obl = District::new("", "Russia_Kaluga Oblast.pbf");
	// let kamchatka_krai = District::new("", "Russia_Kamchatka Krai.pbf");
	// let karachay_cherkessia = District::new("", "Russia_Karachay-Cherkessia.pbf");
	// let kemerov_obl = District::new("", "Russia_Kemerov Oblast.pbf");
	// let khabarovsk_krai = District::new("", "Russia_Khabarovsk Krai.pbf");
	// let khakassia = District::new("", "Russia_Khakassia.pbf");
	// let kirov_obl = District::new("", "Russia_Kirov Oblast.pbf");
	// let komi_repub = District::new("", "Russia_Komi Republic.pbf");
	// let kostroma_obl = District::new("", "Russia_Kostroma Oblast.pbf");
	// let krasnodar_krai = District::new("", "Russia_Krasnodar Krai.pbf");
	// let krasnodar_krai_adygeya = District::new("", "Russia_Krasnodar Krai_Adygeya.pbf");
	// let krasnoyarsk_krai_north = District::new("", "Russia_Krasnoyarsk Krai_North.pbf");
	// let krasnoyarsk_krai_south = District::new("", "Russia_Krasnoyarsk Krai_South.pbf");
	// let kurgan_obl = District::new("", "Russia_Kurgan Oblast.pbf");
	// let kursk_obl = District::new("", "Russia_Kursk Oblast.pbf");
	// let leningrad_obl_karelsky = District::new("", "Russia_Leningradskaya Oblast_Karelsky.pbf");
	// let leningrad_obl_south = District::new("", "Russia_Leningradskaya Oblast_Southeast.pbf");
	// let lipetsk_obl = District::new("", "Russia_Lipetsk Oblast.pbf");
	// let magadan_obl = District::new("", "Russia_Magadan Oblast.pbf");
	// let mari_el = District::new("", "Russia_Mari El.pbf");
	// let moscow_obl_east = District::new("", "Russia_Moscow Oblast_East.pbf");
	// let moscow_obl_west = District::new("", "Russia_Moscow Oblast_West.pbf");
	// let moscow = District::new("", "Russia_Moscow.pbf");



	

	// let array = [crimea, northcaucasus, central, fareastern, northwestern, siberian, south, ural, volga];
	// for x in array.iter() {				
	//     x.convert_mvm_and_graph(); 
	// }
}
