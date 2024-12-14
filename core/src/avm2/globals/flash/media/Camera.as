package flash.media {
    import flash.events.EventDispatcher;
    import flash.utils.ByteArray;
    import flash.geom.Rectangle;
    import flash.display.BitmapData;

    public final class Camera extends EventDispatcher {
        [API("682")]
        public function copyToByteArray(rect:Rectangle, destination:ByteArray) {
            __ruffle__.stub_method("flash.media.Camera", "copyToByteArray");
        }

        [API("682")]
        public function copyToVector(rect:Rectangle, destination:Vector.<uint>) {
            __ruffle__.stub_method("flash.media.Camera", "copyToVector");
        }

        [API("682")]
        public function drawToBitmapData(destination:BitmapData) {
            __ruffle__.stub_method("flash.media.Camera", "drawToBitmapData");
        }

        public static function getCamera(name: String = null):Camera {
            __ruffle__.stub_method("flash.media.Camera", "getCamera");
            return null;
        }

        public function setKeyFrameInterval(keyFrameInterval:int) {
            __ruffle__.stub_method("flash.media.Camera", "setKeyFrameInterval");
        }

        public function setLoopback(compress:Boolean = false) {
            __ruffle__.stub_method("flash.media.Camera", "setLoopback");
        }

        public function setMode(width:int, height:int, fps:Number, favorArea:Boolean = true) {
            __ruffle__.stub_method("flash.media.Camera", "setMode");
        }

        public function setMotionLevel(motionLevel:int, timeout:int = 2000) {
            __ruffle__.stub_method("flash.media.Camera", "setMotionLevel");
        }

        public function setQuality(bandwidth:int, quality:int) {
            __ruffle__.stub_method("flash.media.Camera", "setQuality");
        }

        public function get activityLevel(): Number {
            __ruffle__.stub_getter("flash.media.Camera", "activityLevel");
            return 0;
        }

        public function get bandwidth(): int {
            __ruffle__.stub_getter("flash.media.Camera", "bandwidth");
            return 0;
        }

        public function get currentFPS(): Number {
            __ruffle__.stub_getter("flash.media.Camera", "currentFPS");
            return 0;
        }

        public function get fps(): Number {
            __ruffle__.stub_getter("flash.media.Camera", "fps");
            return 0;
        }

        public function get height(): int {
            __ruffle__.stub_getter("flash.media.Camera", "height");
            return 0;
        }

        public function get index(): int {
            __ruffle__.stub_getter("flash.media.Camera", "index");
            return 0;
        }

        public static function get isSupported(): Boolean {
            __ruffle__.stub_getter("flash.media.Camera", "isSupported");
            return false;
        }

        public function get keyFrameInterval(): int {
            __ruffle__.stub_getter("flash.media.Camera", "keyFrameInterval");
            return 0;
        }

        public function get loopback(): Boolean {
            __ruffle__.stub_getter("flash.media.Camera", "loopback");
            return false;
        }

        public function get motionLevel(): int {
            __ruffle__.stub_getter("flash.media.Camera", "motionLevel");
            return 0;
        }

        public function get motionTimeout(): int {
            __ruffle__.stub_getter("flash.media.Camera", "motionTimeout");
            return 0;
        }

        public function get muted(): Boolean {
            __ruffle__.stub_getter("flash.media.Camera", "muted");
            return true;
        }

        public function get name(): String {
            __ruffle__.stub_getter("flash.media.Camera", "name");
            return "";
        }

        public static function get names(): Array {
            __ruffle__.stub_getter("flash.media.Camera", "names");
            return [];
        }
        
        public function get quality(): int {
            __ruffle__.stub_getter("flash.media.Camera", "quality");
            return 0;
        }
        
        public function get width(): int {
            __ruffle__.stub_getter("flash.media.Camera", "width");
            return 0;
        }
    }

}