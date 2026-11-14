package flash.media {
    import flash.utils.ByteArray;

    [API("688")]
    public class AVSegmentedSource extends AVSource {
        public static const AUDIO:String = "audio";
        public static const AUDIO_DESCRIPTION:String = "audiodescription";
        public static const AUDIO_LANGUAGE:String = "audiolanguage";
        public static const AUDIO_PID:String = "audiopid";
        public static const DASH:String = "DASH";
        public static const DATA:String = "data";
        public static const DATA_DESCRIPTION:String = "datadescription";
        public static const HLS:String = "HLS";
        public static const VIDEO:String = "video";
        public static const VIDEO_DESCRIPTION:String = "videodescription";

        private var _cookieHeader:String = "";
        private var _useRedirectCache:Boolean = false;
        private var _networkingParams:AVNetworkingParams;

        public function AVSegmentedSource() {
            super();
        }

        public function get cookieHeader():String {
            return this._cookieHeader;
        }
        public function set cookieHeader(value:String):void {
            this._cookieHeader = value;
        }

        public function get useRedirectCache():Boolean {
            return this._useRedirectCache;
        }
        public function set useRedirectCache(value:Boolean):void {
            this._useRedirectCache = value;
        }

        public function load(url:String, containerType:String = "HLS", userData:int = 0):AVResult {
            return new AVResult(AVResult.ASYNC_OPERATION_IN_PROGRESS);
        }

        public function loadManifest(uri:String, userData:int = 0, containerType:String = "HLS"):AVResult {
            return new AVResult(AVResult.ASYNC_OPERATION_IN_PROGRESS);
        }

        public function loadWithBackgroundManifest(
            url:String,
            containerType:String,
            userData:int,
            backgroundManifest:String
        ):AVResult {
            return new AVResult(AVResult.ASYNC_OPERATION_IN_PROGRESS);
        }

        public function releaseManifest(userData:int):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        public function selectTrack(periodIndex:int, payloadType:String, trackIndex:int):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        public function selectTrackString(periodIndex:int, payloadType:String, trackString:String):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        public function addCustomTagNames(tagNames:Array):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        public function setCuePointTags(tagArray:Array):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        public function setSubscribedTags(tagArray:Array):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        public function setNetworkingParams(params:AVNetworkingParams):AVResult {
            this._networkingParams = params;
            return new AVResult(AVResult.SUCCESS);
        }

        public function getNetworkingParams():AVNetworkingParams {
            return this._networkingParams;
        }

        public function insertByLocalTime(
            periodIndex:int,
            insertionTime:Number,
            source:AVSegmentedSource,
            userData:int = 0,
            replaceDuration:Number = 0
        ):AVInsertionResult {
            return new AVInsertionResult(AVResult.SUCCESS, periodIndex, insertionTime);
        }

        public function insertByVirtualTime(
            insertionTime:Number,
            source:AVSegmentedSource,
            userData:int = 0,
            replaceDuration:Number = 0
        ):AVInsertionResult {
            return new AVInsertionResult(AVResult.SUCCESS, 0, insertionTime);
        }

        public function removeByLocalTime(
            periodIndex:int,
            timeStart:Number,
            timeEnd:Number,
            replaceWithMainMedia:Boolean
        ):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        public function removeByVirtualTime(timeStart:Number, timeEnd:Number):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        public function setHoldAt(time:Number):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        public function setPauseAtPeriodEnd(periodIndex:int, userData:int = 0):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        public function clearPauseAtPeriodEnd(periodIndex:int):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        override public function dispose():AVResult {
            return new AVResult(AVResult.SUCCESS);
        }
    }
}
