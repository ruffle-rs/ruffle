package flash.display {
    import __ruffle__.stub_getter;

    import flash.events.EventDispatcher;
    import flash.geom.Rectangle;

    [API("661")]
    // FIXME this class should be `[Ruffle(Abstract)]`
    public final class Screen extends EventDispatcher {
        private static var _mainScreen:Screen = null;

        public static function get mainScreen():Screen {
            stub_getter("flash.display.Screen", "mainScreen");

            if (_mainScreen === null) {
                _mainScreen = new Screen();
            }

            return _mainScreen;
        }

        public function get visibleBounds():Rectangle {
            stub_getter("flash.display.Screen", "visibleBounds");

            return new Rectangle(0, 0, 0, 0);
        }
    }
}
