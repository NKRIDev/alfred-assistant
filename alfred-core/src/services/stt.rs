use std::sync::Arc;
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};

/*
Converts an audio stream into text
in string format
I use Whisper by OpenIA,
github : https://github.com/openai/whisper
 */
#[derive(Clone)]
pub struct SttService {
    ctx: Arc<WhisperContext>,
}

impl SttService {

    /*
    Create constructor and init whisper
     */
    pub fn new(model_path: &str) -> Result<Self, String> {
        let ctx = WhisperContext::new_with_params(
            model_path,
            WhisperContextParameters::default(),
        ).map_err(|e| format!("Whisper init error: {:?}", e))?;

        Ok(Self { ctx: Arc::new(ctx) })
    }

    /*
    Implementation of the transcription
    Note: Whisper expects the following audio parameters :
    PCM mono, f32, 16kHz
     */
    pub fn transcribe(&self, audio_samples: &[f32]) -> Result<String, String> {
        let mut state = self.ctx.create_state()
            .map_err(|e| format!("Whisper state error: {:?}", e))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("fr"));
        params.set_print_progress(false);
        params.set_print_special(false);
        params.set_print_realtime(false);

        state.full(params, audio_samples)
            .map_err(|e| format!("Whisper transcribe error: {:?}", e))?;

        let num_segments = state.full_n_segments();

        let mut text = String::new();
        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i) {
                if let Ok(seg_text) = segment.to_str() {
                    text.push_str(seg_text);
                }
            }
        }

        Ok(text.trim().to_string())
    }

    /*
    To test the transcription, I added a .wav file that will be
    played and then transcribed by the model used by Whisper
     */
    pub fn load_wav_as_f32(path: &str) -> Vec<f32> {
        let mut reader = hound::WavReader::open(path).unwrap();
        reader.samples::<i16>()
            .map(|s| s.unwrap() as f32 / i16::MAX as f32)
            .collect()
    }
}