//! A remote is a remote host from which packages can be downloaded.

use common::download;
use common::package::Package;
use common::request::PackageListResponse;
use common::request::PackageSizeResponse;
use common::version::Version;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::io;

// TODO Use https

/// The file which contains the list of remotes.
const REMOTES_FILE: &str = "/usr/lib/blimp/remotes_list";
/// The path to the database storing the list of packages for every remotes.
const DATABASE_PATH: &str = "/usr/lib/blimp/database";

/// Structure representing a remote host.
#[derive(Clone)]
pub struct Remote {
    /// The host's address and port (optional).
    host: String,
}

impl Remote {
    /// Creates a new instance.
    pub fn new(host: String) -> Self {
        Self{
            host,
        }
    }

    /// Returns the list of remote hosts.
    /// `sysroot` is the path to the system's root.
    pub fn list(sysroot: &str) -> io::Result<Vec<Self>> {
        let mut v = Vec::new();

        let path = format!("{}/{}", sysroot, REMOTES_FILE);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for l in reader.lines() {
            v.push(Self::new(l?));
        }

        Ok(v)
    }

	/// Adds a new remote.
    /// `sysroot` is the path to the system's root.
	/// `remote` is the remote to add.
	pub fn add(sysroot: &str, remote: &str) -> io::Result<()> {
        let path = format!("{}/{}", sysroot, REMOTES_FILE);
        let file = OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.open(path)?;
        let mut writer = BufWriter::new(file);

		writer.write(remote.as_bytes())?;
		Ok(())
	}

    /// Returns the host for the remote.
    pub fn get_host(&self) -> &str {
        &self.host
    }

    /// Returns the path to the remote's package database.
    /// `sysroot` is the path to the system's root.
    pub fn get_database_path(&self, sysroot: &str) -> String {
        format!("{}/{}/{}", sysroot, DATABASE_PATH, self.get_host())
    }

    /// Returns the remote's motd.
    pub fn get_motd(&self) -> Result<String, String> {
        let url = format!("http://{}/motd", &self.host);
        let response = reqwest::blocking::get(url).or(Err("HTTP request failed"))?;
        let status = response.status();
        let content = response.text().or(Err("HTTP request failed"))?;

        match status {
            reqwest::StatusCode::OK => {
                Ok(content)
            },

            _ => Err(format!("Failed to retrieve motd: {}", status)),
        }
    }

    /// Fetches the list of all the packages from the remote.
    /// `save` tells whether the result of the request must be saved in the database if the request
    /// succeeded.
    /// `sysroot` is the path to the system's root.
    pub fn fetch_all(&self, save: bool, sysroot: &str) -> Result<Vec<Package>, Box<dyn Error>> {
        let url = format!("http://{}/package", &self.host);
        let response = reqwest::blocking::get(url)?;
        let status = response.status();
        let content = response.text()?;

        match status {
            reqwest::StatusCode::OK => {
                let json: PackageListResponse = serde_json::from_str(&content)?;

                if save {
                    let file = File::create(self.get_database_path(sysroot))?;
                    let writer = BufWriter::new(file);
                    serde_json::to_writer_pretty(writer, &json)?;
                }

                Ok(json.packages)
            },

            _ => Err(format!("Failed to retrieve packages list from remote: {}", status).into()),
        }
    }

    /// Returns the package with the given name `name` and version `version`.
    /// If the package doesn't exist on the remote, the function returns None.
    /// `sysroot` is the path to the system's root.
    pub fn get_package(sysroot: &str, name: &str, version: &Version)
        -> io::Result<Option<(Self, Package)>> {
        // Iterating over remotes
        for r in Remote::list(sysroot)? {
            let file = File::open(r.get_database_path(sysroot))?;
            let reader = BufReader::new(file);

            let json: PackageListResponse = serde_json::from_reader(reader)?;

            // Iterating over packages on the remote
            for p in json.packages {
                if p.get_name() == name && p.get_version() == version {
                    return Ok(Some((r, p)));
                }
            }
        }

        Ok(None)
    }

    /// Returns the latest version of the package with name `name`.
    /// If the package doesn't exist, the function returns None.
    /// `sysroot` is the path to the system's root.
    pub fn get_latest(sysroot: &str, name: &String) -> io::Result<Option<(Self, Package)>> {
        // Iterating over remotes
        for r in Remote::list(sysroot)? {
            let file = File::open(r.get_database_path(sysroot))?;
            let reader = BufReader::new(file);

            let json: PackageListResponse = serde_json::from_reader(reader)?;

            // TODO Take highest version
            // Iterating over packages on the remote
            for p in json.packages {
                if p.get_name() == name {
                    return Ok(Some((r, p)));
                }
            }
        }

        Ok(None)
    }

    /// Returns the download size of the package `package` in bytes.
    pub async fn get_size(&self, package: &Package) -> Result<u64, String> {
        let url = format!("http://{}/package/{}/version/{}/size",
            self.host, package.get_name(), package.get_version());
        let response = reqwest::get(url).await
        	.or_else(| e | Err(format!("HTTP request failed: {}", e)))?;
        let content = response.text().await
        	.or_else(| e | Err(format!("HTTP request failed: {}", e)))?;

        let json: PackageSizeResponse = serde_json::from_str(&content)
            .or_else(| e | Err(format!("Failed to parse JSON response: {}", e)))?;
        Ok(json.size)
    }

    // TODO Do not keep the whole file in RAM before writing
    /// Downloads the package `package` and writes it in cache.
    /// `sysroot` is the path to the system's root.
    pub async fn download(&self, sysroot: &str, package: &Package) -> Result<(), Box<dyn Error>> {
        let url = format!("http://{}/package/{}/version/{}/archive",
            self.host, package.get_name(), package.get_version());
		download::download_file(&url, &package.get_cache_path(sysroot)).await
    }
}
