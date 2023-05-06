use std::{
    io::Write,
    process::{Command, Output},
};

use crate::error::WarpError;

pub trait CommandWithInput {
    fn call_process_with_input(&mut self, input: &str) -> Result<Output, WarpError>;
}

impl CommandWithInput for Command {
    fn call_process_with_input(&mut self, input: &str) -> Result<Output, WarpError> {
        let child = self.spawn()?;

        {
            let mut stdin = child.stdin.as_ref().unwrap();
            stdin.write_all(input.as_bytes())?;
            stdin.flush()?;
        }
        let out = child.wait_with_output()?;
        Ok(out)
    }
}
