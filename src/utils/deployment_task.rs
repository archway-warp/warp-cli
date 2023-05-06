use super::project_config::AutoDeployStep;

#[derive(Clone, Debug)]
pub struct DeploymentTask<'a> {
    pub step: &'a AutoDeployStep,
    pub code_id: Option<String>,
    pub contract_address: Option<String>,
}
