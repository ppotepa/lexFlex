use crate::validation::{LanguageModelValidator, LanguageValidationIssue};
use crate::{
    compile_lexical_sense, CompiledLexicalSense, Form, FormId, FormIndex, LanguageCompileError,
    Lexeme, LexemeId, LexicalSense, LexicalSenseId, SenseIndex,
};
use lexflex_model::{canonical_hash, ConceptCatalog, LanguageId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

const SUPPORTED_MANIFEST_SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguagePackageManifest {
    pub schema: u32,
    pub package_id: String,
    pub language: LanguageId,
    pub lexemes: String,
    pub senses: String,
    pub forms: String,
    pub paradigms: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageModel {
    pub manifest: LanguagePackageManifest,
    pub lexemes: BTreeMap<LexemeId, Lexeme>,
    pub senses: BTreeMap<LexicalSenseId, LexicalSense>,
    pub compiled_senses: BTreeMap<LexicalSenseId, CompiledLexicalSense>,
    pub forms: BTreeMap<FormId, Form>,
    pub paradigms: BTreeMap<crate::ParadigmId, MorphologyParadigm>,
    pub form_index: FormIndex,
    pub sense_index: SenseIndex,
    pub model_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MorphologyParadigm {
    pub id: crate::ParadigmId,
    pub language: LanguageId,
    #[serde(default)]
    pub forms: Vec<Form>,
}

impl LanguageModel {
    pub fn lexeme(&self, id: &LexemeId) -> Option<&Lexeme> {
        self.lexemes.get(id)
    }

    pub fn sense(&self, id: &LexicalSenseId) -> Option<&LexicalSense> {
        self.senses.get(id)
    }

    pub fn compiled_sense(&self, id: &LexicalSenseId) -> Option<&CompiledLexicalSense> {
        self.compiled_senses.get(id)
    }

    pub fn form(&self, id: &FormId) -> Option<&Form> {
        self.forms.get(id)
    }

    pub fn paradigm(&self, id: &crate::ParadigmId) -> Option<&MorphologyParadigm> {
        self.paradigms.get(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LanguageLoadError {
    #[error("io error reading {path}: {kind:?}")]
    Io {
        path: PathBuf,
        kind: std::io::ErrorKind,
    },
    #[error("parse error reading {path}: {message}")]
    Parse { path: PathBuf, message: String },
    #[error("unsupported language manifest schema: {schema}")]
    UnsupportedSchema { schema: u32 },
    #[error("validation error: {0}")]
    Validation(LanguageValidationIssue),
    #[error("duplicate {kind} id: {id}")]
    DuplicateId { kind: &'static str, id: String },
    #[error("unsafe package path: {relative}")]
    UnsafePath { relative: PathBuf },
    #[error("language compile error: {0}")]
    Compile(#[from] LanguageCompileError),
}

pub struct LanguagePackageLoader;

impl LanguagePackageLoader {
    pub fn load(
        &self,
        root: &Path,
        catalog: &ConceptCatalog,
    ) -> Result<LanguageModel, LanguageLoadError> {
        let manifest: LanguagePackageManifest = read_ron(&root.join("manifest.ron"))?;
        if manifest.schema != SUPPORTED_MANIFEST_SCHEMA {
            return Err(LanguageLoadError::UnsupportedSchema {
                schema: manifest.schema,
            });
        }
        let lexemes: Vec<Lexeme> = read_ron(&safe_package_path(root, &manifest.lexemes)?)?;
        let senses: Vec<LexicalSense> = read_ron(&safe_package_path(root, &manifest.senses)?)?;
        let forms: Vec<Form> = read_ron(&safe_package_path(root, &manifest.forms)?)?;
        let paradigms: Vec<MorphologyParadigm> =
            read_ron(&safe_package_path(root, &manifest.paradigms)?)?;
        let lexemes = collect_unique("lexeme", lexemes, |lexeme| lexeme.id.clone())?;
        let senses = collect_unique("sense", senses, |sense| sense.id.clone())?;
        let forms = collect_unique("form", forms, |form| form.id.clone())?;
        let paradigms = collect_unique("paradigm", paradigms, |paradigm| paradigm.id.clone())?;
        let form_index = FormIndex::build(forms.values().cloned());
        let compiled_senses = senses
            .values()
            .map(|sense| {
                let compiled = compile_lexical_sense(sense)?;
                Ok((compiled.id.clone(), compiled))
            })
            .collect::<Result<BTreeMap<_, _>, LanguageCompileError>>()?;
        let sense_index = SenseIndex::build(senses.values().cloned());
        let model_hash = canonical_hash(&(&manifest, &lexemes, &senses, &forms, &paradigms));
        let model = LanguageModel {
            manifest,
            lexemes,
            senses,
            compiled_senses,
            forms,
            paradigms,
            form_index,
            sense_index,
            model_hash,
        };
        LanguageModelValidator
            .validate(&model, catalog)
            .map_err(|error| match error {
                crate::validation::LanguageValidationError::Issue(issue) => {
                    LanguageLoadError::Validation(issue)
                }
            })?;
        Ok(model)
    }
}

fn safe_package_path(root: &Path, relative: &str) -> Result<PathBuf, LanguageLoadError> {
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(LanguageLoadError::UnsafePath {
            relative: relative.to_path_buf(),
        });
    }
    Ok(root.join(relative))
}

fn collect_unique<K, V>(
    kind: &'static str,
    values: impl IntoIterator<Item = V>,
    key: impl Fn(&V) -> K,
) -> Result<BTreeMap<K, V>, LanguageLoadError>
where
    K: Ord + Clone + ToString,
{
    let mut output = BTreeMap::new();
    for value in values {
        let id = key(&value);
        if output.insert(id.clone(), value).is_some() {
            return Err(LanguageLoadError::DuplicateId {
                kind,
                id: id.to_string(),
            });
        }
    }
    Ok(output)
}

fn read_ron<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, LanguageLoadError> {
    let source = fs::read_to_string(path).map_err(|error| LanguageLoadError::Io {
        path: path.to_path_buf(),
        kind: error.kind(),
    })?;
    ron::from_str(&source).map_err(|error| LanguageLoadError::Parse {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}
