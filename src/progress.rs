use std::sync::atomic::{AtomicUsize, Ordering};
use indicatif::{ProgressBar, ProgressStyle};

pub struct Progress {
    pub running: AtomicUsize,
    pub pb: ProgressBar,
}

impl Progress {
    pub fn new(total: usize) -> Self {
        let pb = ProgressBar::new(total as u64);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}\n{msg}"
            )
            .unwrap()
            .progress_chars("#>-"),
        );

        Self {
            running: AtomicUsize::new(0),
            pb,
        }
    }

    pub fn start_task(&self, name: &str) {
        let active = self.running.fetch_add(1, Ordering::SeqCst) + 1;
        self.pb.set_message(format!("Running: {} ({} active)", name, active));
    }

    pub fn finish_task(&self) {
        let active = self.running.fetch_sub(1, Ordering::SeqCst) - 1;
        if active > 0 {
            self.pb.set_message(format!("⚙️  {} tasks in flight...", active));
        } else {
            self.pb.set_message("💤 Waiting for dependencies...");
        }
        self.pb.inc(1);
    }
}