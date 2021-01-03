use serde::{Serialize, Deserialize};
use std::fs;
use std::io;

const MIRRORS_LIST_PATH: &str = "/etc/blimp/mirrors";

#[derive(Serialize, Deserialize)]
struct Mirror {
	url: String,
	// TODO key
}

struct MirrorInfo {
	name: String,
	owner: String,
	uptime: usize,
}

impl Mirror {
	fn new(url: &String) -> Self {
		Self {
			url: url.clone(),
		}
	}

	fn get_infos(&self) -> Result<MirrorInfo, u32> {
		// TODO
		Err(0)
	}
}

fn open_list() -> Result<String, io::Error> {
	fs::read_to_string(MIRRORS_LIST_PATH)
}

pub fn parse_list() -> Result<Vec<Mirror>, String> {
	if let Ok(data) = open_list() {
		let j = serde_json::from_str(&data);
		if j.is_ok() {
			let list: Vec<Mirror> = j.unwrap();
			Ok(list)
		} else {
			Err("Mirrors list parse error".to_string())
		}
	} else {
		Err("Failed to open mirrors list file".to_string())
	}
}
