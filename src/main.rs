use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
// use hound;
// use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const SAMPLE_RATE: u32 = 44100;
// const CHANNELS: u16 = 1;
const RECORD_DURATION_SECS: u64 = 5;
// const RECORDING_FILE: &str = "recording.wav";

fn main() {
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("Failed to get default input device");

    let input_config = input_device.default_input_config().expect("Failed to get default input config");
    let sample_rate = input_config.sample_rate().0;
    let record_duration_samples = sample_rate * RECORD_DURATION_SECS as u32;

    let recording = Arc::new(Mutex::new(Vec::new()));
    let recording_writer = recording.clone();

    let input_stream = input_device.build_input_stream(
        &input_config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut recording = recording_writer.lock().unwrap();
            for &sample in data.iter() {
                recording.push(sample);
                if recording.len() as u32 >= record_duration_samples {
                    break;
                }
            }
        },
        move |err| {
            eprintln!("An error occurred on the input stream: {}", err);
        },
    ).unwrap();

    input_stream.play().unwrap();
    std::thread::sleep(Duration::from_secs(RECORD_DURATION_SECS));

    // let spec = hound::WavSpec {
    //     channels: CHANNELS,
    //     sample_rate: sample_rate as u32,
    //     bits_per_sample: 32,
    //     sample_format: hound::SampleFormat::Float,
    // };

    // let mut writer = hound::WavWriter::new(BufWriter::new(std::fs::File::create(RECORDING_FILE).unwrap()), spec).unwrap();
    let mut recording = recording.lock().unwrap();
    // for &sample in recording.iter() {
    //     writer.write_sample(sample).unwrap();
    // }
    // writer.finalize().unwrap();

    println!("Recording complete.");


    remove_dc_offset(&mut recording);
    println!("Offsetting complete.");

    normalize_audio(&mut recording, 1.0);
    println!("normalization complete.");

    let num_lsb = 8; // Adjust this value depending on the desired quality of randomness
    let output_length = 5; // Set the desired output length (in bytes)
    let random_data = extract_random_data(&recording, num_lsb, output_length);

    print_random_data_as_hex(&random_data);
}

fn remove_dc_offset(samples: &mut Vec<f32>) {
    let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
    samples.iter_mut().for_each(|sample| *sample -= mean);
}

fn normalize_audio(samples: &mut Vec<f32>, max_level: f32) {
    let max_sample = samples
        .iter()
        .cloned()
        .map(f32::abs)
        .fold(f32::MIN, f32::max);
    let normalization_factor = max_level / max_sample;
    samples.iter_mut().for_each(|sample| *sample *= normalization_factor);
}

fn extract_random_data(samples: &[f32], num_lsb: u32, output_length: usize) -> Vec<u8> {
    let mut random_data = Vec::with_capacity(output_length);

    let samples_per_byte = (samples.len() - 1) / output_length;

    for i in (1..samples.len()).step_by(samples_per_byte) {
        let difference = samples[i] - samples[i - 1];
        let difference_as_int = difference.to_bits();
        let lsb_bits = difference_as_int & ((1 << num_lsb) - 1);

        random_data.push(lsb_bits as u8);

        if random_data.len() >= output_length {
            break;
        }
    }

    random_data
}

fn print_random_data_as_hex(random_data: &[u8]) {
    let hex_string = random_data
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>();

    println!("Random data (hex): {}", hex_string);
}