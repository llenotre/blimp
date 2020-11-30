const VERSION: &str = "1.0";

fn print_help() -> i32 {
	println!("usage: blimp ...");
	println!();
	println!("arguments:");
	println!("\t-h, --help: Shows help");
	println!("\t-v, --version: Shows version");
	println!("\t-m, --mirrors: Lists mirrors, checking if they are available");
	println!("\t-i <name>, --install <name>: Installs the package with the given name");
	println!("\t-u [name], --update [name]: Updates a package. If no package name is specified,\
every packages are updated");
	println!("\t-r <name>, --remove <name>: Removes the package with the given name");
	0
}

fn print_version() -> i32 {
	println!("blimp version {}", VERSION);
	0
}

fn list_mirrors() -> i32 {
	// TODO Read mirrors list
	// TODO Call each and check if up
	0
}

fn install_package(_name: &String) -> i32 {
	// TODO Install package
	0
}

fn update_package(_name: &String) -> i32 {
	// TODO Update package
	0
}

fn update_all() -> i32 {
	// TODO Update all packages
	0
}

fn remove_package(_name: &String) -> i32 {
	// TODO Remove package
	0
}

fn main() {
	let args: Vec<String> = std::env::args().collect();
	if args.len() <= 1 {
		print_help();
		return;
	}

	// TODO Handle several packages at once
	let status = match args[1].as_str() {
		"-h" | "--help" => print_help(),
		"-v" | "--version" => print_version(),
		"-m" | "--mirrors" => list_mirrors(),
		"-i" | "--install" => {
			if args.len() == 3 {
				install_package(&args[2])
			} else {
				print_help()
			}
		},
		"-u" | "--update" => {
			if args.len() == 3 {
				update_package(&args[2])
			} else {
				update_all()
			}
		},
		"-r" | "--remove" => {
			if args.len() == 3 {
				remove_package(&args[2])
			} else {
				print_help()
			}
		},
		_ => print_help(),
	};
	std::process::exit(status);
}
