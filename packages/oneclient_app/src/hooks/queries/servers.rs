use freya::query::{Query, QueryCapability, UseQuery, use_query};
use oneclient_core::LauncherError;
use oneclient_core::servers::ServerList;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ServersKeys;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ServersQuery;

impl QueryCapability for ServersQuery {
    type Ok = ServerList;
    type Err = LauncherError;
    type Keys = ServersKeys;

    async fn run(&self, _keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let state = crate::launcher::state()?;
        oneclient_core::servers::fetch_servers(&state.services.requester).await
    }
}

pub fn use_servers() -> UseQuery<ServersQuery> {
    use_query(Query::new(ServersKeys, ServersQuery))
}
