#![deny(unused_variables)]

use anyhow::{Context, Error};
use build_helper::stage0_parser::{Stage0Config, VersionMetadata, parse_stage0_file};
use curl::easy::Easy;
use indexmap::IndexMap;

const PATH: &str = "src/stage0";
const COMPILER_COMPONENTS: &[&str] = &["rustc", "rust-std", "cargo", "clippy-preview"];
const RUSTFMT_COMPONENTS: &[&str] = &["rustfmt-preview", "rustc"];

struct Tool {
    config: Stage0Config,

    channel: Channel,
    date: Option<String>,
    version: [u16; 3],
    checksums: IndexMap<String, String>,
}

impl Tool {
    fn new(date: Option<String>) -> Result<Self, Error> {
        let channel = match std::fs::read_to_string("src/ci/channel")?.trim() {
            "stable" => Channel::Stable,
            "beta" => Channel::Beta,
            "nightly" => Channel::Nightly,
            other => anyhow::bail!("unsupported channel: {}", other),
        };

        // Split "1.42.0" into [1, 42, 0]
        let version = std::fs::read_to_string("src/version")?
            .trim()
            .split('.')
            .map(|val| val.parse())
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| anyhow::anyhow!("failed to parse version"))?;

        let existing = parse_stage0_file();

        Ok(Self { channel, version, date, config: existing.config, checksums: IndexMap::new() })
    }

    fn update_stage0_file(mut self) -> Result<(), Error> {
        const COMMENTS: &str = r#"# The configuration above this comment is editable, and can be changed
# by forks of the repository if they have alternate values.
#
# The section below is generated by `./x.py run src/tools/bump-stage0`,
# run that command again to update the bootstrap compiler.
#
# All changes below this comment will be overridden the next time the
# tool is executed.
"#;

        let mut file_content = String::new();

        // Destructure `Stage0Config` here to ensure the stage0 file is synced with any new
        // fields when they are added.
        let Stage0Config {
            dist_server,
            artifacts_server,
            artifacts_with_llvm_assertions_server,
            git_merge_commit_email,
            git_repository,
            nightly_branch,
        } = &self.config;

        file_content.push_str(&format!("dist_server={}\n", dist_server));
        file_content.push_str(&format!("artifacts_server={}\n", artifacts_server));
        file_content.push_str(&format!(
            "artifacts_with_llvm_assertions_server={}\n",
            artifacts_with_llvm_assertions_server
        ));
        file_content.push_str(&format!("git_merge_commit_email={}\n", git_merge_commit_email));
        file_content.push_str(&format!("git_repository={}\n", git_repository));
        file_content.push_str(&format!("nightly_branch={}\n", nightly_branch));

        file_content.push_str("\n");
        file_content.push_str(COMMENTS);
        file_content.push_str("\n");

        let compiler = self.detect_compiler()?;
        file_content.push_str(&format!("compiler_date={}\n", compiler.date));
        file_content.push_str(&format!("compiler_version={}\n", compiler.version));

        if let Some(rustfmt) = self.detect_rustfmt()? {
            file_content.push_str(&format!("rustfmt_date={}\n", rustfmt.date));
            file_content.push_str(&format!("rustfmt_version={}\n", rustfmt.version));
        }

        file_content.push_str("\n");

        for (key, value) in self.checksums {
            file_content.push_str(&format!("{}={}\n", key, value));
        }

        std::fs::write(PATH, file_content)?;
        Ok(())
    }

    // Currently Rust always bootstraps from the previous stable release, and in our train model
    // this means that the master branch bootstraps from beta, beta bootstraps from current stable,
    // and stable bootstraps from the previous stable release.
    //
    // On the master branch the compiler version is configured to `beta` whereas if you're looking
    // at the beta or stable channel you'll likely see `1.x.0` as the version, with the previous
    // release's version number.
    fn detect_compiler(&mut self) -> Result<VersionMetadata, Error> {
        let channel = match self.channel {
            Channel::Stable | Channel::Beta => {
                // The 1.XX manifest points to the latest point release of that minor release.
                format!("{}.{}", self.version[0], self.version[1] - 1)
            }
            Channel::Nightly => "beta".to_string(),
        };

        let manifest = fetch_manifest(&self.config, &channel, self.date.as_deref())?;
        self.collect_checksums(&manifest, COMPILER_COMPONENTS)?;
        Ok(VersionMetadata {
            date: manifest.date,
            version: if self.channel == Channel::Nightly {
                "beta".to_string()
            } else {
                // The version field is like "1.42.0 (abcdef1234 1970-01-01)"
                manifest.pkg["rust"]
                    .version
                    .split_once(' ')
                    .expect("invalid version field")
                    .0
                    .to_string()
            },
        })
    }

    /// We use a nightly rustfmt to format the source because it solves some bootstrapping issues
    /// with use of new syntax in this repo. For the beta/stable channels rustfmt is not provided,
    /// as we don't want to depend on rustfmt from nightly there.
    fn detect_rustfmt(&mut self) -> Result<Option<VersionMetadata>, Error> {
        if self.channel != Channel::Nightly {
            return Ok(None);
        }

        let manifest = fetch_manifest(&self.config, "nightly", self.date.as_deref())?;
        self.collect_checksums(&manifest, RUSTFMT_COMPONENTS)?;
        Ok(Some(VersionMetadata { date: manifest.date, version: "nightly".into() }))
    }

    fn collect_checksums(&mut self, manifest: &Manifest, components: &[&str]) -> Result<(), Error> {
        let prefix = format!("{}/", self.config.dist_server);
        for component in components {
            let pkg = manifest
                .pkg
                .get(*component)
                .ok_or_else(|| anyhow::anyhow!("missing component from manifest: {}", component))?;
            for target in pkg.target.values() {
                for pair in &[(&target.url, &target.hash), (&target.xz_url, &target.xz_hash)] {
                    if let (Some(url), Some(sha256)) = pair {
                        let url = url
                            .strip_prefix(&prefix)
                            .ok_or_else(|| {
                                anyhow::anyhow!("url doesn't start with dist server base: {}", url)
                            })?
                            .to_string();
                        self.checksums.insert(url, sha256.clone());
                    }
                }
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let tool = Tool::new(std::env::args().nth(1))?;
    tool.update_stage0_file()?;
    Ok(())
}

fn fetch_manifest(
    config: &Stage0Config,
    channel: &str,
    date: Option<&str>,
) -> Result<Manifest, Error> {
    let url = if let Some(date) = date {
        format!("{}/dist/{}/channel-rust-{}.toml", config.dist_server, date, channel)
    } else {
        format!("{}/dist/channel-rust-{}.toml", config.dist_server, channel)
    };

    Ok(toml::from_slice(&http_get(&url)?)?)
}

fn http_get(url: &str) -> Result<Vec<u8>, Error> {
    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.fail_on_error(true)?;
    handle.url(url)?;
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        })?;
        transfer.perform().context(format!("failed to fetch {url}"))?;
    }
    Ok(data)
}

#[derive(Debug, PartialEq, Eq)]
enum Channel {
    Stable,
    Beta,
    Nightly,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Manifest {
    date: String,
    pkg: IndexMap<String, ManifestPackage>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ManifestPackage {
    version: String,
    target: IndexMap<String, ManifestTargetPackage>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ManifestTargetPackage {
    url: Option<String>,
    hash: Option<String>,
    xz_url: Option<String>,
    xz_hash: Option<String>,
}
