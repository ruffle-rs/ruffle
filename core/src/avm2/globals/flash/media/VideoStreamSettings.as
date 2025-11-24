package flash.media {
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;

    [API("674")]
    public class VideoStreamSettings {
        private var _bandwidth:int;
        private var _codec:String;
        private var _fps:Number;
        private var _height:int;
        private var _keyFrameInterval:int;
        private var _quality:int;
        private var _width:int;

        public function setKeyFrameInterval(keyFrameInterval:int):void {
            stub_method("flash.media.VideoStreamSettings", "setKeyFrameInterval");
        }

        public function setMode(width:int, height:int, fps:Number):void {
            stub_method("flash.media.VideoStreamSettings", "setMode");
        }

        public function setQuality(bandwidth:int, quality:int):void {
            stub_method("flash.media.VideoStreamSettings", "setQuality");
        }

        public function get bandwidth():int {
            return this._bandwidth;
        }

        public function get codec():String {
            return this._codec;
        }

        public function get fps():Number {
            return this._fps;
        }

        public function get height():int {
            return this._height;
        }

        public function get keyFrameInterval():int {
            return this._keyFrameInterval;
        }

        public function get quality():int {
            stub_getter("flash.media.VideoStreamSettings", "quality");
            return this._quality;
        }

        public function get width():int {
            return this._width;
        }
    }
}
