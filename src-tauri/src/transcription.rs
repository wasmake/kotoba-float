use crate::auth::BLOCK_REASON;
pub trait Transcriber: Send + Sync {
    fn transcribe(&self, _audio: &[f32]) -> Result<String, String>;
}
pub struct UnsupportedSubscriptionTranscriber;
impl Transcriber for UnsupportedSubscriptionTranscriber {
    fn transcribe(&self, _: &[f32]) -> Result<String, String> {
        Err(BLOCK_REASON.into())
    }
}
