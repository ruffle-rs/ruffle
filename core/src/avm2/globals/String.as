package {
    [Ruffle(CustomConstructor)]
    [Ruffle(CallHandler)]
    public final class String {
         {
            prototype.charAt = function(index:Number = 0):String {
                var s:String = this;
                return s.AS3::charAt(index);
            };

            prototype.charCodeAt = function(index:Number = 0):Number {
                var s:String = this;
                return s.AS3::charCodeAt(index);
            };

            prototype.concat = function(... args):String {
                var s:String = this;
                return s.AS3::concat.apply(s, args);
            };

            prototype.indexOf = function(str:String = "undefined", index:Number = 0):int {
                var s:String = this;
                return s.AS3::indexOf(str, index);
            };

            prototype.lastIndexOf = function(str:String = "undefined", index:Number = 2147483647):int {
                var s:String = this;
                return s.AS3::lastIndexOf(str, index);
            };

            prototype.localeCompare = function(string:* = void 0):int {
                var s:String = this;
                return s.AS3::localeCompare(string);
            };

            prototype.match = function(pattern:* = void 0):Array {
                var s:String = this;
                return s.AS3::match(pattern);
            };

            prototype.replace = function(pattern:* = void 0, replace:* = void 0):String {
                var s:String = this;
                return s.AS3::replace(pattern, replace);
            };

            prototype.search = function(pattern:* = void 0):int {
                var s:String = this;
                return s.AS3::search(pattern);
            };

            prototype.slice = function(start:Number = 0, end:Number = 2147483647.0):String {
                var s:String = this;
                return s.AS3::slice(start, end);
            };

            prototype.split = function(delimeter:* = void 0, limit:* = 4294967295.0):Array {
                var s:String = this;
                return s.AS3::split(delimeter, limit);
            };

            prototype.substr = function(start:Number = 0, len:Number = 2147483647.0):String {
                var s:String = this;
                return s.AS3::substr(start,len);
            };

            prototype.substring = function(start:Number = 0, end:Number = 2147483647.0):String {
                var s:String = this;
                return s.AS3::substring(start, end);
            };

            prototype.toLocaleLowerCase = function():String {
                var s:String = this;
                return s.AS3::toLowerCase();
            };

            prototype.toLocaleUpperCase = function():String {
                var s:String = this;
                return s.AS3::toUpperCase();
            };

            prototype.toLowerCase = function():String {
                var s:String = this;
                return s.AS3::toLowerCase();
            };

            prototype.toUpperCase = function():String {
                var s:String = this;
                return s.AS3::toUpperCase();
            };

            prototype.toString = function():String {
                if(this === String.prototype) {
                    return "";
                }

                if(!(this is String)) {
                    throw new TypeError("Error #1004: Method String.prototype.toString was invoked on an incompatible object.", 1004);
                }

                return this;
            };

            prototype.valueOf = function():* {
                if(this === String.prototype) {
                    return "";
                }

                if(!(this is String)) {
                    throw new TypeError("Error #1004: Method String.prototype.valueOf was invoked on an incompatible object.", 1004);
                }

                return this;
            };

            prototype.setPropertyIsEnumerable("charAt", false);
            prototype.setPropertyIsEnumerable("charCodeAt", false);
            prototype.setPropertyIsEnumerable("concat", false);
            prototype.setPropertyIsEnumerable("indexOf", false);
            prototype.setPropertyIsEnumerable("lastIndexOf", false);
            prototype.setPropertyIsEnumerable("localeCompare", false);
            prototype.setPropertyIsEnumerable("match", false);
            prototype.setPropertyIsEnumerable("replace", false);
            prototype.setPropertyIsEnumerable("search", false);
            prototype.setPropertyIsEnumerable("slice", false);
            prototype.setPropertyIsEnumerable("split", false);
            prototype.setPropertyIsEnumerable("substr", false);
            prototype.setPropertyIsEnumerable("substring", false);
            prototype.setPropertyIsEnumerable("toLocaleLowerCase", false);
            prototype.setPropertyIsEnumerable("toLocaleUpperCase", false);
            prototype.setPropertyIsEnumerable("toLowerCase", false);
            prototype.setPropertyIsEnumerable("toUpperCase", false);
            prototype.setPropertyIsEnumerable("toString", false);
            prototype.setPropertyIsEnumerable("valueOf", false);
        }

        public function String(value:* = "") {
            // The String constructor is implemented natively:
            // this AS-defined method does nothing
        }

        AS3 static native function fromCharCode(... rest):String;

        public static native function fromCharCode(... rest):String;

        // Instance methods
        public native function get length():int;

        AS3 native function charAt(index:Number = 0):String;

        AS3 native function charCodeAt(index:Number = 0):Number;

        AS3 native function concat(... rest):String;

        AS3 native function indexOf(str:String = "undefined", index:Number = 0):int;

        AS3 native function lastIndexOf(str:String = "undefined", index:Number = 2147483647.0):int;

        AS3 native function localeCompare(string:* = void 0):int;

        // We can't just have a native `match` method because `match` is a Rust keyword.
        AS3 function match(pattern:* = void 0):Array {
            return this.matchInternal(pattern);
        }

        private native function matchInternal(pattern:*):Array;

        AS3 native function replace(pattern:* = void 0, replace:* = void 0):String;

        AS3 native function search(pattern:* = void 0):int;

        AS3 native function slice(start:Number = 0, end:Number = 2147483647.0):String;

        AS3 native function split(delimeter:* = void 0, limit:* = 4294967295):Array;

        AS3 native function substr(start:Number = 0, length:Number = 2147483647.0):String;

        AS3 native function substring(start:Number = 0, end:Number = 2147483647.0):String;

        AS3 function toLocaleLowerCase() : String {
            return this.toLowerCase();
        }

        AS3 function toLocaleUpperCase() : String {
            return this.toUpperCase();
        }

        AS3 native function toLowerCase() : String;

        AS3 native function toUpperCase() : String;

        AS3 function toString() : String {
            return this;
        }

        AS3 function valueOf() : String {
            return this;
        }

        public static const length:int = 1;
    }
}
