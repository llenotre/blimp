//! A build descriptor defines how to build a package.
//!
//! A build descriptor contains general informations about the package, but also sources for files
//! used for building the package.
//!
//! Source files may come from different sources. See [`SourceInner`].
//!
//! A tarball is a compressed file containing sources for a package.
//! Tarballs may contain a single directory in which all files are present. "Unwrapping" is the
//! action of moving all the files out of this directory while decompressing the archive.

use crate::package::Package;
use crate::util;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[cfg(feature = "network")]
use crate::download::DownloadTask;

// TODO add an option to allow fetching a tarball without unwrapping it?

/// Source-type specific fields.
#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SourceInner {
	/// Download a tarball from an URL.
	Url {
		/// The URL of the sources.
		url: String,
		// TODO add optional hash to check tarball
		// TODO add option to fetch hash from URL?
	},
	/// Clone the given repository.
	Git {
		/// The URL to the Git repository.
		git_url: String,
		/// The branch to clone from. If not specified, the default branch is used.
		branch: Option<String>,
	},
	/// Copy from a local path.
	Local {
		/// The path to the local tarball or directory.
		path: PathBuf,
	},
}

/// Description of sources files, where to find them and where to places them for building.
#[derive(Clone, Deserialize, Serialize)]
pub struct Source {
	/// Source-type specific fields.
	#[serde(flatten)]
	inner: SourceInner,

	/// The location relative to the build directory where the source files will be placed.
	location: PathBuf,
	/// Tells whether the files must unwrapped.
	#[serde(default)]
	unwrap: bool,
}

impl Source {
	/// Fetches files from the source and uncompresses them if necessary.
	///
	/// Files are placed into the build directory `build_dir` according to the specified location.
	pub async fn fetch(&self, build_dir: &Path) -> Result<()> {
		let dest_path = util::concat_paths(build_dir, &self.location);

		match &self.inner {
			SourceInner::Local {
				path,
			} => {
				let metadata = fs::metadata(path)?;
				if metadata.is_dir() {
					util::recursive_copy(path, &dest_path)?;
				} else {
					// TODO uncompress only if it is an actual archive
					// Uncompress tarball
					util::uncompress(path, &dest_path, self.unwrap)?;
				}
			}

			#[cfg(not(feature = "network"))]
			_ => panic!("Feature `network` is not enabled! Please recompile blimp common with this feature enabled"),
			#[cfg(feature = "network")]
			_ => {}
		}

		#[cfg(feature = "network")]
		match &self.inner {
			SourceInner::Url {
				url,
			} => {
				// Download
				let (path, _) = util::create_tmp_file()?;
				let mut download_task = DownloadTask::new(url, &path).await?;
				while download_task.next().await? {}

				// TODO check integrity with hash if specified

				// Uncompress the archive
				util::uncompress(&path, &dest_path, self.unwrap)?;

				// TODO remove archive?
			}

			SourceInner::Git {
				git_url,
				branch,
			} => {
				use anyhow::bail;
				use std::process::Command;

				let mut cmd = Command::new("git");
				cmd.arg("clone")
					// Only keep the last commit
					.arg("--depth")
					.arg("1")
					.arg("--single-branch");
				if let Some(branch) = branch {
					cmd.arg("-b");
					cmd.arg(branch);
				}
				let status = cmd.arg(git_url).arg(dest_path).status()?;
				if !status.success() {
					bail!("Cloning from `{git_url}` failed");
				}
			}

			_ => {}
		}

		Ok(())
	}
}

/// Structure describing how to build a package.
#[derive(Deserialize, Serialize)]
pub struct BuildDescriptor {
	/// The list of sources for the package.
	pub sources: Vec<Source>,
	/// The package's descriptor.
	pub package: Package,
}
