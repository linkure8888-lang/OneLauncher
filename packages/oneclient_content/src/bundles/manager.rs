use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use oneclient_db::dao::bundle as bundle_dao;
use oneclient_db::dao::package_metadata as meta_dao;
use oneclient_db::models::BundleRow;
use tokio::sync::RwLock;

use crate::bundles::error::BundleError;
use crate::bundles::polymrpack;
use crate::bundles::types::{BundleArchive, BundleFileKind};
use crate::ctx::ContentCtx;
use crate::error::ContentResult;
use oneclient_common::domain::{GameLoader, ProviderId};
use oneclient_common::paths;

#[derive(Debug, Clone)]
pub struct Bundle {
    pub remote_path: String,
    pub mc_version: String,
    pub loader: GameLoader,
    pub file_name: String,
    pub name: String,
    pub version_id: String,
    pub category: String,
    pub loader_version: String,
    pub path: PathBuf,
    pub hidden: bool,
}

impl Bundle {
    fn try_from_row(row: BundleRow) -> ContentResult<Self> {
        let loader = GameLoader::from_repr(row.mc_loader as u8)
            .ok_or(BundleError::InvalidLoader(row.mc_loader))?;

        let launcher_dir = paths::data_dir()?;
        let path = launcher_dir.join(&row.disk_path);

        Ok(Self {
            remote_path: row.remote_path,
            mc_version: row.mc_version,
            loader,
            file_name: row.file_name,
            name: row.name.unwrap_or_default(),
            version_id: row.version_id.unwrap_or_default(),
            category: row.category.unwrap_or_default(),
            loader_version: row.loader_version.unwrap_or_default(),
            path,
            hidden: row.hidden != 0,
        })
    }
}

pub struct BundlesManager {
    pub(crate) archive_cache: RwLock<HashMap<PathBuf, crate::bundles::types::BundleManifest>>,
}

impl BundlesManager {
    pub fn new() -> Self {
        Self {
            archive_cache: RwLock::new(HashMap::new()),
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn sync(self: &Arc<Self>, ctx: &ContentCtx) -> ContentResult<bool> {
        // The bundle catalog is retired for good: it was served from a third
        // party and presented pre-made modpacks the player never asked for.
        // Every sync wipes the local catalog so no bundle card can come back,
        // while clusters the player already created stay untouched.
        bundle_dao::purge_catalog(&ctx.db)
            .await
            .map_err(|err| tracing::warn!(error = %err, "bundle catalog purge failed"))
            .ok();
        return Ok(false);
    }

    pub async fn cache_archive_manifest(
        &self,
        path: std::path::PathBuf,
        manifest: crate::bundles::types::BundleManifest,
    ) {
        self.archive_cache.write().await.insert(path, manifest);
    }

    #[tracing::instrument(level = "debug", skip(self, ctx))]
    pub async fn archives_for(
        &self,
        ctx: &ContentCtx,
        mc_version: &str,
        loader: GameLoader,
    ) -> ContentResult<Vec<BundleArchive>> {
        let mut archives = Vec::new();
        for bundle in self.list_for(ctx, mc_version, loader).await? {
            let manifest = self.manifest_for_archive(ctx, &bundle.path).await?;
            archives.push(BundleArchive { bundle, manifest });
        }
        Ok(archives)
    }

    #[tracing::instrument(level = "debug", skip(self, ctx))]
    async fn manifest_for_archive(
        &self,
        ctx: &ContentCtx,
        path: &Path,
    ) -> ContentResult<crate::bundles::types::BundleManifest> {
        if let Some(manifest) = self.archive_cache.read().await.get(path) {
            return Ok(manifest.clone());
        }
        let manifest = polymrpack::read_manifest_from_archive(path).await?;
        record_external_metadata(&manifest, ctx).await;
        self.archive_cache
            .write()
            .await
            .insert(path.to_path_buf(), manifest.clone());
        Ok(manifest)
    }

    #[tracing::instrument(level = "debug", skip(self, ctx))]
    pub async fn list_for(
        &self,
        ctx: &ContentCtx,
        mc_version: &str,
        loader: GameLoader,
    ) -> ContentResult<Vec<Bundle>> {
        let rows =
            bundle_dao::list_visible_for_version_loader(&ctx.db, mc_version, loader as i64).await?;

        rows.into_iter()
            .map(Bundle::try_from_row)
            .collect::<Result<Vec<_>, _>>()
    }
}

/// Registers the local metadata that ships inside a bundle's mrpack so the
/// packages appear with proper names and descriptions once installed.
#[tracing::instrument(level = "debug", skip_all, fields(bundle = %manifest.name))]
async fn record_external_metadata(
    manifest: &crate::bundles::types::BundleManifest,
    ctx: &ContentCtx,
) {
    for bundle_file in &manifest.files {
        let BundleFileKind::External {
            file,
            meta: Some(meta),
            ..
        } = &bundle_file.kind
        else {
            continue;
        };

        if let Err(err) = meta_dao::upsert_package_metadata(
            &ctx.db,
            ProviderId::Local as i64,
            &file.sha1,
            meta.name.as_deref().unwrap_or(&file.name),
            meta.description.as_deref().unwrap_or_default(),
            &meta.authors.join(", "),
            meta.icon_url.as_deref(),
        )
        .await
        {
            tracing::warn!(file = %file.name, error = %err, "failed to record bundle file metadata");
        }
    }
}

impl Default for BundlesManager {
    fn default() -> Self {
        Self::new()
    }
}
