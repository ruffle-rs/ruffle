package {
    [Ruffle(InstanceAllocator)]
    [Ruffle(CallHandler)]
    public dynamic class RegExp {
        // NOTE: FP doesn't do args checking for the RegExp constructor because
        // they mark the class as `construct="override"` (the equivalent of
        // `[Ruffle(CustomConstructor)]` in Ruffle). However, because RegExp
        // isn't `final`, we can't mark it as `CustomConstructor`. Instead, to
        // allow calling RegExp with more than two arguments, we mark the
        // constructor function as variadic using `...rest`.
        public function RegExp(re:* = undefined, flags:* = undefined, ...rest) {
            this.init(re, flags)
        }

        private native function init(re:*, flags:*):void;

        public native function get dotall():Boolean;
        public native function get extended():Boolean;
        public native function get global():Boolean;
        public native function get ignoreCase():Boolean;
        public native function get multiline():Boolean;
        public native function get lastIndex():int;
        public native function set lastIndex(value:int):*;
        public native function get source():String;

        AS3 native function exec(str:String = ""):*;
        AS3 native function test(str:String = ""):Boolean;

        prototype.exec = function(str:* = ""):* {
            var self:RegExp = this;
            return self.AS3::exec(str);
        }

        prototype.test = function(str:* = ""):Boolean {
            var self:RegExp = this;
            return self.AS3::test(str);
        }

        prototype.toString = function():String {
            // Note: This function is not generic and will throw for non-regexps.
            var regexp: RegExp = this;

            // ECMA-262 Edition 5.1 - RegExp.prototype.toString():
            //  Return the String value formed by concatenating the Strings "/",
            //  the String value of the source property of this RegExp object, and "/";
            //  plus "g" if the global property is true,
            //  "i" if the ignoreCase property is true,
            //  and "m" if the multiline property is true.
            var string = "/" + regexp.source + "/";
            if (regexp.global) {
                string += "g";
            }
            if (regexp.ignoreCase) {
                string += "i";
            }
            if (regexp.multiline) {
                string += "m";
            }
            if (regexp.dotall) {
                string += "s";
            }
            if (regexp.extended) {
                string += "x";
            }
            return string;
        }

        prototype.setPropertyIsEnumerable("exec", false);
        prototype.setPropertyIsEnumerable("test", false);
        prototype.setPropertyIsEnumerable("toString", false);

        public static const length:int = 1;
    }
}
