use freya::query::{QueriesStorage, Query, QueryCapability, UseQuery, use_query};
use oneclient_core::{LauncherError, LibrarySkin};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SkinLibraryKeys;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SkinLibraryQuery;

impl QueryCapability for SkinLibraryQuery {
    type Ok = Vec<(Uuid, LibrarySkin)>;
    type Err = LauncherError;
    type Keys = SkinLibraryKeys;

    async fn run(&self, _keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        Ok(oneclient_core::list_library_skins().await)
    }
}

pub fn use_skin_library() -> UseQuery<SkinLibraryQuery> {
    use_query(Query::new(SkinLibraryKeys, SkinLibraryQuery))
}

pub async fn invalidate_skin_library() {
    QueriesStorage::<SkinLibraryQuery>::invalidate_matching(SkinLibraryKeys).await;
}
