use clap::ValueEnum;
use lexflex_lingua::solve::EvidencePolicy;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum EvidencePolicyArg {
    Required,
    Optional,
    Ignore,
}

impl From<EvidencePolicyArg> for EvidencePolicy {
    fn from(value: EvidencePolicyArg) -> Self {
        match value {
            EvidencePolicyArg::Required => EvidencePolicy::Required,
            EvidencePolicyArg::Optional => EvidencePolicy::Optional,
            EvidencePolicyArg::Ignore => EvidencePolicy::Ignore,
        }
    }
}
