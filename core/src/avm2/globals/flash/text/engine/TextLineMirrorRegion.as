package flash.text.engine {
    import flash.events.EventDispatcher;
    import flash.geom.Rectangle;

    [API("662")]
    [Ruffle(Abstract)]
    public final class TextLineMirrorRegion {
        private var _bounds:Rectangle = new Rectangle();
        private var _element:ContentElement = null;
        private var _mirror:EventDispatcher = null;
        private var _nextRegion:TextLineMirrorRegion = null;
        private var _previousRegion:TextLineMirrorRegion = null;
        private var _textLine:TextLine = null;

        public function get bounds():Rectangle {
            return this._bounds;
        }

        public function get element():ContentElement {
            return this._element;
        }

        public function get mirror():EventDispatcher {
            return this._mirror;
        }

        public function get nextRegion():TextLineMirrorRegion {
            return this._nextRegion;
        }

        public function get previousRegion():TextLineMirrorRegion {
            return this._previousRegion;
        }

        public function get textLine():TextLine {
            return this._textLine;
        }
    }
}
