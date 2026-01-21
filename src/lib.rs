use zed_extension_api::{
    self as zed, Command, DownloadedFileType, LanguageServerId, Result, Worktree,
};

struct MoveOnAptosExtension {
    cached_binary_path: Option<String>,
}

impl MoveOnAptosExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<String> {
        // Return cached path if available
        if let Some(path) = &self.cached_binary_path {
            if std::fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        // Try to find in system PATH first (for users who installed manually)
        if let Some(path) = worktree.which("aptos-language-server") {
            self.cached_binary_path = Some(path.clone());
            return Ok(path);
        }

        // Download from GitHub releases
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "aptos-labs/move-vscode-extension",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let (asset_name, file_type) = asset_name_for_platform(platform, arch)?;

        // Find matching asset
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "No prebuilt binary found for {:?}-{:?}. Looking for asset: {}. Available assets: {:?}",
                    platform,
                    arch,
                    asset_name,
                    release.assets.iter().map(|a| &a.name).collect::<Vec<_>>()
                )
            })?;

        let version_dir = format!("aptos-language-server-{}", release.version);
        let binary_name = if platform == zed::Os::Windows {
            "aptos-language-server.exe"
        } else {
            "aptos-language-server"
        };
        let binary_path = format!("{version_dir}/{binary_name}");

        // Download if not already present
        if !std::fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            std::fs::create_dir_all(&version_dir)
                .map_err(|e| format!("Failed to create directory: {e}"))?;

            zed::download_file(&asset.download_url, &binary_path, file_type)
                .map_err(|e| format!("Failed to download language server: {e}"))?;

            zed::make_file_executable(&binary_path)?;

            // Clean up old versions
            if let Ok(entries) = std::fs::read_dir(".") {
                for entry in entries.flatten() {
                    let entry_name = entry.file_name();
                    if entry_name
                        .to_string_lossy()
                        .starts_with("aptos-language-server-")
                        && entry_name != version_dir.as_str()
                    {
                        let _ = std::fs::remove_dir_all(entry.path());
                    }
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

fn asset_name_for_platform(
    os: zed::Os,
    arch: zed::Architecture,
) -> Result<(String, DownloadedFileType)> {
    match os {
        zed::Os::Mac => match arch {
            zed::Architecture::Aarch64 => Ok((
                "aptos-language-server-aarch64-apple-darwin.gz".to_string(),
                DownloadedFileType::Gzip,
            )),
            zed::Architecture::X8664 => Ok((
                "aptos-language-server-x86_64-apple-darwin.gz".to_string(),
                DownloadedFileType::Gzip,
            )),
            _ => Err(format!("Unsupported macOS architecture: {:?}", arch)),
        },
        zed::Os::Linux => match arch {
            zed::Architecture::X8664 => Ok((
                "aptos-language-server-x86_64-unknown-linux-gnu.gz".to_string(),
                DownloadedFileType::Gzip,
            )),
            zed::Architecture::Aarch64 => Ok((
                "aptos-language-server-aarch64-unknown-linux-gnu.gz".to_string(),
                DownloadedFileType::Gzip,
            )),
            _ => Err(format!("Unsupported Linux architecture: {:?}", arch)),
        },
        zed::Os::Windows => match arch {
            zed::Architecture::X8664 => Ok((
                "aptos-language-server-x86_64-pc-windows-msvc.zip".to_string(),
                DownloadedFileType::Zip,
            )),
            _ => Err(format!("Unsupported Windows architecture: {:?}", arch)),
        },
    }
}

impl zed::Extension for MoveOnAptosExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let binary_path = self.language_server_binary_path(language_server_id, worktree)?;

        Ok(Command {
            command: binary_path,
            args: vec!["lsp-server".to_string()],
            env: Default::default(),
        })
    }
}

zed::register_extension!(MoveOnAptosExtension);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_name_mac_m1() {
        let (name, file_type) =
            asset_name_for_platform(zed::Os::Mac, zed::Architecture::Aarch64).unwrap();
        assert_eq!(name, "aptos-language-server-aarch64-apple-darwin.gz");
        assert!(matches!(file_type, DownloadedFileType::Gzip));
    }

    #[test]
    fn test_asset_name_mac_intel() {
        let (name, file_type) =
            asset_name_for_platform(zed::Os::Mac, zed::Architecture::X8664).unwrap();
        assert_eq!(name, "aptos-language-server-x86_64-apple-darwin.gz");
        assert!(matches!(file_type, DownloadedFileType::Gzip));
    }

    #[test]
    fn test_asset_name_linux_x64() {
        let (name, file_type) =
            asset_name_for_platform(zed::Os::Linux, zed::Architecture::X8664).unwrap();
        assert_eq!(name, "aptos-language-server-x86_64-unknown-linux-gnu.gz");
        assert!(matches!(file_type, DownloadedFileType::Gzip));
    }

    #[test]
    fn test_asset_name_windows_x64() {
        let (name, file_type) =
            asset_name_for_platform(zed::Os::Windows, zed::Architecture::X8664).unwrap();
        assert_eq!(name, "aptos-language-server-x86_64-pc-windows-msvc.zip");
        assert!(matches!(file_type, DownloadedFileType::Zip));
    }

    #[test]
    #[ignore = "Requires network and external tools (curl, tar/unzip)"]
    fn test_real_download_and_execution() {
        // 1. Detect Host OS/Arch
        let os = match std::env::consts::OS {
            "macos" => zed::Os::Mac,
            "linux" => zed::Os::Linux,
            "windows" => zed::Os::Windows,
            _ => panic!("Unsupported OS for test: {}", std::env::consts::OS),
        };

        let arch = match std::env::consts::ARCH {
            "aarch64" => zed::Architecture::Aarch64,
            "x86_64" => zed::Architecture::X8664,
            _ => panic!("Unsupported Arch for test: {}", std::env::consts::ARCH),
        };

        // 2. Get expected asset name using our logic
        let (asset_name, file_type) = asset_name_for_platform(os, arch).expect("Failed to get asset name");
        println!("Target asset: {}", asset_name);

        // 3. Fetch latest release info to get URL
        let output = std::process::Command::new("curl")
            .args(&[
                "-s",
                "https://api.github.com/repos/aptos-labs/move-vscode-extension/releases/latest",
            ])
            .output()
            .expect("Failed to run curl");
        
        let json_str = String::from_utf8_lossy(&output.stdout);
        
        // Simple string parsing to find the browser_download_url for our asset
        // (Avoids adding serde_json dependency just for this test)
        let pattern = format!("\"name\": \"{}\"", asset_name);
        
        let mut download_url = String::new();

        if let Some(name_idx) = json_str.find(&pattern) {
            // Look forward from the name. 
            // We assume browser_download_url appears after "name" in the same object.
            // This is typical for GitHub Release API.
            let rest = &json_str[name_idx..];
            
            if let Some(url_key_idx) = rest.find("\"browser_download_url\": \"") {
                let url_start = url_key_idx + 25; // 25 is length of `"browser_download_url": "`
                let url_rest = &rest[url_start..];
                if let Some(url_end) = url_rest.find('"') {
                    download_url = url_rest[..url_end].to_string();
                }
            }
        } else {
             // Print a snippet of json for debugging
             println!("JSON Response (snippet): {:.200}...", json_str);
             panic!("Release JSON does not contain asset: {}", asset_name);
        }

        if download_url.is_empty() {
             // Debugging help
             println!("JSON Response (snippet after name): {:.200}...", 
                json_str.get(json_str.find(&pattern).unwrap_or(0).. )
                .unwrap_or("")
             );
            panic!("Could not find download URL for {}", asset_name);
        }
        println!("Downloading from: {}", download_url);

        // 4. Download
        let status = std::process::Command::new("curl")
            .args(&["-L", "-o", &asset_name, &download_url])
            .status()
            .expect("Failed to download");
        assert!(status.success(), "Download failed");

        // 5. Extract
        match file_type {
            DownloadedFileType::Gzip => {
                let status = std::process::Command::new("gzip")
                    .args(&["-d", "-k", "-f", &asset_name]) // -k keep, -f force
                    .status()
                    .expect("Failed to unzip");
                assert!(status.success(), "Gunzip failed");
            }
            DownloadedFileType::Zip => {
                 let status = std::process::Command::new("unzip")
                    .args(&["-o", &asset_name])
                    .status()
                    .expect("Failed to unzip");
                assert!(status.success(), "Unzip failed");
            }
            _ => panic!("Unsupported archive type"),
        }

        // 6. Run Binary
        let binary_name = if os == zed::Os::Windows {
            "aptos-language-server.exe"
        } else {
            // Gzip usually extracts to filename without .gz
            if asset_name.ends_with(".gz") {
                &asset_name[..asset_name.len() - 3]
            } else {
                "aptos-language-server"
            }
        };

        // chmod +x
        if os != zed::Os::Windows {
            std::process::Command::new("chmod")
                .args(&["+x", binary_name])
                .status()
                .expect("Failed to chmod");
        }

        println!("Running binary: ./{}", binary_name);
        let output = std::process::Command::new(format!("./{}", binary_name))
            .arg("--version")
            .output()
            .expect("Failed to run binary");

        if !output.status.success() {
             let stderr = String::from_utf8_lossy(&output.stderr);
             panic!("Binary failed to run: {}", stderr);
        }

        println!("Binary Output: {}", String::from_utf8_lossy(&output.stdout));

        // 7. Cleanup
        let _ = std::fs::remove_file(&asset_name);
        let _ = std::fs::remove_file(binary_name);
    }
}
