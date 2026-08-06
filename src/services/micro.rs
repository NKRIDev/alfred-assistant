use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::io::{self, Write};

/*
Manages the micro part
 */
pub struct MicroService;

impl MicroService {

    /*
    Manages the microphone recording part
     */
    pub fn record() -> (Vec<f32>, u32) {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("No microphone found");
        let config = device.default_input_config().expect("Invalid microphone configuration");
        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;

        println!("Micro détecté, sample rate: {}, channels: {}", sample_rate, channels);
        print!("[MICRO] Appuyez sur Entrée pour COMMENCER l'enregistrement...");
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();

        let samples = Arc::new(Mutex::new(Vec::<f32>::new()));
        let samples_clone = samples.clone();

        let stream = device.build_input_stream(
            config.into(),
            move |data: &[f32], _| {
                let mut buf = samples_clone.lock().unwrap();
                for frame in data.chunks(channels) {
                    let mono_sample: f32 = frame.iter().sum::<f32>() / channels as f32;
                    buf.push(mono_sample);
                }
            },
            |err| eprintln!("Micro stream error : {}", err),
            None,
        ).expect("Unable to create microphone stream");

        stream.play().expect("Impossible de démarrer le micro");
        println!("[MICRO] Enregistrement en cours... Appuie sur Entrée pour STOPPER");

        let mut input2 = String::new();
        io::stdin().read_line(&mut input2).ok();

        drop(stream);

        let final_samples = Arc::try_unwrap(samples).unwrap().into_inner().unwrap();
        println!("[MICRO] Recording completed !");

        (final_samples, sample_rate)
    }

    /*
    Sampled audio at 16khz frequency for whisper
     */
    pub fn resample_to_16k(input: &[f32], input_rate: u32) -> Vec<f32> {
        if input_rate == 16000 {
            return input.to_vec();
        }

        let ratio = 16000.0 / input_rate as f64;
        let output_len = (input.len() as f64 * ratio) as usize;
        let mut output = Vec::with_capacity(output_len);

        for i in 0..output_len {
            let src_pos = i as f64 / ratio;
            let src_idx = src_pos as usize;
            let frac = (src_pos - src_idx as f64) as f32;

            if src_idx + 1 < input.len() {
                let sample = input[src_idx] * (1.0 - frac) + input[src_idx + 1] * frac;
                output.push(sample);
            }
            else if src_idx < input.len() {
                output.push(input[src_idx]);
            }
        }

        output
    }
}