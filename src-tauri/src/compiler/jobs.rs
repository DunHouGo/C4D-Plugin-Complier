//! 内存中的构建任务注册表。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Emitter};

use crate::compiler::build::{build_log_timestamp, execute_build};
use crate::types::{
    BuildArtifact, BuildFinishedEvent, BuildJobId, BuildLogEvent, BuildProgressEvent, BuildRequest,
};

#[derive(Clone, Default)]
pub struct JobManager {
    inner: Arc<Mutex<HashMap<String, JobState>>>,
}

#[derive(Clone, Default)]
struct JobState {
    artifacts: Vec<BuildArtifact>,
    cancelled: bool,
    finished: bool,
}

impl JobManager {
    pub fn start_build(&self, app: AppHandle, request: BuildRequest) -> BuildJobId {
        let job_id = create_job_id();
        {
            let mut inner = self.inner.lock().expect("job manager poisoned");
            inner.insert(job_id.clone(), JobState::default());
        }

        let manager = self.clone();
        let thread_job_id = job_id.clone();
        thread::spawn(move || {
            let log_app = app.clone();
            let progress_app = app.clone();
            let log_job_id = thread_job_id.clone();
            let progress_job_id = thread_job_id.clone();

            let log = |event: BuildLogEvent| {
                let _ = log_app.emit("build://log", event);
            };
            let progress = |event: BuildProgressEvent| {
                let _ = progress_app.emit("build://progress", event);
            };

            let result = execute_build(&thread_job_id, &request, &log, &progress);
            match result {
                Ok(artifacts) => {
                    manager.set_artifacts(&thread_job_id, artifacts.clone());
                    for artifact in artifacts {
                        let _ = app.emit("build://artifact", artifact);
                    }
                    manager.finish(&thread_job_id);
                    let _ = app.emit(
                        "build://finished",
                        BuildFinishedEvent {
                            job_id: thread_job_id,
                            success: true,
                            message: "Build finished".to_string(),
                        },
                    );
                }
                Err(error) => {
                    manager.finish(&thread_job_id);
                    let _ = app.emit(
                        "build://log",
                        BuildLogEvent {
                            job_id: log_job_id,
                            level: "error".to_string(),
                            category: "system".to_string(),
                            timestamp: build_log_timestamp(),
                            message: error.clone(),
                        },
                    );
                    let _ = app.emit(
                        "build://finished",
                        BuildFinishedEvent {
                            job_id: progress_job_id,
                            success: false,
                            message: error,
                        },
                    );
                }
            }
        });

        BuildJobId { id: job_id }
    }

    pub fn cancel_build(&self, job_id: &str) -> bool {
        let mut inner = self.inner.lock().expect("job manager poisoned");
        if let Some(state) = inner.get_mut(job_id) {
            state.cancelled = true;
            true
        } else {
            false
        }
    }

    pub fn list_artifacts(&self, job_id: &str) -> Vec<BuildArtifact> {
        self.inner
            .lock()
            .expect("job manager poisoned")
            .get(job_id)
            .map(|state| state.artifacts.clone())
            .unwrap_or_default()
    }

    fn set_artifacts(&self, job_id: &str, artifacts: Vec<BuildArtifact>) {
        if let Some(state) = self
            .inner
            .lock()
            .expect("job manager poisoned")
            .get_mut(job_id)
        {
            state.artifacts = artifacts;
        }
    }

    fn finish(&self, job_id: &str) {
        if let Some(state) = self
            .inner
            .lock()
            .expect("job manager poisoned")
            .get_mut(job_id)
        {
            state.finished = true;
        }
    }
}

fn create_job_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("build-{millis}")
}
