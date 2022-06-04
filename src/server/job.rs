//! TODO doc

use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::get;
use actix_web::post;
use actix_web::web;
use common::version::Version;
use crate::global_data::GlobalData;
use crate::util;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::process::Child;
use std::process::Command;
use std::sync::Mutex;

/// Enumeration of possible job status.
#[derive(Clone, Deserialize, Serialize)]
pub enum JobStatus {
	/// The job is pending for one or several other jobs to finish.
	Pending,
	/// The job is running.
	InProgress,
	/// The job ended successfully.
	Success,
	/// The job failed.
	Failed,
	/// The job was aborted.
	Aborted,
}

/// Structure representing a job description.
#[derive(Deserialize, Serialize)]
pub struct JobDesc {
	/// The job's ID.
	id: String,

	/// The package's name.
	package: String,
	/// The package's version.
	version: Version,

	/// The job's current status.
	status: JobStatus,
}

/// Structure representing a job.
pub struct Job {
	/// The job description.
	desc: JobDesc,

	/// The job's process.
	process: Option<Child>,
}

impl Job {
	/// Returns the job's HTML representation in the jobs list.
	pub fn get_list_html(&self) -> String {
		let id = &self.desc.id;
		let package = &self.desc.package;
		let version = &self.desc.version;
		let status_html = match self.desc.status {
			JobStatus::Pending => "<td class=\"status-progress\">Pending</td>",
			JobStatus::InProgress => "<td class=\"status-progress\">In progress</td>",
			JobStatus::Success => "<td class=\"status-success\">Success</td>",
			JobStatus::Failed => "<td class=\"status-failed\">Failed</td>",
			JobStatus::Aborted => "<td class=\"status-failed\">Aborted</td>",
		};

		format!("<tr>
			<td><a href=\"/dashboard/job/{id}\">#{id}</a></td>
			<td>{package}</td>
			<td>{version}</td>
			{status_html}
		</tr>")
	}

	/// Tells whether the job is in capacity to run.
	/// The function checks that build dependencies are available.
	pub fn can_run(&self) -> bool {
		// TODO
		true
	}

	/// Runs the job.
	/// If the job cannot run, the function does nothing.
	pub fn run(&mut self) -> io::Result<()> {
		if !matches!(self.desc.status, JobStatus::Pending) {
			return Ok(());
		}
		if !self.can_run() {
			return Ok(());
		}

		// TODO Redirect stdout and stderr to logs file
		self.process = Some(Command::new("blimp-builder")
			.args(["TODO", "TODO"]) // TODO
			.spawn()?);
		self.desc.status = JobStatus::InProgress;

		Ok(())
	}

	/// Aborts the job.
	pub fn abort(&mut self) {
		if matches!(self.desc.status, JobStatus::Success | JobStatus::Failed) {
			return;
		}

		// Killing job's processes
		if let Some(child) = &mut self.process {
			let _ = child.kill();
		}

		self.desc.status = JobStatus::Aborted;
	}
}

#[get("/dashboard/job/{id}")]
async fn job_get(
	data: web::Data<Mutex<GlobalData>>,
	web::Path(id): web::Path<String>,
) -> impl Responder {
	let data = data.lock().unwrap();

	let job = data.get_jobs()
		.iter()
		.filter(| j | {
			j.desc.id == id
		}).next();
	let job = match job {
		Some(job) => job,
		None => return HttpResponse::NotFound().finish(),
	};

	// TODO
	HttpResponse::Ok().body("TODO")
}

#[get("/dashboard/job/{id}/logs")]
async fn job_logs(
	data: web::Data<Mutex<GlobalData>>,
	web::Path(id): web::Path<String>,
) -> impl Responder {
	if util::is_correct_job_id(&id) {
		// TODO Put build logs directory name in constant
		let path = format!("job_logs/{}.log", id);
		let logs = fs::read_to_string(&path).unwrap(); // TODO Handle error

		HttpResponse::Ok().body(logs)
	} else {
		HttpResponse::NotFound().finish()
	}
}

/// Structure representing the query for a request which starts a build job.
#[derive(Deserialize)]
struct JobStartQuery {
	/// The name of the package to build.
	name: String,
	/// The version of the package to build.
	version: Version,
}

#[post("/dashboard/job/start")]
async fn job_start(
	data: web::Data<Mutex<GlobalData>>,
	web::Query(query): web::Query<JobStartQuery>,
) -> impl Responder {
	let mut data = data.lock().unwrap();
	let id = data.new_job_id();

	let mut job = Job {
		desc: JobDesc {
			id,

			package: query.name,
			version: query.version,

			status: JobStatus::Pending,
		},

		process: None,
	};

	job.run().unwrap(); // TODO Handle error

	data.get_jobs_mut().push(job);
	HttpResponse::Ok().json(&data.get_jobs_mut().last().unwrap().desc)
}

#[post("/dashboard/job/{id}/abort")]
async fn job_abort(
	data: web::Data<Mutex<GlobalData>>,
	web::Path(id): web::Path<String>,
) -> impl Responder {
	let mut data = data.lock().unwrap();

	let job = data.get_jobs_mut()
		.iter_mut()
		.filter(| j | {
			j.desc.id == id
		}).next();
	let job = match job {
		Some(job) => job,
		None => return HttpResponse::NotFound().finish(),
	};

	job.abort();
	HttpResponse::Ok().finish()
}
