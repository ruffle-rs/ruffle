#![feature(allocator_api)]

use ctru::prelude::*;
use ctru::services::ndsp::{AudioFormat, AudioMix, InterpolationType, Ndsp, OutputMode};
use std::sync::Arc;
use ctru::linear::LinearAllocator;

use ruffle_core::PlayerBuilder;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_render_wgpu::backend::request_adapter_and_device;

use ruffle_render_wgpu::backend::WgpuRenderBackend;
use ruffle_render_wgpu::descriptors::Descriptors;
use ruffle_render_wgpu::target::TextureTarget;

use symphonia::core::audio::{AudioBuffer, AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::{MediaSourceStream, MediaSource};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};


const SAMPLE_RATE: f32 = 22050.0;

fn main() {
    ctru::applets::error::set_panic_hook(false);

    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let gfx = Gfx::new().unwrap();
    let _console = Console::new(gfx.top_screen.borrow_mut());

    println!("Hello, World!");
    println!("\x1b[29;16HPress Start to exit");

    let player = PlayerBuilder::new()
        // .with_audio()
        .build();


    let mut ndsp = Ndsp::new().expect("Couldn't obtain NDSP controller");
    ndsp.set_output_mode(OutputMode::Stereo);

    let mut channel_zero = ndsp.channel(0).unwrap();
    channel_zero.set_interpolation(InterpolationType::Linear);
    channel_zero.set_sample_rate(SAMPLE_RATE);
    channel_zero.set_format(AudioFormat::PCM16Stereo);

    let mix = AudioMix::default();
    channel_zero.set_mix(&mix);

    let mp3_src = std::fs::File::open("romfs:/example.mp3").expect("Failed to open mp3");
    let mss = MediaSourceStream::new(Box::new(mp3_src), Default::default());

    let mut format_reader = get_probe().format(
        &Hint::new().with_extension("mp3").mime_type("audio/mpeg"),
        mss,
        &Default::default(),
        &Default::default(),
    ).expect("Failed to format mp3").format;

    let track = format_reader.default_track().expect("couldn't find mp3 track");
    let mut decoder = get_codecs().make(&track.codec_params, &Default::default()).expect("Failed to create decoder");

    let mut pcm_packets = Vec::new_in(LinearAllocator);

     loop {
         let packet = format_reader.next_packet()?;
         if packet.track_id() != track.id() {
             continue;
         }

         match decoder.decode(&packet)
    }

    let mut paused = false;
    while apt.main_loop() {
        gfx.wait_for_vblank();

        hid.scan_input();
        if hid.keys_down().contains(KeyPad::START) {
            break;
        }
        else if hid.keys_down().contains(KeyPad::SELECT) {
            paused = !paused;
            channel_zero.set_paused(paused);
        }
    }
}

