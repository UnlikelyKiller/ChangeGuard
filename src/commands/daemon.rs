use crate::commands::watch::execute_watch;
use miette::Result;

pub fn execute_daemon(_interval_ms: u64) -> Result<()> {
    // We re-use the watch command but force JSON output mode to maintain a stable stream.
    execute_watch(1000, true)
}
