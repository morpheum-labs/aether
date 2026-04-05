/// Intel TDX placeholder — real integration would produce hardware quotes.
#[derive(Clone, Debug)]
pub struct TdxBackend;

impl TdxBackend {
    pub fn is_available() -> bool {
        false
    }
}
