package flash.desktop {
    [API("661")]
    [Ruffle(Abstract)]
    public class NativeDragManager {

        [API("668")]
        public static function get isSupported():Boolean {
            return false;
        }
    }
}
