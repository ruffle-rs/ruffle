package flash.text {
    import flash.events.EventDispatcher;

    public dynamic class StyleSheet extends EventDispatcher {
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
            transform(_createShallowCopy(styleObject)); // TODO: Store this in a way that Rust can access it, when we implement `TextField.stylesheet`
        }
        
        public function transform(formatObject:Object):TextFormat {
            if (!formatObject) {
                return null;
            }
            var result = new TextFormat();

            if (formatObject.color) {
                result.color = innerParseColor(formatObject.color);
            }

            if (formatObject.display) {
                result.display = formatObject.display;
            }

            if (formatObject.fontFamily) {
                result.font = innerParseFontFamily(formatObject.fontFamily);
            }

            if (formatObject.fontSize) {
                var size = parseInt(formatObject.fontSize);
                if (size > 0) {
                    result.size = size;
                }
            }

            if (formatObject.fontStyle == "italic") {
                result.italic = true;
            } else if (formatObject.fontStyle == "normal") {
                result.italic = false;
            }

            if (formatObject.fontWeight == "bold") {
                result.bold = true;
            } else if (formatObject.fontWeight == "normal") {
                result.bold = false;
            }

            if (formatObject.kerning == "true") {
                result.kerning = true;
            } else if (formatObject.kerning == "false") {
                result.kerning = false;
            } else {
                // Seems to always set, not just if defined
                result.kerning = parseInt(formatObject.kerning);
            }

            if (formatObject.leading) {
                result.leading = parseInt(formatObject.leading);
            }

            if (formatObject.letterSpacing) {
                result.letterSpacing = parseFloat(formatObject.letterSpacing);
            }

            if (formatObject.marginLeft) {
                result.leftMargin = parseFloat(formatObject.marginLeft);
            }

            if (formatObject.marginRight) {
                result.rightMargin = parseFloat(formatObject.marginRight);
            }

            if (formatObject.textAlign) {
                result.align = formatObject.textAlign;
            }

            if (formatObject.textDecoration == "underline") {
                result.underline = true;
            } else if (formatObject.textDecoration == "none") {
                result.underline = false;
            }

            if (formatObject.textIndent) {
                result.indent = parseInt(formatObject.textIndent);
            }

            return result;
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
        private native function innerParseColor(color: String): Number;
        private native function innerParseFontFamily(fontFamily: String): String;
    }
}
