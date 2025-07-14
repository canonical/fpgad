#[derive(Debug, thiserror::Error)]
pub enum FpgadSoftenerError {
    #[error("FpgadSoftenerError::DfxMgr: {0}")]
    DfxMgr(std::io::Error),
}
