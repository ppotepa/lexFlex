use lexflex_language::{LanguageId, LanguageLoadError, LanguageModel, LanguagePackageLoader};
use lexflex_model::{CanonicalDigest, CanonicalHashError, ConceptCatalog};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct LanguageRegistry {
    pub models: BTreeMap<LanguageId, Arc<LanguageModel>>,
    pub registry_hash: CanonicalDigest,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LanguageRegistryError {
    #[error("language load error: {0}")]
    Load(#[from] LanguageLoadError),
    #[error("missing language package: {root}")]
    MissingPackage { root: PathBuf },
    #[error("language directory mismatch: directory={directory}, manifest={manifest}")]
    DirectoryLanguageMismatch {
        directory: String,
        manifest: LanguageId,
    },
    #[error("duplicate language package: {0}")]
    DuplicateLanguage(LanguageId),
    #[error("duplicate package id: {0}")]
    DuplicatePackageId(String),
    #[error("canonical hash error: {0}")]
    CanonicalHash(#[from] CanonicalHashError),
}

impl LanguageRegistry {
    pub fn load(root: &Path, catalog: Arc<ConceptCatalog>) -> Result<Self, LanguageRegistryError> {
        let loader = LanguagePackageLoader;
        let mut models: BTreeMap<LanguageId, Arc<LanguageModel>> = BTreeMap::new();

        for language_root in discover_language_roots(root)? {
            let model = loader
                .load(&language_root, catalog.as_ref())
                .map_err(LanguageRegistryError::from)?;
            let language = model.manifest.language.clone();
            let directory = language_root
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned)
                .unwrap_or_else(|| language_root.display().to_string());
            if directory != language.as_str() {
                return Err(LanguageRegistryError::DirectoryLanguageMismatch {
                    directory,
                    manifest: language,
                });
            }
            let package_id = model.manifest.package_id.clone();
            if models.contains_key(&language) {
                return Err(LanguageRegistryError::DuplicateLanguage(language));
            }
            for existing in models.values() {
                if existing.as_ref().manifest.package_id == package_id {
                    return Err(LanguageRegistryError::DuplicatePackageId(
                        package_id.to_string(),
                    ));
                }
            }
            models.insert(language, Arc::new(model));
        }

        if models.is_empty() {
            return Err(LanguageRegistryError::MissingPackage {
                root: root.to_path_buf(),
            });
        }

        let registry_hash = registry_hash_for(&models)?;

        Ok(Self {
            models,
            registry_hash,
        })
    }

    pub fn get(&self, language: &LanguageId) -> Option<&Arc<LanguageModel>> {
        self.models.get(language)
    }
}

pub fn workspace_language_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join("data/languages")
}

fn discover_language_roots(root: &Path) -> Result<Vec<PathBuf>, LanguageRegistryError> {
    let mut roots = Vec::new();
    let entries = fs::read_dir(root).map_err(|error| {
        LanguageRegistryError::Load(LanguageLoadError::Io {
            path: root.to_path_buf(),
            kind: error.kind(),
        })
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            LanguageRegistryError::Load(LanguageLoadError::Io {
                path: root.to_path_buf(),
                kind: error.kind(),
            })
        })?;
        let path = entry.path();
        if path.is_dir() && path.join("manifest.ron").exists() {
            roots.push(path);
        }
    }
    roots.sort();
    Ok(roots)
}

fn registry_hash_for(
    models: &BTreeMap<LanguageId, Arc<LanguageModel>>,
) -> Result<CanonicalDigest, CanonicalHashError> {
    lexflex_model::canonical_hash(
        &models
            .iter()
            .map(|(id, model)| {
                (
                    id,
                    (
                        &model.manifest.package_id,
                        &model.manifest.language,
                        &model.model_hash,
                        model.lexemes.len(),
                        model.senses.len(),
                        model.compiled_senses.len(),
                        model.forms.len(),
                        model.paradigms.len(),
                    ),
                )
            })
            .collect::<BTreeMap<_, _>>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexflex_language::{LanguagePackageManifest, Lexeme};
    use lexflex_model::{
        ConceptCatalog, ConceptId, ConceptKind, ConceptSchema, LanguageId, SemanticType,
    };
    use std::collections::BTreeMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        std::env::temp_dir().join(format!("lexflex-language-registry-{stamp}"))
    }

    fn catalog() -> Arc<ConceptCatalog> {
        let mut concepts = BTreeMap::new();
        concepts.insert(
            ConceptId::new_unchecked("CITY"),
            ConceptSchema {
                id: ConceptId::new_unchecked("CITY"),
                kind: ConceptKind::EntityType,
                parameters: BTreeMap::new(),
                result_type: SemanticType::Predicate(Box::new(SemanticType::EntityOf(
                    ConceptId::new_unchecked("CITY"),
                ))),
            },
        );
        Arc::new(ConceptCatalog {
            concepts,
            entities: BTreeMap::new(),
            parents: BTreeMap::new(),
        })
    }

    #[test]
    fn discovers_packages_from_manifest_files() {
        let root = temp_root();
        let en = root.join("en");
        let pl = root.join("pl");
        std::fs::create_dir_all(&en).expect("create en");
        std::fs::create_dir_all(&pl).expect("create pl");
        std::fs::write(
            en.join("manifest.ron"),
            r#"(
                schema: 1,
                package_id: "lexflex:language:en:test",
                language: "en",
                lexemes: "lexemes.ron",
                senses: "senses.ron",
                forms: "forms.ron",
                paradigms: "paradigms.ron",
            )"#,
        )
        .expect("write en manifest");
        std::fs::write(
            pl.join("manifest.ron"),
            r#"(
                schema: 1,
                package_id: "lexflex:language:pl:test",
                language: "pl",
                lexemes: "lexemes.ron",
                senses: "senses.ron",
                forms: "forms.ron",
                paradigms: "paradigms.ron",
            )"#,
        )
        .expect("write pl manifest");

        for dir in [&en, &pl] {
            std::fs::write(dir.join("lexemes.ron"), "[]").expect("write lexemes");
            std::fs::write(dir.join("senses.ron"), "[]").expect("write senses");
            std::fs::write(dir.join("forms.ron"), "[]").expect("write forms");
            std::fs::write(dir.join("paradigms.ron"), "[]").expect("write paradigms");
        }

        let registry = LanguageRegistry::load(&root, catalog()).expect("load registry");
        assert!(registry.get(&LanguageId::new_unchecked("en")).is_some());
        assert!(registry.get(&LanguageId::new_unchecked("pl")).is_some());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn duplicate_package_ids_are_rejected() {
        let root = temp_root();
        let en = root.join("en");
        let pl = root.join("pl");
        std::fs::create_dir_all(&en).expect("create en");
        std::fs::create_dir_all(&pl).expect("create pl");

        for (dir, language) in [(&en, "en"), (&pl, "pl")] {
            std::fs::write(
                dir.join("manifest.ron"),
                format!(
                    r#"(
                schema: 1,
                package_id: "lexflex:language:duplicate",
                language: "{language}",
                lexemes: "lexemes.ron",
                senses: "senses.ron",
                forms: "forms.ron",
                paradigms: "paradigms.ron",
            )"#
                ),
            )
            .expect("write manifest");
            std::fs::write(dir.join("lexemes.ron"), "[]").expect("write lexemes");
            std::fs::write(dir.join("senses.ron"), "[]").expect("write senses");
            std::fs::write(dir.join("forms.ron"), "[]").expect("write forms");
            std::fs::write(dir.join("paradigms.ron"), "[]").expect("write paradigms");
        }

        let err = LanguageRegistry::load(&root, catalog()).expect_err("duplicate package id");
        assert!(matches!(err, LanguageRegistryError::DuplicatePackageId(_)));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn registry_hash_tracks_model_content() {
        let mut models: BTreeMap<LanguageId, Arc<LanguageModel>> = BTreeMap::new();
        let en = LanguageId::new_unchecked("en");
        let base_manifest = LanguagePackageManifest {
            schema: 1,
            package_id: "lexflex:language:en:test".into(),
            language: en.clone(),
            lexemes: "lexemes.ron".into(),
            senses: "senses.ron".into(),
            forms: "forms.ron".into(),
            paradigms: "paradigms.ron".into(),
        };
        models.insert(
            en.clone(),
            Arc::new(LanguageModel {
                manifest: base_manifest.clone(),
                lexemes: BTreeMap::from([(
                    lexflex_language::LexemeId::new_unchecked("lexeme:test:one"),
                    Lexeme {
                        id: lexflex_language::LexemeId::new_unchecked("lexeme:test:one"),
                        language: en.clone(),
                        lemma: "alpha".into(),
                        normalized_lemma: "alpha".into(),
                    },
                )]),
                senses: BTreeMap::new(),
                compiled_senses: BTreeMap::new(),
                forms: BTreeMap::new(),
                paradigms: BTreeMap::new(),
                form_index: Default::default(),
                sense_index: Default::default(),
                model_hash: CanonicalDigest::new(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .expect("valid digest"),
            }),
        );
        let first = registry_hash_for(&models);
        models.insert(
            en,
            Arc::new(LanguageModel {
                manifest: base_manifest,
                lexemes: BTreeMap::new(),
                senses: BTreeMap::new(),
                compiled_senses: BTreeMap::new(),
                forms: BTreeMap::new(),
                paradigms: BTreeMap::new(),
                form_index: Default::default(),
                sense_index: Default::default(),
                model_hash: CanonicalDigest::new(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .expect("valid digest"),
            }),
        );
        let second = registry_hash_for(&models);
        assert_ne!(first, second);
    }

    #[test]
    fn directory_language_mismatch_is_rejected() {
        let root = temp_root();
        let en = root.join("en");
        std::fs::create_dir_all(&en).expect("create en");
        std::fs::write(
            en.join("manifest.ron"),
            r#"(
                schema: 1,
                package_id: "lexflex:language:en:test",
                language: "pl",
                lexemes: "lexemes.ron",
                senses: "senses.ron",
                forms: "forms.ron",
                paradigms: "paradigms.ron",
            )"#,
        )
        .expect("write manifest");
        std::fs::write(en.join("lexemes.ron"), "[]").expect("write lexemes");
        std::fs::write(en.join("senses.ron"), "[]").expect("write senses");
        std::fs::write(en.join("forms.ron"), "[]").expect("write forms");
        std::fs::write(en.join("paradigms.ron"), "[]").expect("write paradigms");

        let err = LanguageRegistry::load(&root, catalog()).expect_err("directory mismatch");
        assert!(matches!(
            err,
            LanguageRegistryError::DirectoryLanguageMismatch { .. }
        ));

        let _ = std::fs::remove_dir_all(root);
    }
}
