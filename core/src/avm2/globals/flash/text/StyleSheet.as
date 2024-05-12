package flash.text {
    import __ruffle__.stub_method;
    
    public dynamic class StyleSheet {
        // Shallow copies of the original style objects. Not used by Ruffle itself, just for getStyle()
        private var _styles: Object = {};

        public function StyleSheet() {}
        
        public function get styleNames():Array {
            var result = [];
            for (var key in _styles) {
                result.push(key);
            }
            return result;
        }
        
        public function clear():void {
            _styles = {};
        }
        
        public function getStyle(styleName:String):Object {
            return _createShallowCopy(_styles[styleName.toLowerCase()]);
        }
        
        public function parseCSS(CSSText:String):void {
            var parsed = innerParseCss(CSSText);
            if (!parsed) {
                // No thrown errors, silent failure. If the whole thing doesn't parse, just ignore it all.
                return;
            }

            for (var key in parsed) {
                setStyle(key, parsed[key]);
            }
        }
        
        public function setStyle(styleName:String, styleObject:Object):void {
            _styles[styleName.toLowerCase()] = _createShallowCopy(styleObject);
        }
        
        public function transform(formatObject:Object):TextFormat {
            stub_method("flash.text.StyleSheet", "transform");
            return null;
        }

        private function _createShallowCopy(original: *): Object {
            var copy = {};
            for (var key in original) {
                copy[key] = original[key];
            }
            return copy;
        }

        // Avoid doing potentially expensive string parsing in AS :D
        private native function innerParseCss(css: String): Object;
    }
}
