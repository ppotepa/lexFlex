use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

string_id!(LexemeId);
string_id!(LexicalSenseId);
string_id!(FormId);
string_id!(ParadigmId);
string_id!(ValencySlotId);
string_id!(SurfaceRelationId);
string_id!(FeatureName);
string_id!(FeatureValue);
string_id!(CategoryTypeVariableId);
string_id!(MeaningTemplateId);
