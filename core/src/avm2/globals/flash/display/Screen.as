package flash.display {
    import flash.events.EventDispatcher;
    import flash.geom.Rectangle;

    [API("661")]
    public final class Screen extends EventDispatcher {
        private static var _mainScreen:Screen = null;

        public static function get mainScreen():Screen {
            if (_mainScreen === null) {
                _mainScreen = new Screen();
            }

            return _mainScreen;
        }

        public static function get screens():Array {
            return [mainScreen];
        }

        public function get bounds():Rectangle {
            return new Rectangle(0, 0, 0, 0);
        }

        public function get colorDepth():int {
            return 32;
        }

        public function get visibleBounds():Rectangle {
            return new Rectangle(0, 0, 0, 0);
        }
    }
}
