use {
    crate::parser::expr::ContextualExpr,
    anyhow::{anyhow, bail},
    itertools::Itertools,
    serde::{Deserialize, Serialize},
    serde_json::Value,
    std::{
        fmt::{Display, Write},
        fs::OpenOptions,
        path::{Path, PathBuf},
        str::FromStr,
        sync::{LazyLock, OnceLock, RwLock},
    },
};

pub static PACKAGE: OnceLock<RwLock<(Package, Option<Package>)>> = OnceLock::new();

#[derive(Clone)]
pub struct SemverPackage(String, Semver, SemverMode);

#[derive(Clone, Debug)]
pub struct Semver {
    numerals: (u16, u16, u16),
    label: Option<String>,
}

#[derive(Clone, Debug, Copy)]
pub enum SemverMode {
    Strict,
    UpdatePatch,
    UpdateFeature,
    UpdateMajor,
}

impl FromStr for Semver {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (sem, label) = s.split_once('-').map(|(a, b)| (a, Some(b.to_string()))).unwrap_or((s, None));
        let (major, feature, patch) = sem
            .split('.')
            .map(|v| v.parse::<u16>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("Invalid digit in semver: {e:?}"))?
            .into_iter()
            .collect_tuple()
            .unwrap();

        Ok(Semver { numerals: (major, feature, patch), label })
    }
}

impl FromStr for SemverMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "^" => SemverMode::UpdateFeature,
            "~" => SemverMode::UpdatePatch,
            "=" => SemverMode::Strict,
            _ => bail!("Unknown semver match mode '{s}'"),
        })
    }
}

impl FromStr for SemverPackage {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, ver) = s.split_once('@').ok_or(anyhow!("No version provided for package '{s}'"))?;
        let (mode, ver) = ver.split_at(2);

        Ok(Self(name.to_string(), Semver::from_str(ver)?, SemverMode::from_str(mode)?))
    }
}

impl Display for SemverMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            SemverMode::Strict => '=',
            SemverMode::UpdatePatch => '~',
            SemverMode::UpdateFeature => '^',
            SemverMode::UpdateMajor => '*',
        })
    }
}

impl Display for Semver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.numerals.0, self.numerals.1, self.numerals.2).and_then(|_| match &self.label {
            Some(l) => write!(f, "-{l}"),
            None => write!(f, ""),
        })
    }
}

impl Display for SemverPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{name}@{mode}{version}", name = self.0, mode = self.2, version = self.1)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    pub main: String,

    #[serde(skip_serializing)]
    pub disk_path: String,
}

impl Package {
    pub fn from_file(p: PathBuf) -> anyhow::Result<Self> {
        let mut j: Value = serde_json::from_reader(OpenOptions::new().read(true).open(p.clone()).unwrap())
            .map_err(|e| anyhow!("Failed reading manifest for path '{}': {e:?}", p.display()))?;

        j.as_object_mut().unwrap().insert(
            "disk_path".to_string(),
            serde_json::Value::String(p.parent().unwrap().canonicalize()?.display().to_string()),
        );
        serde_json::from_value(j).map_err(|e| anyhow!("Failed parsing manifest for path '{}': {e:?}", p.display()))
    }

    pub fn from_folder(p: PathBuf) -> anyhow::Result<Self> {
        match p.join("manifest.json").exists() {
            false => bail!("No manifest.json file found"),
            true => Self::from_file(p.join("manifest.json")),
        }
    }

    pub fn resolve_dependent(p: SemverPackage) -> anyhow::Result<Package> {
        let path = Path::new(&pack().disk_path).join(&format!(".fl/dep/{p}")).join("manifest.json");
        match path.exists() {
            false => bail!("No manifest found for package '{p}'. Might not be installed."),
            true => Ok(serde_json::from_reader(OpenOptions::new().read(true).open(path).unwrap())
                .map_err(|e| anyhow!("Failed parsing manifest for '{p}': {e:?}"))?),
        }
    }
}

pub fn pack() -> Package { PACKAGE.get().unwrap().read().unwrap().clone().0 }
