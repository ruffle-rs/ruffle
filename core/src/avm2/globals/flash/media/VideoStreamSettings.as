package flash.media {
    [API("674")]
    public class VideoStreamSettings {
        private var _bandwidth:int = 16384;
        private var _codec:String = VideoCodec.SORENSON;
        private var _fps:Number = 15;
        private var _height:int = 120;
        private var _keyFrameInterval:int = 15;
        private var _quality:int = 0;
        private var _width:int = 160;

        public function VideoStreamSettings() {
        }

        public function setKeyFrameInterval(keyFrameInterval:int):void {
            this._keyFrameInterval = Math.max(1, Math.min(300, keyFrameInterval));
        }

        public function setMode(width:int, height:int, fps:Number):void {
            this._width = width;
            this._height = height;
            this._fps = fps;
        }

        public function setQuality(bandwidth:int, quality:int):void {
            this._bandwidth = bandwidth;
            this._quality = quality;
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
            return this._quality;
        }

        public function get width():int {
            return this._width;
        }
    }
}
