use super::*;
use std::io::BufReader;
use std::process::{Command, Stdio};
use tokio_process::CommandExt;

impl RunnableAgent for Arc<super::Executor> {
    fn execute(&self) -> Result<()> {
        let mut zelf = self.clone();
        let mut child = Command::new(self.command.clone())
            .args(self.args.clone())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn_async()?;

        let stdout = child.stdout().take().ok_or(AppError::Critical {
            message: format!(
                "Failed to start agent {}, failed to capture stdout",
                self.name
            ),
        })?;

        let stderr = child.stderr().take().ok_or(AppError::Critical {
            message: format!(
                "Failed to start agent {}, failed to capture stderr",
                self.name
            ),
        })?;

        spawn!(
            tokio::io::lines(BufReader::new(stdout)).for_each(move |line| {
                info!("{}" [&line] agent: zelf.name => zelf.logger);
                Ok(())
            })
        )?;

        zelf = self.clone();
        spawn!(
            tokio::io::lines(BufReader::new(stderr)).for_each(move |line| {
                warn!("{}" [&line] agent: zelf.name => zelf.logger);
                Ok(())
            })
        )?;

        zelf = self.clone();
        let zelf2 = self.clone();
        spawn!(child
            .map(move |status| info!("Process exited with status: {}" [status] agent: zelf.name, reason: status => zelf.logger))
            .map_err(move |e| warn!("Failed to start process: {}" [e] agent: zelf2.name => zelf2.logger)))
    }
}
