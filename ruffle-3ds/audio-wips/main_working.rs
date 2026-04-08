//! Audio Filters example.
//!
//! This example showcases basic audio functionality using [`Ndsp`].

#![feature(allocator_api)]

use std::f32::consts::PI;

use ctru::linear::LinearAllocator;
use ctru::prelude::*;
use ctru::services::ndsp::{
    wave::{Status, Wave},
    AudioFormat, AudioMix, InterpolationType, Ndsp, OutputMode,
};

// Configuration for the NDSP process and channels.
const SAMPLE_RATE: usize = 22050;
const SAMPLES_PER_BUF: usize = SAMPLE_RATE / 10; // 2205
const BYTES_PER_SAMPLE: usize = AudioFormat::PCM16Stereo.size();
const AUDIO_WAVE_LENGTH: usize = SAMPLES_PER_BUF * BYTES_PER_SAMPLE;

// Note frequencies.
const NOTEFREQ: [f32; 7] = [220., 440., 880., 1760., 3520., 7040., 14080.];

fn fill_buffer(audio_data: &mut [u8], frequency: f32) {
    // The audio format is Stereo PCM16.
    // As such, a sample is made up of 2 "Mono" samples (2 * i16), one for each channel (left and right).
    let formatted_data = bytemuck::cast_slice_mut::<_, [i16; 2]>(audio_data);

    for (i, chunk) in formatted_data.iter_mut().enumerate() {
        // This is a simple sine wave, with a frequency of `frequency` Hz, and an amplitude 30% of maximum.
        let sample: f32 = (frequency * (i as f32 / SAMPLE_RATE as f32) * 2. * PI).sin();
        let amplitude = 0.3 * i16::MAX as f32;

        let result = (sample * amplitude) as i16;

        // Stereo samples are interleaved: left and right channels.
        *chunk = [result, result];
    }
}

fn main() {
    let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
    let mut hid = Hid::new().expect("Couldn't obtain HID controller");
    let apt = Apt::new().expect("Couldn't obtain APT controller");
    let _console = Console::new(gfx.top_screen.borrow_mut());

    let mut note: usize = 4;

    // Filter names to display.
    let filter_names = [
        "None",
        "Low-Pass",
        "High-Pass",
        "Band-Pass",
        "Notch",
        "Peaking",
    ];

    let mut filter: i32 = 0;

    // We set up two wave buffers and alternate between the two,
    // effectively streaming an infinitely long sine wave.

    // We create a buffer on the LINEAR memory that will hold our audio data.
    // It's necessary for the buffer to live on the LINEAR memory sector since it needs to be accessed by the DSP processor.
    let mut audio_data1: Box<[_], _> = Box::new_in([0u8; AUDIO_WAVE_LENGTH], LinearAllocator);

    // Fill the buffer with the first set of data. This simply writes a sine wave into the buffer.
    fill_buffer(&mut audio_data1, NOTEFREQ[4]);

    // Clone the original buffer to obtain an equal buffer on the LINEAR memory used for double buffering.
    let audio_data2 = audio_data1.clone();

    // Setup two wave info objects with the correct configuration and ownership of the audio data.
    let mut wave_info1 = Wave::new(audio_data1, AudioFormat::PCM16Stereo, false);
    let mut wave_info2 = Wave::new(audio_data2, AudioFormat::PCM16Stereo, false);

    // Setup the NDSP service and its configuration.

    let mut ndsp = Ndsp::new().expect("Couldn't obtain NDSP controller");
    ndsp.set_output_mode(OutputMode::Stereo);

    // Channel configuration. We use channel zero but any channel would do just fine.
    let mut channel_zero = ndsp.channel(0).unwrap();
    channel_zero.set_interpolation(InterpolationType::Linear);
    channel_zero.set_sample_rate(SAMPLE_RATE as f32);
    channel_zero.set_format(AudioFormat::PCM16Stereo);

    // Output at 100% on the first pair of left and right channels.
    let mix = AudioMix::default();
    channel_zero.set_mix(&mix);

    // First set of queueing for the two buffers. The second one will only play after the first one has ended.
    channel_zero.queue_wave(&mut wave_info1).unwrap();
    channel_zero.queue_wave(&mut wave_info2).unwrap();

    println!("\x1b[1;1HPress up/down to change tone frequency");
    println!("\x1b[2;1HPress left/right to change filter");
    println!("\x1b[4;1Hnote = {} Hz        ", NOTEFREQ[note]);
    println!(
        "\x1b[5;1Hfilter = {}         ",
        filter_names[filter as usize]
    );

    println!("\x1b[29;16HPress Start to exit");

    let mut altern = true; // true is wave_info1, false is wave_info2

    while apt.main_loop() {
        hid.scan_input();
        let keys_down = hid.keys_down();

        if keys_down.contains(KeyPad::START) {
            break;
        }

        // Note frequency controller using the buttons.
        if keys_down.intersects(KeyPad::DOWN) {
            note = note.saturating_sub(1);
        } else if keys_down.intersects(KeyPad::UP) {
            note = std::cmp::min(note + 1, NOTEFREQ.len() - 1);
        }

        // Filter controller using the buttons.
        let mut update_params = false;
        if keys_down.intersects(KeyPad::LEFT) {
            filter -= 1;
            filter = filter.rem_euclid(filter_names.len() as _);

            update_params = true;
        } else if keys_down.intersects(KeyPad::RIGHT) {
            filter += 1;
            filter = filter.rem_euclid(filter_names.len() as _);

            update_params = true;
        }

        println!("\x1b[4;1Hnote = {} Hz        ", NOTEFREQ[note]);
        println!(
            "\x1b[5;1Hfilter = {}         ",
            filter_names[filter as usize]
        );

        if update_params {
            match filter {
                1 => channel_zero.iir_biquad_set_params_low_pass_filter(1760., 0.707),
                2 => channel_zero.iir_biquad_set_params_high_pass_filter(1760., 0.707),
                3 => channel_zero.iir_biquad_set_params_band_pass_filter(1760., 0.707),
                4 => channel_zero.iir_biquad_set_params_notch_filter(1760., 0.707),
                5 => channel_zero.iir_biquad_set_params_peaking_equalizer(1760., 0.707, 3.),
                _ => channel_zero.iir_biquad_set_enabled(false),
            }
        }

        // Double buffer alternation depending on the one used.
        let current: &mut Wave<_> = if altern {
            &mut wave_info1
        } else {
            &mut wave_info2
        };

        // If the current buffer has finished playing, we can refill it with new data and re-queue it.
        let status = current.status();
        if let Status::Done = status {
            fill_buffer(current.get_buffer_mut().unwrap(), NOTEFREQ[note]);

            channel_zero.queue_wave(current).unwrap();

            altern = !altern;
        }

        gfx.wait_for_vblank();
    }
}