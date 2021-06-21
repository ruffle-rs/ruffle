//! Decoder types

bitflags! {
    /// Options which influence the decoding of a bitstream.
    pub struct DecoderOption : u8 {
        /// Attempt to decode the video as a Sorenson Spark bitstream.
        ///
        /// Sorenson Spark is a modified H.263 video format notably used in early
        /// iterations of Macromedia Flash Player. It was replaced with On2 VP6,
        /// and later on, standard H.264.
        const SORENSON_SPARK_BITSTREAM = 0b1;

        /// Whether or not the use of Annex O's Temporal, SNR, and Spatial
        /// Scalability mode has been negotiated.
        const USE_SCALABILITY_MODE = 0b10;
    }
}
