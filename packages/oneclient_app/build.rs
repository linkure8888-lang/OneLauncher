fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icons/icon.ico");
        // Everything the launcher does is per-user (HKCU registry, %LOCALAPPDATA%),
        // so it must never trigger a UAC prompt or run elevated.
        res.set_manifest(
            r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="asInvoker" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#,
        );

        let version = env!("CARGO_PKG_VERSION");
        res.set("CompanyName", "RemixClient Contributors");
        res.set("ProductName", "RemixClient");
        res.set("FileDescription", "RemixClient");
        res.set(
            "LegalCopyright",
            "© 2026 RemixClient Contributors - GPL-3.0",
        );
        res.set("OriginalFilename", "oneclient_app.exe");
        res.set("InternalName", "oneclient_app");
        res.set("FileVersion", version);
        res.set("ProductVersion", version);

        res.compile()
            .expect("failed to embed Windows icon resource");
    }
}
