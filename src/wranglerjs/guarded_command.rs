use std::process::{Child, Command};

// wrapper around spawning child processes such that they
// have the same behavior as spawned threads i.e. a spawned
// child process using GuardedChild has the same lifetime as
// the main thread.
pub struct GuardedCommand(Child);

impl GuardedCommand {
    pub fn spawn(mut command: Command) -> GuardedCommand {
        GuardedCommand(command.spawn().expect("failed to execute child command"))
    }
}

impl Drop for GuardedCommand {
    fn drop(&mut self) {
        if let Err(e) = self.0.kill() {
            panic!("Failed to kill child process: {:?}", e);
        }
    }
}
