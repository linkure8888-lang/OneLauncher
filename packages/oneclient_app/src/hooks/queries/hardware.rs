use freya::query::{Query, QueryCapability, UseQuery, use_query};
use oneclient_core::LauncherError;
use oneclient_core::presets::Hardware;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HardwareKeys;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HardwareQuery;

impl QueryCapability for HardwareQuery {
    type Ok = Hardware;
    type Err = LauncherError;
    type Keys = HardwareKeys;

    async fn run(&self, _keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        Ok(oneclient_core::presets::detect_hardware().await)
    }
}

pub fn use_hardware() -> UseQuery<HardwareQuery> {
    use_query(Query::new(HardwareKeys, HardwareQuery))
}
