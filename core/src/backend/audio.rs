use generational_arena::{Arena, Index};

pub mod swf {
    pub use swf::{read, AudioCompression, Sound, SoundFormat, SoundStreamInfo};
}

pub type AudioStreamHandle = Index;
pub type SoundHandle = Index;

type Error = Box<std::error::Error>;

pub trait AudioBackend {
    fn register_sound(&mut self, swf_sound: &swf::Sound) -> Result<SoundHandle, Error>;
    fn register_stream(&mut self, stream_info: &swf::SoundStreamInfo) -> AudioStreamHandle;
    fn play_sound(&mut self, sound: SoundHandle);
    fn queue_stream_samples(&mut self, handle: AudioStreamHandle, samples: &[u8]);
}

pub struct NullAudioBackend {
    sounds: Arena<()>,
    streams: Arena<()>,
}

impl NullAudioBackend {
    pub fn new() -> NullAudioBackend {
        NullAudioBackend {
            streams: Arena::new(),
            sounds: Arena::new(),
        }
    }
}

impl AudioBackend for NullAudioBackend {
    fn register_sound(&mut self, sound: &swf::Sound) -> Result<SoundHandle, Error> {
        Ok(self.sounds.insert(()))
    }

    fn play_sound(&mut self, sound: SoundHandle) {}

    fn register_stream(&mut self, _stream_info: &swf::SoundStreamInfo) -> AudioStreamHandle {
        self.streams.insert(())
    }

    fn queue_stream_samples(&mut self, _handle: AudioStreamHandle, _samples: &[u8]) {
        // Noop
    }
}

pub struct AdpcmDecoder<R: std::io::Read> {
    inner: swf::read::Reader<R>,
    is_stereo: bool,
    bits_per_sample: u8,
    left_predictor: i16,
    left_step_index: i8,
    left_step: u16,
    right_predictor: i16,
    right_step_index: i8,
    right_step: u16,
}

// impl<R: std::io::Read> AdpcmDecoder<R> {
//     const INDEX_TABLE: [&'static [i8]; 4] = [
//         &[-1, 2, -1, 2],
//         &[-1, -1, 2, 4, -1, -1, 2, 4],
//         &[-1, -1, -1, -1, 2, 4, 6, 8, -1, -1, -1, -1, 2, 4, 6, 8],
//         &[
//             -1, -1, -1, -1, -1, -1, -1, -1, 1, 2, 4, 6, 8, 10, 13, 16, -1, -1, -1, -1, -1, -1, -1,
//             -1, 1, 2, 4, 6, 8, 10, 13, 16,
//         ],
//     ];

//     const STEP_TABLE: [u16; 89] = [
//         7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60,
//         66, 73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371,
//         408, 449, 494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878,
//         2066, 2272, 2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845,
//         8630, 9493, 10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086,
//         29794, 32767,
//     ];

//     fn new(inner: R, bits_per_sample: u8, is_stereo: bool) -> Result<Self, Error> {
//         use self::swf::read::SwfRead;

//         let reader = swf::read::Reader::new(inner, 1);
//         let bits_per_sample = reader.read_ubits(2)? + 2;

//         let left_predictor = reader.read_i16()?;
//         let left_step_index = reader.read_ubits(6)?;
//         let left_step = Self::STEP_TABLE[self.left_step_index as usize];
//         let right_predictor = reader.read_i16()?;
//         let right_step_index = reader.read_ubits(6)?;
//         let right_step = Self::STEP_TABLE[self.right_step_index as usize];
//         Ok(Self {
//             inner: reader,
//             is_stereo,
//             bits_per_sample,
//             left_predictor,
//             left_step,
//             left_step_index,
//             right_predictor,
//             right_step,
//             right_step_index,
//         })
//     }

//     fn next(&mut self) -> Result<(i16, i16), Error> {
//         use self::swf::read::SwfRead;
//         let data = self.inner.read_ubits(self.bits_per_sample)?;

//         assert!(data < (1 << self.bits_per_sample));
//         self.left_step_index +=
//             Self::INDEX_TABLE[(self.bits_per_sample - 2) as usize][data as usize];
//         if self.left_step_index < 0 {
//             self.left_step_index = 0;
//         } else if self.left_step_index >= Self::STEP_TABLE.len() as i8 {
//             self.left_step_index = Self::STEP_TABLE.len() as i8 - 1;
//         }
//         let diff = (2 * data + 1) * self.left_step / 8;
//         self.left_predictor.saturating_add(diff);
//         self.left_step = Self::STEP_TABLE[self.left_step_index as usize];

//         if self.is_stereo {
//             assert!(data < (1 << self.bits_per_sample));
//             self.right_step_index +=
//                 Self::INDEX_TABLE[(self.bits_per_sample - 2) as usize][data as usize];
//             if self.right_step_index < 0 {
//                 self.right_step_index = 0;
//             } else if self.right_step_index >= Self::STEP_TABLE.len() as i8 {
//                 self.right_step_index = Self::STEP_TABLE.len() as i8 - 1;
//             }
//             let diff = (2 * data + 1) * self.right_step / 8;
//             self.right_predictor.saturating_add(diff);
//             self.right_step = Self::STEP_TABLE[self.right_step_index as usize];
//             Ok((self.left_predictor, self.right_predictor))
//         } else {
//             Ok((self.left_predictor, self.left_predictor))
//         }
//     }
// }
