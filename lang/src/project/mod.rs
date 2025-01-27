use {
    crate::{
        errors::ErroneousExt,
        runtime::{self, scope::Scope, types::ContextualValue},
        sitter::{self, expr::ContextualExpr, Span},
    },
    anyhow::{anyhow, bail, ensure},
    itertools::Itertools,
    serde::{Deserialize, Serialize},
    serde_json::Value,
    source::SOURCES,
    std::{
        collections::HashMap,
        fmt::{Display, Write},
        fs::OpenOptions,
        io::Read,
        path::{Path, PathBuf},
        str::FromStr,
        sync::{Arc, LazyLock, OnceLock, RwLock},
    },
};

pub mod source;

pub static PACKAGE: OnceLock<RwLock<(Package, Option<Package>)>> = OnceLock::new();
pub static EXPORTS: LazyLock<RwLock<HashMap<String, Arc<Scope>>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(Clone, Hash, Debug)]
pub struct SemverPackage(String, Semver, SemverMode);
impl SemverPackage {
    pub fn name(&self) -> String {
        self.0.clone()
    }

    pub fn semver(&self) -> (Semver, SemverMode) {
        (self.1.clone(), self.2.clone())
    }
}

#[derive(Clone, Debug, Hash)]
pub struct Semver {
    numerals: (u16, u16, u16),
    label: Option<String>,
}

#[derive(Clone, Debug, Copy, Hash)]
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

impl<'de> Deserialize<'de> for SemverPackage {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        SemverPackage::from_str(&s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

impl Serialize for SemverPackage {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        String::serialize(&self.to_string(), s)
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
    pub dependencies: Vec<SemverPackage>,
    pub main: String,

    #[serde(skip_serializing)]
    pub disk_path: String,
}

impl Package {
    pub fn from_file(p: PathBuf) -> anyhow::Result<Self> {
        if p.extension().unwrap_or_default() == "fl" {
            return Self::from_folder(p.parent().unwrap().to_path_buf());
        }

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
            false => match p.parent().is_some() {
                true => Self::from_folder(p.parent().unwrap().to_path_buf()),
                false => bail!("No manifest.json file found"),
            },
            true => Self::from_file(p.join("manifest.json")),
        }
    }

    pub fn resolve_dependent(&self, p: SemverPackage) -> anyhow::Result<Package> {
        let path = Path::new(&self.disk_path).join(&format!(".fl/dep/{p}")).join("manifest.json");
        match path.exists() {
            false => bail!("No manifest found for package '{p}'. Might not be installed."),
            true => Ok(serde_json::from_reader(OpenOptions::new().read(true).open(path).unwrap())
                .map_err(|e| anyhow!("Failed parsing manifest for '{p}': {e:?}"))?),
        }
    }

    pub fn dependent_package(&self, mut index: Vec<String>) -> anyhow::Result<Package> {
        let root = Path::new(&{
            let root: String = index.remove(0);
            if root == self.name {
                let path = Path::new(&self.disk_path.clone()).join(self.main.clone());
                path.parent().unwrap().display().to_string()
            } else {
                let dep = self
                    .dependencies
                    .iter()
                    .find(|d| d.name() == root)
                    .ok_or(anyhow!("No package named '{root}' in manifest"))?;

                let dep = self.resolve_dependent(dep.clone()).unwrap();
                let path = Path::new(&dep.disk_path.clone()).join(dep.main.clone());
                path.parent().unwrap().display().to_string()
            }
        })
        .to_path_buf();

        let mut folder = root;
        for package in index.iter().take(index.len() - 1) {
            ensure!(folder.is_dir(), "Expected folder, found file '{package}'");
            folder = folder.join(package);
            if !folder.exists() {
                bail!("Failed to resolve package {} at '{}'", index.join("::"), package);
            }
        }

        let mut p = self.clone();
        p.name = index.clone().join("::");
        p.main = folder.join(format!("{}.fl", index.last().unwrap())).display().to_string();
        Ok(p)
    }

    pub fn process(&self) -> anyhow::Result<(Option<ContextualValue>, Vec<Span>)> {
        process_file(Path::new(&self.disk_path).join(self.main.clone()))
    }

    pub fn child(&self, path: String) -> anyhow::Result<String> {
        let binding = Path::new(&self.disk_path).join(self.main.clone());
        let relative = binding.parent().unwrap();
        let mut child = Path::new(&path).parent().unwrap();
        let mut tree = vec![self.name.clone()];

        while child != relative {
            if let Some(parent) = child.parent() {
                child = parent;
                tree.push(parent.file_name().unwrap().to_string_lossy().to_string())
            } else {
                bail!("Couldn't find child '{path}'")
            }
        }

        tree.push(Path::new(&path).file_stem().unwrap().to_string_lossy().to_string());
        Ok(tree.join("::"))
    }

    pub fn snoop(&self, path: Option<PathBuf>) {
        let path = path.unwrap_or(Path::new(&self.disk_path).to_path_buf());
        if let Ok(dir) = path.read_dir() {
            for entry in dir {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() && !entry.file_name().to_string_lossy().to_string().starts_with(".") {
                        self.snoop(Some(entry.path()));
                    } else {
                        let mut input = String::new();
                        if let Ok(mut file) = OpenOptions::new().read(true).open(entry.path().clone()) {
                            file.read_to_string(&mut input).unwrap();
                            SOURCES.add_source(entry.path().display().to_string(), input);
                        }
                    }
                }
            }
        };
    }
}

pub fn pack() -> Package {
    PACKAGE.get().unwrap().read().unwrap().clone().0
}
pub fn export(path: String) -> Arc<Scope> {
    let package = Package::from_file(PathBuf::from(path.clone())).unwrap().child(path).unwrap();
    let ex = EXPORTS.read().unwrap().get(&package).unwrap_or(&Arc::new(Scope::new())).clone();
    EXPORTS.write().unwrap().insert(package, ex.clone());
    ex
}

pub fn process_file(path: PathBuf) -> anyhow::Result<(Option<ContextualValue>, Vec<Span>)> {
    let path = path.canonicalize().unwrap();

    let mut input = String::new();
    OpenOptions::new().read(true).open(path.clone())?.read_to_string(&mut input).unwrap();
    SOURCES.add_source(path.display().to_string(), input);

    let (tree, errors) = sitter::parse(path.display().to_string());

    if !errors.is_empty() {
        return Ok((None, errors));
    }

    Ok((runtime::process(tree, None, Some(path.display().to_string())).unwrappers(), Vec::new()))
}
