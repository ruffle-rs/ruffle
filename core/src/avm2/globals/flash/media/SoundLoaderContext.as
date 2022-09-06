package flash.media {
    public class SoundLoaderContext {
        public var bufferTime:Number = 1000;
        public var checkPolicyFile:Boolean = false;

        public function SoundLoaderContext(bufferTime:Number = 1000, checkPolicyFile:Boolean = false) {
            this.checkPolicyFile = checkPolicyFile;
            this.bufferTime = bufferTime;
        }
    }
}
