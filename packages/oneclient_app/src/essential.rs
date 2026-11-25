use oneclient_content::packages::ProviderId;

/// Packages whose disable or removal must be confirmed with a dedicated
/// explanation before it goes through. The guard is driven by this list; the
/// fork currently ships none, but bundled packages reach the same confirm
/// through the disable-warnings flow.
#[derive(Debug, Clone, Copy)]
pub struct EssentialPackage {
    pub provider: ProviderId,
    pub project_id: &'static str,
    pub name: &'static str,
    pub disable_body: fn() -> &'static str,
    pub remove_body: fn() -> &'static str,
}

impl PartialEq for EssentialPackage {
    fn eq(&self, other: &Self) -> bool {
        self.provider == other.provider && self.project_id == other.project_id
    }
}

impl Eq for EssentialPackage {}

pub const ESSENTIAL_PACKAGES: &[EssentialPackage] = &[];

pub fn lookup(provider: ProviderId, package_id: &str) -> Option<&'static EssentialPackage> {
    ESSENTIAL_PACKAGES
        .iter()
        .find(|package| package.provider == provider && package.project_id == package_id)
}
