use crate::error::WarpError;

pub trait Executable {
    fn execute(&self) -> Result<(), WarpError>;
}
