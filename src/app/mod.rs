use std::collections::VecDeque;

use anyhow::{Result, bail};

use crate::output::{self, OutputFormat};
use crate::scanner::{ScannedFile, ScannerConfig, scan};

/// Single source of truth for MVU state.
pub struct Model {
    config: ScannerConfig,
    files: Vec<ScannedFile>,
    status: Status,
    error: Option<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Status {
    Idle,
    Scanning,
    Completed,
    Failed,
}

#[derive(Debug)]
pub enum Msg {
    Start,
    ScanCompleted(Vec<ScannedFile>),
    ScanFailed(String),
}

/// Side-effect requests emitted by `update`.
enum Command {
    Scan(ScannerConfig),
}

impl Command {
    fn execute(self) -> Msg {
        match self {
            Command::Scan(config) => match scan(&config) {
                Ok(files) => Msg::ScanCompleted(files),
                Err(error) => Msg::ScanFailed(error.to_string()),
            },
        }
    }
}

impl Model {
    #[must_use]
    pub fn new(config: ScannerConfig) -> Self {
        Self {
            config,
            files: Vec::new(),
            status: Status::Idle,
            error: None,
        }
    }
}

/// Executes the MVU loop and returns the rendered report when scanning completes.
///
/// # Errors
/// Returns an error if scanning fails or if the MVU state machine does not reach `Completed`.
pub fn run(config: ScannerConfig, format: OutputFormat) -> Result<String> {
    let mut model = Model::new(config);
    let mut queue = VecDeque::new();
    // Seed the MVU cycle with the initial message.
    queue.push_back(Msg::Start);

    while let Some(message) = queue.pop_front() {
        // Update the model and execute any command emitted by this message.
        if let Some(command) = update(&mut model, message) {
            let follow_up = command.execute();
            queue.push_back(follow_up);
        }
    }

    match model.status {
        Status::Completed => Ok(output::render_report(&model.config, &model.files, format)),
        Status::Failed => {
            // Propagate failure details collected during the update phase.
            if let Some(message) = model.error {
                bail!(message);
            }
            bail!("command failed without providing an error message")
        }
        Status::Idle | Status::Scanning => bail!("application exited before finishing processing"),
    }
}

fn update(model: &mut Model, message: Msg) -> Option<Command> {
    match message {
        Msg::Start => {
            model.status = Status::Scanning;
            Some(Command::Scan(model.config.clone()))
        }
        Msg::ScanCompleted(files) => {
            model.status = Status::Completed;
            model.files = files;
            None
        }
        Msg::ScanFailed(error) => {
            model.status = Status::Failed;
            model.error = Some(error);
            None
        }
    }
}
