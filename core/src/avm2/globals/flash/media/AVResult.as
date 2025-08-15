package flash.media {
    [API("688")]
    public class AVResult {
        public static const END_OF_PERIOD:int = -1;
        public static const SUCCESS:int = 0;
        public static const ASYNC_OPERATION_IN_PROGRESS:int = 1;
        public static const EOF:int = 2;
        public static const DECODER_FAILED:int = 3;
        public static const DEVICE_OPEN_ERROR:int = 4;
        public static const FILE_NOT_FOUND:int = 5;
        public static const GENERIC_ERROR:int = 6;
        public static const IRRECOVERABLE_ERROR:int = 7;
        public static const LOST_CONNECTION_RECOVERABLE:int = 8;
        public static const NO_FIXED_SIZE:int = 9;
        public static const NOT_IMPLEMENTED:int = 10;
        public static const OUT_OF_MEMORY:int = 11;
        public static const PARSE_ERROR:int = 12;
        public static const SIZE_UNKNOWN:int = 13;
        public static const UNDERFLOW:int = 14;
        public static const UNSUPPORTED_CONFIGURATION:int = 15;
        public static const UNSUPPORTED_OPERATION:int = 16;
        public static const WAITING_FOR_INIT:int = 17;
        public static const INVALID_PARAMETER:int = 18;
        public static const INVALID_OPERATION:int = 19;
        public static const ONLY_ALLOWED_IN_PAUSED_STATE:int = 20;
        public static const INVALID_WITH_AUDIO_ONLY_FILE:int = 21;
        public static const PREVIOUS_STEP_SEEK_IN_PROGRESS:int = 22;
        public static const RESOURCE_NOT_SPECIFIED:int = 23;
        public static const RANGE_ERROR:int = 24;
        public static const INVALID_SEEK_TIME:int = 25;
        public static const FILE_STRUCTURE_INVALID:int = 26;
        public static const COMPONENT_CREATION_FAILURE:int = 27;
        public static const DRM_INIT_ERROR:int = 28;
        public static const CONTAINER_NOT_SUPPORTED:int = 29;
        public static const SEEK_FAILED:int = 30;
        public static const CODEC_NOT_SUPPORTED:int = 31;
        public static const NETWORK_UNAVAILABLE:int = 32;
        public static const NETWORK_ERROR:int = 33;
        public static const OVERFLOW:int = 34;
        public static const VIDEO_PROFILE_NOT_SUPPORTED:int = 35;
        public static const PERIOD_NOT_LOADED:int = 36;
        public static const INVALID_REPLACE_DURATION:int = 37;
        public static const CALLED_FROM_WRONG_THREAD:int = 38;
        public static const FRAGMENT_READ_ERROR:int = 39;
        public static const OPERATION_ABORTED:int = 40;
        public static const UNSUPPORTED_HLS_VERSION:int = 41;
        public static const CANNOT_FAIL_OVER:int = 42;
        public static const HTTP_TIME_OUT:int = 43;
        public static const NETWORK_DOWN:int = 44;
        public static const NO_USEABLE_BITRATE_PROFILE:int = 45;
        public static const BAD_MANIFEST_SIGNATURE:int = 46;
        public static const CANNOT_LOAD_PLAY_LIST:int = 47;
        public static const REPLACEMENT_FAILED:int = 48;
        public static const SWITCH_TO_ASYMMETRIC_PROFILE:int = 49;
        public static const LIVE_WINDOW_MOVED_BACKWARD:int = 50;
        public static const CURRENT_PERIOD_EXPIRED:int = 51;
        public static const CONTENT_LENGTH_MISMATCH:int = 52;
        public static const PERIOD_HOLD:int = 53;
        public static const LIVE_HOLD:int = 54;
        public static const BAD_MEDIA_INTERLEAVING:int = 55;
        public static const DRM_NOT_AVAILABLE:int = 56;
        public static const PLAYBACK_NOT_ENABLED:int = 57;
        public static const BAD_MEDIASAMPLE_FOUND:int = 58;
        public static const RANGE_SPANS_READHEAD:int = 59;
        public static const POSTROLL_WITH_LIVE_NOT_ALLOWED:int = 60;
        public static const INTERNAL_ERROR:int = 61;
        public static const SPS_PPS_FOUND_OUTSIDE_AVCC:int = 62;
        public static const PARTIAL_REPLACEMENT:int = 63;
        public static const RENDITION_M3U8_ERROR:int = 64;
        public static const NULL_OPERATION:int = 65;
        public static const SEGMENT_SKIPPED_ON_FAILURE:int = 66;
        public static const INCOMPATIBLE_RENDER_MODE:int = 67;
        public static const PROTOCOL_NOT_SUPPORTED:int = 68;
        public static const INCOMPATIBLE_VERSION:int = 69;
        public static const MANIFEST_FILE_UNEXPECTEDLY_CHANGED:int = 70;
        public static const CANNOT_SPLIT_TIMELINE:int = 71;
        public static const CANNOT_ERASE_TIMELINE:int = 72;
        public static const DID_NOT_GET_NEXT_FRAGMENT:int = 73;
        public static const NO_TIMELINE:int = 74;
        public static const LISTENER_NOT_FOUND:int = 75;
        public static const AUDIO_START_ERROR:int = 76;
        public static const NO_AUDIO_SINK:int = 77;
        public static const FILE_OPEN_ERROR:int = 78;
        public static const FILE_WRITE_ERROR:int = 79;
        public static const FILE_READ_ERROR:int = 80;
        public static const ID3_PARSE_ERROR:int = 81;
        public static const SECURITY_ERROR:int = 82;
        public static const TIMELINE_TOO_SHORT:int = 83;
        public static const AUDIO_ONLY_STREAM_START:int = 84;
        public static const AUDIO_ONLY_STREAM_END:int = 85;
        public static const CANNOT_HANDLE_MAIN_MANIFEST_UPDATE:int = 86;
        public static const KEY_NOT_FOUND:int = 87;
        public static const INVALID_KEY:int = 88;
        public static const KEY_SERVER_NOT_FOUND:int = 89;
        public static const MAIN_MANIFEST_UPDATE_TO_BE_HANDLED:int = 90;
        public static const UNREPORTED_TIME_DISCONTINUITY_FOUND:int = 91;

        public static const CRYPTO_ALGORITHM_NOT_SUPPORTED:int = 300;
        public static const CRYPTO_ERROR_CORRUPTED_DATA:int = 301;
        public static const CRYPTO_ERROR_BUFFER_TOO_SMALL:int = 302;
        public static const CRYPTO_ERROR_BAD_CERTIFICATE:int = 303;
        public static const CRYPTO_ERROR_DIGEST_UPDATE:int = 304;
        public static const CRYPTO_ERROR_DIGEST_FINISH:int = 305;
        public static const CRYPTO_ERROR_BAD_PARAMETER:int = 306;
        public static const CRYPTO_ERROR_UNKNOWN:int = 307;

        private var _result:int;

        public function AVResult(result:int) {
            this._result = result;
        }

        public function get result():int {
            return this._result;
        }
    }
}
