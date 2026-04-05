/// AMD SEV-SNP placeholder.
#[derive(Clone, Debug)]
pub struct SevBackend;

impl SevBackend {
    pub fn is_available() -> bool {
        false
    }
}
