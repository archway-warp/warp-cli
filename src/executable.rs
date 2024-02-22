use crate::{
    chains::chain_profile::ChainProfile, error::WarpError, utils::project_config::ProjectConfig,
};

pub trait Executable {
    fn execute(
        &self,
        project_root: Option<std::path::PathBuf>,
        config: Option<ProjectConfig>,
        profile: &Box<dyn ChainProfile>,
    ) -> Result<(), WarpError>;
}
