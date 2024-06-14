use exit_future::Signal;
use futures::channel::mpsc::Receiver;
use futures::StreamExt;
use task_executor::{ShutdownReason, TaskExecutor};
use tokio::{
    runtime::Runtime,
    signal::unix::{signal, SignalKind},
};

pub fn make_environment() -> Result<(Environment, Runtime, TaskExecutor), String> {
    let (signal, exit) = exit_future::signal();
    let (signal_tx, signal_rx) = futures::channel::mpsc::channel(1);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to start runtime: {:?}", e))?;
    let executor = TaskExecutor::new(runtime.handle().clone(), exit, signal_tx);
    Ok((Environment { signal, signal_rx }, runtime, executor))
}

pub struct Environment {
    #[allow(unused)]
    signal: Signal,
    signal_rx: Receiver<ShutdownReason>,
}

impl Environment {
    pub async fn wait_shutdown_signal(mut self) {
        let mut sig_term = match signal(SignalKind::terminate()) {
            Ok(x) => x,
            Err(e) => {
                error!(target: "Shutdown Handler", error = %e, "Could not register SIGTERM handler");
                return;
            }
        };

        let mut sig_int = match signal(SignalKind::interrupt()) {
            Ok(x) => x,
            Err(e) => {
                error!(target: "Shutdown Handler", error = %e, "Could not register SIGINT handler");
                return;
            }
        };

        let mut sig_hup = match signal(SignalKind::hangup()) {
            Ok(x) => x,
            Err(e) => {
                error!(target: "Shutdown Handler", error = %e, "Could not register SIGHUP handler");
                return;
            }
        };

        let mut internal_signal_enabled = true;

        loop {
            tokio::select! {
                res = self.signal_rx.next(), if internal_signal_enabled => {
                    match res {
                        Some(reason) => {
                            info!(target: "Shutdown Handler", reason = reason.message(), "Internal shutdown received");
                            return;
                        },
                        None => {
                            debug!(target: "Shutdown Handler", "Internal shutdown channel closed");
                            internal_signal_enabled = false;
                        },
                    }
                },
                _ = sig_term.recv() => {
                    info!(target: "Shutdown Handler", "Received SIGTERM");
                    return;
                }
                _ = sig_int.recv() => {
                    info!(target: "Shutdown Handler", "Received SIGINT");
                    return;
                }
                _ = sig_hup.recv() => {
                    info!(target: "Shutdown Handler", "Received SIGHUP");
                    return;
                }
            }
        }
    }
}
