package flash.trace {
    public class Trace {
        public static const OFF:int = 0;
        public static const METHODS:int = 1;
        public static const METHODS_WITH_ARGS:int = 2;
        public static const METHODS_AND_LINES:int = 3;
        public static const METHODS_AND_LINES_WITH_ARGS:int = 4;

        public static const FILE = 1;
        public static const LISTENER = 2;

        private static var _fileLevel:int = 0;
        private static var _listenerLevel:int = 0;
        private static var _listener:Function = null;

        public static function getLevel(target:int = LISTENER):int {
            if (target <= FILE) {
                return _fileLevel;
            }
            return _listenerLevel;
        }

        public static function getListener():Function {
            return _listener;
        }

        // These next two functions are no-ops in the release version of Flash Player.
        // TODO: Implement them properly in the future, maybe behind a flag?

        public static function setLevel(level:int, target:int = LISTENER) {}

        public static function setListener(func:Function) {}
    }
}
