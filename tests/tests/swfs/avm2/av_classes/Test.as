package  {
    
import flash.display.Sprite;
import flash.media.*;
import flash.utils.Dictionary;
import flash.events.EventDispatcher;


public class Test extends Sprite {

    public function Test() {
        super();

        var o:*;

        trace("=== Testing AVABRParameters");
        o = new AVABRParameters("hello", 2.2, 3.3, 4.4);
        testProp(o, "policy");
        testProp(o, "startBitsPerSecond");
        testProp(o, "minBitsPerSecond");
        testProp(o, "maxBitsPerSecond");
        testConsts(AVABRParameters, "AGGRESSIVE", "CONSERVATIVE", "MODERATE");
        trace("");

        trace("=== Testing AVABRProfileInfo");
        o = new AVABRProfileInfo(1.1, 2.2, 3.3);
        testProp(o, "bitsPerSecond");
        testProp(o, "width");
        testProp(o, "height");
        trace("");

        trace("=== Testing AVCaptionStyle");
        o = new AVCaptionStyle();
        testProp(o, "backgroundColor");
        testProp(o, "backgroundOpacity");
        testProp(o, "bottomInset");
        testProp(o, "edgeColor");
        testProp(o, "fillColor");
        testProp(o, "fillOpacity");
        testProp(o, "font");
        testProp(o, "fontColor");
        testProp(o, "fontEdge");
        testProp(o, "fontOpacity");
        testProp(o, "size");
        testConsts(
            AVCaptionStyle,
            "DEFAULT", "NONE",
            "MONOSPACE_WITH_SERIFS", "MONOSPACED_WITHOUT_SERIFS", "PROPORTIONAL_WITH_SERIFS", "PROPORTIONAL_WITHOUT_SERIFS",
            "CASUAL", "CURSIVE", "DEPRESSED", "RAISED", "SMALL_CAPITALS", "UNIFORM",
            "SMALL", "MEDIUM", "LARGE",
            "BRIGHT_MAGENTA", "MAGENTA", "DARK_MAGENTA",
            "BRIGHT_RED", "RED", "DARK_RED",
            "BRIGHT_YELLOW", "YELLOW", "DARK_YELLOW",
            "BRIGHT_GREEN", "GREEN", "DARK_GREEN",
            "BRIGHT_CYAN", "CYAN", "DARK_CYAN",
            "BRIGHT_BLUE", "BLUE", "DARK_BLUE",
            "BRIGHT_WHITE", "WHITE", "GRAY", "BLACK",
            "LEFT_DROP_SHADOW", "RIGHT_DROP_SHADOW"
        );
        trace("");

        trace("=== Testing AVCuePoint");
        trace("== with null dictionary");
        o = new AVCuePoint(null, 2.2);
        testProp(o, "dictionary");
        trace("== with actual dictionary");
        o = new AVCuePoint(new Dictionary(), 2.2);
        testProp(o, "dictionary");
        testProp(o, "localTime");
        trace("");

        trace("=== Testing AVInsertionResult");
        o = new AVInsertionResult(1.1, 2.2, 3.3);
        trace("o is AVResult: " + (o is AVResult));
        testProp(o, "result");
        testProp(o, "periodIndex");
        testProp(o, "insertedBeforeReadHead");
        trace("");

        trace("=== Testing AVNetworkingParams");
        trace("== with default params");
        o = new AVNetworkingParams();
        testProp(o, "forceNativeNetworking");
        testProp(o, "readSetCookieHeader");
        testProp(o, "useCookieHeaderForAllRequests");
        testProp(o, "networkDownVerificationUrl");
        testProp(o, "appendRandomQueryParameter");
        trace("== with provided params");
        o = new AVNetworkingParams(true, false, true, "hello");
        testProp(o, "forceNativeNetworking");
        testProp(o, "readSetCookieHeader");
        testProp(o, "useCookieHeaderForAllRequests");
        testProp(o, "networkDownVerificationUrl");
        testProp(o, "appendRandomQueryParameter");
        trace("");

        trace("=== Testing AVPeriodInfo");
        o = new AVPeriodInfo(1.1, 2.2, 3.3, 4.4, 5.5, 6.6, 7.7, 8.8, 9.9, 10.1);
        testProp(o, "localStartTime");
        testProp(o, "virtualStartTime");
        testProp(o, "duration");
        testProp(o, "firstCuePointIndex");
        testProp(o, "lastCuePointIndex");
        testProp(o, "firstSubscribedTagIndex");
        testProp(o, "lastSubscribedTagIndex");
        testProp(o, "userData");
        testProp(o, "supportsTrickPlay");
        testProp(o, "targetDuration");
        trace("");

        trace("=== Testing AVPlayState");
        o = new AVPlayState(1.1);
        testProp(o, "state");
        testConsts(
            AVPlayState,
            "UNINITIALIZED",
            "READY",
            "BUFFERING",
            "PLAYING",
            "PAUSED",
            "EOF",
            "SUSPENDED",
            "TRICK_PLAY",
            "UNRECOVERABLE_ERROR"
        );
        trace("");

        trace("=== Testing AVResult");
        o = new AVResult(1.2);
        testProp(o, "result");
        testConsts(
            AVResult,
            "END_OF_PERIOD", "SUCCESS", "ASYNC_OPERATION_IN_PROGRESS",
            "EOF", "DECODER_FAILED", "DEVICE_OPEN_ERROR",
            "FILE_NOT_FOUND", "GENERIC_ERROR", "IRRECOVERABLE_ERROR",
            "LOST_CONNECTION_RECOVERABLE", "NO_FIXED_SIZE", "NOT_IMPLEMENTED",
            "OUT_OF_MEMORY", "PARSE_ERROR", "SIZE_UNKNOWN",
            "UNDERFLOW", "UNSUPPORTED_CONFIGURATION", "UNSUPPORTED_OPERATION",
            "WAITING_FOR_INIT", "INVALID_PARAMETER", "INVALID_OPERATION",
            "ONLY_ALLOWED_IN_PAUSED_STATE", "INVALID_WITH_AUDIO_ONLY_FILE", "PREVIOUS_STEP_SEEK_IN_PROGRESS",
            "RESOURCE_NOT_SPECIFIED", "RANGE_ERROR", "INVALID_SEEK_TIME",
            "FILE_STRUCTURE_INVALID", "COMPONENT_CREATION_FAILURE", "DRM_INIT_ERROR",
            "CONTAINER_NOT_SUPPORTED", "SEEK_FAILED", "CODEC_NOT_SUPPORTED",
            "NETWORK_UNAVAILABLE", "NETWORK_ERROR", "OVERFLOW",
            "VIDEO_PROFILE_NOT_SUPPORTED", "PERIOD_NOT_LOADED", "INVALID_REPLACE_DURATION",
            "CALLED_FROM_WRONG_THREAD", "FRAGMENT_READ_ERROR", "OPERATION_ABORTED",
            "UNSUPPORTED_HLS_VERSION", "CANNOT_FAIL_OVER", "HTTP_TIME_OUT",
            "NETWORK_DOWN", "NO_USEABLE_BITRATE_PROFILE", "BAD_MANIFEST_SIGNATURE",
            "CANNOT_LOAD_PLAY_LIST", "REPLACEMENT_FAILED", "SWITCH_TO_ASYMMETRIC_PROFILE",
            "LIVE_WINDOW_MOVED_BACKWARD", "CURRENT_PERIOD_EXPIRED", "CONTENT_LENGTH_MISMATCH",
            "PERIOD_HOLD", "LIVE_HOLD", "BAD_MEDIA_INTERLEAVING",
            "DRM_NOT_AVAILABLE", "PLAYBACK_NOT_ENABLED", "BAD_MEDIASAMPLE_FOUND",
            "RANGE_SPANS_READHEAD", "POSTROLL_WITH_LIVE_NOT_ALLOWED", "INTERNAL_ERROR",
            "SPS_PPS_FOUND_OUTSIDE_AVCC", "PARTIAL_REPLACEMENT", "RENDITION_M3U8_ERROR",
            "NULL_OPERATION", "SEGMENT_SKIPPED_ON_FAILURE", "INCOMPATIBLE_RENDER_MODE",
            "PROTOCOL_NOT_SUPPORTED", "INCOMPATIBLE_VERSION", "MANIFEST_FILE_UNEXPECTEDLY_CHANGED",
            "CANNOT_SPLIT_TIMELINE", "CANNOT_ERASE_TIMELINE", "DID_NOT_GET_NEXT_FRAGMENT",
            "NO_TIMELINE", "LISTENER_NOT_FOUND", "AUDIO_START_ERROR",
            "NO_AUDIO_SINK", "FILE_OPEN_ERROR", "FILE_WRITE_ERROR",
            "FILE_READ_ERROR", "ID3_PARSE_ERROR", "SECURITY_ERROR",
            "TIMELINE_TOO_SHORT", "AUDIO_ONLY_STREAM_START", "AUDIO_ONLY_STREAM_END",
            "CANNOT_HANDLE_MAIN_MANIFEST_UPDATE", "KEY_NOT_FOUND", "INVALID_KEY",
            "KEY_SERVER_NOT_FOUND", "MAIN_MANIFEST_UPDATE_TO_BE_HANDLED", "UNREPORTED_TIME_DISCONTINUITY_FOUND",
            "CRYPTO_ALGORITHM_NOT_SUPPORTED", "CRYPTO_ERROR_CORRUPTED_DATA", "CRYPTO_ERROR_BUFFER_TOO_SMALL",
            "CRYPTO_ERROR_BAD_CERTIFICATE", "CRYPTO_ERROR_DIGEST_UPDATE", "CRYPTO_ERROR_DIGEST_FINISH",
            "CRYPTO_ERROR_BAD_PARAMETER", "CRYPTO_ERROR_UNKNOWN"
        );
        trace("");

        trace("=== Testing AVSegmentedSource");
        trace("AVSegmentedSource is AVSource:", (new AVSegmentedSource()) is AVSource);
        testConsts(
            AVSegmentedSource,
            "AUDIO",
            "AUDIO_DESCRIPTION",
            "AUDIO_LANGUAGE",
            "AUDIO_PID",
            "DASH",
            "DATA",
            "DATA_DESCRIPTION",
            "HLS",
            "VIDEO",
            "VIDEO_DESCRIPTION"
        );
        trace("");

        trace("=== Testing AVSource");
        trace("AVSource is EventDispatcher:", (new AVSource()) is EventDispatcher);
        trace("");

        trace("=== Testing AVStream");
        trace("AVStream is EventDispatcher", (new AVStream(new AVSource())) is EventDispatcher);
        testConsts(
            AVStream,
            "HARDWARE", "SOFTWARE", "UNDEFINED"
        );
        trace("");

        trace("=== Testing AVTagData");
        o = new AVTagData("hello", 2.2);
        testProp(o, "data");
        testProp(o, "localTime");
        trace("");
        
        trace("=== Testing AVTimeline");
        o = new AVTimeline("hello", 2.2, 3.3, 4.4, 5.5, 6.6, 7.7, true);
        testProp(o, "type");
        testProp(o, "virtualStartTime");
        testProp(o, "virtualDuration");
        testProp(o, "firstPeriodIndex");
        testProp(o, "lastPeriodIndex");
        testProp(o, "firstSubscribedTagIndex");
        testProp(o, "lastSubscribedTagIndex");
        testProp(o, "complete");
        trace("");

        trace("=== Testing AVTrackInfo");
        o = new AVTrackInfo("hello", "world", 3.3, 4.4, 5.5, 6.6, 7.7, 8.8);
        testProp(o, "description");
        testProp(o, "language");
        testProp(o, "defaultTrack");
        testProp(o, "autoSelect");
        testProp(o, "forced");
        testProp(o, "activity");
        testProp(o, "dataTrackInfoServiceType");
        testProp(o, "pid");
        testConsts(
            AVTrackInfo,
            "DTI_608_CAPTIONS", "DTI_708_CAPTIONS", "DTI_WEBVTT_CAPTIONS"
        );
    }

    private function testProp(cls:*, prop:String) {
        trace(prop + ": " + cls[prop], typeof cls[prop]);

        try {
            cls[prop] = 123.45;
            trace("Successfully set " + prop + " to " + cls[prop] + ", type " + typeof cls[prop]);
        } catch(e) {
            trace(e);
        }
    }

    private function testConsts(cls:*, ...props) {
        trace("== Testing constants");
        for (var i in props) {
            trace(props[i] + ": " + cls[props[i]], typeof cls[props[i]]);
        }
    }
}
    
}
