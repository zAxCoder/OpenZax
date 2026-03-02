use chrono::{DateTime, Utc};
use cron::Schedule;
use notify::{Event as NotifyEvent, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, str::FromStr};
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TriggerError {
    #[error("Invalid cron expression '{expr}': {reason}")]
    InvalidCron { expr: String, reason: String },

    #[error("Filesystem watch error: {0}")]
    WatchError(String),

    #[error("Trigger already registered for workflow {0}")]
    AlreadyRegistered(Uuid),

    #[error("Trigger not found: {0}")]
    NotFound(Uuid),

    #[error("Channel send error")]
    ChannelClosed,
}

pub type TriggerResult<T> = std::result::Result<T, TriggerError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerConfig {
    Cron {
        schedule: String,
    },
    FilesystemWatch {
        path: PathBuf,
        recursive: bool,
        events: Vec<FsEvent>,
    },
    Webhook {
        path: String,
        method: String,
        secret: Option<String>,
    },
    OsEvent {
        event_type: OsEventType,
    },
    McpEvent {
        server_id: String,
        event_name: String,
    },
    Manual,
    ChainedFrom {
        workflow_id: Uuid,
        condition: Option<String>,
    },
}

impl TriggerConfig {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Cron { .. } => "cron",
            Self::FilesystemWatch { .. } => "filesystem_watch",
            Self::Webhook { .. } => "webhook",
            Self::OsEvent { .. } => "os_event",
            Self::McpEvent { .. } => "mcp_event",
            Self::Manual => "manual",
            Self::ChainedFrom { .. } => "chained_from",
        }
    }

    pub fn validate(&self) -> TriggerResult<()> {
        if let Self::Cron { schedule } = self {
            Schedule::from_str(schedule).map_err(|e| TriggerError::InvalidCron {
                expr: schedule.clone(),
                reason: e.to_string(),
            })?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FsEvent {
    Created,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OsEventType {
    Startup,
    Shutdown,
    NetworkConnect,
    NetworkDisconnect,
    UsbAttached,
    LowMemory,
}

/// Event emitted when a trigger fires
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEvent {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub trigger_config: TriggerConfig,
    pub fired_at: DateTime<Utc>,
    pub payload: serde_json::Value,
}

impl TriggerEvent {
    pub fn new(workflow_id: Uuid, config: TriggerConfig, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            workflow_id,
            trigger_config: config,
            fired_at: Utc::now(),
            payload,
        }
    }
}

struct RegisteredTrigger {
    workflow_id: Uuid,
    config: TriggerConfig,
    abort_handle: Option<tokio::task::AbortHandle>,
}

/// Manages all trigger listeners for registered workflows
pub struct TriggerManager {
    triggers: HashMap<Uuid, RegisteredTrigger>,
    event_tx: mpsc::Sender<TriggerEvent>,
}

impl TriggerManager {
    pub fn new(event_tx: mpsc::Sender<TriggerEvent>) -> Self {
        Self {
            triggers: HashMap::new(),
            event_tx,
        }
    }

    /// Register a trigger configuration for the given workflow
    pub fn register(&mut self, workflow_id: Uuid, config: TriggerConfig) -> TriggerResult<()> {
        config.validate()?;

        if self.triggers.contains_key(&workflow_id) {
            return Err(TriggerError::AlreadyRegistered(workflow_id));
        }

        self.triggers.insert(
            workflow_id,
            RegisteredTrigger {
                workflow_id,
                config,
                abort_handle: None,
            },
        );

        debug!("Registered trigger for workflow {workflow_id}");
        Ok(())
    }

    /// Start all registered trigger listeners
    pub async fn start(&mut self) -> TriggerResult<()> {
        let workflow_ids: Vec<Uuid> = self.triggers.keys().copied().collect();

        for workflow_id in workflow_ids {
            self.start_trigger(workflow_id).await?;
        }

        info!("Started {} trigger(s)", self.triggers.len());
        Ok(())
    }

    async fn start_trigger(&mut self, workflow_id: Uuid) -> TriggerResult<()> {
        let trigger = self
            .triggers
            .get(&workflow_id)
            .ok_or(TriggerError::NotFound(workflow_id))?;

        let config = trigger.config.clone();
        let tx = self.event_tx.clone();

        let handle = match &config {
            TriggerConfig::Cron { schedule } => {
                let schedule = schedule.clone();
                tokio::spawn(async move {
                    run_cron_trigger(workflow_id, schedule, config, tx).await;
                })
                .abort_handle()
            }

            TriggerConfig::FilesystemWatch {
                path,
                recursive,
                events,
            } => {
                let path = path.clone();
                let recursive = *recursive;
                let events = events.clone();
                tokio::spawn(async move {
                    run_fs_watch_trigger(workflow_id, path, recursive, events, config, tx).await;
                })
                .abort_handle()
            }

            TriggerConfig::Manual => {
                // Manual triggers fire on-demand; no background task needed
                debug!("Manual trigger registered for workflow {workflow_id}");
                return Ok(());
            }

            TriggerConfig::Webhook { .. } => {
                // Webhook triggers are handled by the HTTP server; no background task
                debug!("Webhook trigger registered for workflow {workflow_id}");
                return Ok(());
            }

            TriggerConfig::ChainedFrom { .. } => {
                // Chained triggers fire when parent workflow completes; no background task
                debug!("Chained trigger registered for workflow {workflow_id}");
                return Ok(());
            }

            TriggerConfig::McpEvent {
                server_id,
                event_name,
            } => {
                let server_id = server_id.clone();
                let event_name = event_name.clone();
                tokio::spawn(async move {
                    run_mcp_event_trigger(workflow_id, server_id, event_name, config, tx).await;
                })
                .abort_handle()
            }

            TriggerConfig::OsEvent { event_type } => {
                let event_type = event_type.clone();
                tokio::spawn(async move {
                    run_os_event_trigger(workflow_id, event_type, config, tx).await;
                })
                .abort_handle()
            }
        };

        if let Some(trigger) = self.triggers.get_mut(&workflow_id) {
            trigger.abort_handle = Some(handle);
        }

        Ok(())
    }

    /// Stop all trigger listeners
    pub fn stop(&mut self) {
        for trigger in self.triggers.values_mut() {
            if let Some(handle) = trigger.abort_handle.take() {
                handle.abort();
                debug!("Stopped trigger for workflow {}", trigger.workflow_id);
            }
        }
        info!("All triggers stopped");
    }

    /// Manually fire a trigger (for Manual and testing purposes)
    pub async fn fire_manual(
        &self,
        workflow_id: Uuid,
        payload: serde_json::Value,
    ) -> TriggerResult<()> {
        let event = TriggerEvent::new(workflow_id, TriggerConfig::Manual, payload);
        self.event_tx
            .send(event)
            .await
            .map_err(|_| TriggerError::ChannelClosed)
    }

    /// Fire a chained trigger when a parent workflow completes
    pub async fn fire_chained(
        &self,
        parent_workflow_id: Uuid,
        output: serde_json::Value,
    ) -> TriggerResult<()> {
        for trigger in self.triggers.values() {
            if let TriggerConfig::ChainedFrom {
                workflow_id,
                condition,
            } = &trigger.config
            {
                if *workflow_id == parent_workflow_id {
                    // Evaluate condition if present (simplified: always fire if no condition)
                    let should_fire = condition
                        .as_ref()
                        .map(|_cond| true) // full eval would use an expression engine
                        .unwrap_or(true);

                    if should_fire {
                        let event = TriggerEvent::new(
                            trigger.workflow_id,
                            trigger.config.clone(),
                            output.clone(),
                        );
                        self.event_tx
                            .send(event)
                            .await
                            .map_err(|_| TriggerError::ChannelClosed)?;
                    }
                }
            }
        }
        Ok(())
    }
}

// ── Trigger runner tasks ──────────────────────────────────────────────────────

async fn run_cron_trigger(
    workflow_id: Uuid,
    schedule_str: String,
    config: TriggerConfig,
    tx: mpsc::Sender<TriggerEvent>,
) {
    let Ok(schedule) = Schedule::from_str(&schedule_str) else {
        error!("Invalid cron schedule '{schedule_str}' for workflow {workflow_id}");
        return;
    };

    info!("Cron trigger started for workflow {workflow_id}: {schedule_str}");

    loop {
        let now = chrono::Utc::now();
        let next = schedule.upcoming(chrono::Utc).next();

        if let Some(next_time) = next {
            let wait = (next_time - now).to_std().unwrap_or_default();
            tokio::time::sleep(wait).await;

            let event = TriggerEvent::new(
                workflow_id,
                config.clone(),
                serde_json::json!({
                    "fired_at": Utc::now().to_rfc3339(),
                    "schedule": schedule_str,
                }),
            );

            if tx.send(event).await.is_err() {
                debug!("Cron trigger channel closed for workflow {workflow_id}");
                return;
            }
        } else {
            warn!("No upcoming schedule for cron trigger {workflow_id}");
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    }
}

async fn run_fs_watch_trigger(
    workflow_id: Uuid,
    path: PathBuf,
    recursive: bool,
    watched_events: Vec<FsEvent>,
    config: TriggerConfig,
    tx: mpsc::Sender<TriggerEvent>,
) {
    let (notify_tx, mut notify_rx) = mpsc::channel::<NotifyEvent>(64);

    let mut watcher: RecommendedWatcher =
        match notify::recommended_watcher(move |res: notify::Result<NotifyEvent>| {
            if let Ok(event) = res {
                let _ = notify_tx.blocking_send(event);
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                error!("Failed to create filesystem watcher: {e}");
                return;
            }
        };

    let mode = if recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };
    if let Err(e) = watcher.watch(&path, mode) {
        error!("Failed to watch path {:?}: {e}", path);
        return;
    }

    info!(
        "Filesystem watch trigger started for workflow {workflow_id}: {:?}",
        path
    );

    while let Some(notify_event) = notify_rx.recv().await {
        let fs_event = match notify_event.kind {
            EventKind::Create(_) => FsEvent::Created,
            EventKind::Modify(_) => FsEvent::Modified,
            EventKind::Remove(_) => FsEvent::Deleted,
            _ => continue,
        };

        if !watched_events.is_empty() && !watched_events.contains(&fs_event) {
            continue;
        }

        let paths: Vec<String> = notify_event
            .paths
            .iter()
            .map(|p| p.display().to_string())
            .collect();

        let event = TriggerEvent::new(
            workflow_id,
            config.clone(),
            serde_json::json!({
                "event": format!("{:?}", fs_event),
                "paths": paths,
                "fired_at": Utc::now().to_rfc3339(),
            }),
        );

        if tx.send(event).await.is_err() {
            debug!("Fs watch trigger channel closed for workflow {workflow_id}");
            return;
        }
    }
}

async fn run_mcp_event_trigger(
    workflow_id: Uuid,
    server_id: String,
    event_name: String,
    _config: TriggerConfig,
    _tx: mpsc::Sender<TriggerEvent>,
) {
    // Poll MCP server for events every 5 seconds
    info!("MCP event trigger started for workflow {workflow_id}: {server_id}/{event_name}");
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

    loop {
        interval.tick().await;

        // In a full implementation this would connect to the MCP server via the
        // openzax-mcp-client crate and subscribe to the event stream
        debug!("MCP poll: {server_id}/{event_name} for workflow {workflow_id}");
    }
}

async fn run_os_event_trigger(
    workflow_id: Uuid,
    event_type: OsEventType,
    config: TriggerConfig,
    tx: mpsc::Sender<TriggerEvent>,
) {
    // OS event monitoring - simplified stub; real impl uses platform APIs
    info!(
        "OS event trigger started for workflow {workflow_id}: {:?}",
        event_type
    );

    match event_type {
        OsEventType::Startup => {
            // Fire once on startup
            let event = TriggerEvent::new(
                workflow_id,
                config,
                serde_json::json!({ "event": "startup", "fired_at": Utc::now().to_rfc3339() }),
            );
            let _ = tx.send(event).await;
        }
        _ => {
            // Other OS events require platform-specific polling
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                debug!(
                    "OS event poll for {:?} (workflow {workflow_id})",
                    event_type
                );
            }
        }
    }
}
