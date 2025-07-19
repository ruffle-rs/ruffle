package {
    [Ruffle(CallHandler)]
    [Ruffle(InstanceAllocator)]
    public dynamic class Array {
        public static const CASEINSENSITIVE:uint = 1;

        public static const DESCENDING:uint = 2;

        public static const UNIQUESORT:uint = 4;

        public static const RETURNINDEXEDARRAY:uint = 8;

        public static const NUMERIC:uint = 16;

        // FIXME avmplus allows for calling some of these prototype functions on any
        // Array-like object (for example, `Array.prototype.sort.call(myVector)` works),
        // but currently we only support calling them on real Arrays
        {
            prototype.concat = function(... rest):Array {
                var a:Array = this;
                return a.AS3::concat.apply(a, rest);
            };

            prototype.every = function(callback:Function, receiver:* = null):Boolean {
                var a:Array = this;
                return a.AS3::every(callback, receiver);
            };

            prototype.filter = function(callback:Function, receiver:* = null):Array {
                var a:Array = this;
                return a.AS3::filter(callback, receiver);
            };

            prototype.forEach = function(callback:Function, receiver:* = null):void {
                var a:Array = this;
                return a.AS3::forEach(callback, receiver);
            };

            prototype.indexOf = function(searchVal:*, from:* = 0):int {
                var a:Array = this;
                return a.AS3::indexOf(searchVal, from);
            };

            prototype.join = function(separator:* = void 0):String {
                var a:Array = this;
                return a.AS3::join(separator);
            };

            prototype.lastIndexOf = function(searchVal:*, from:* = 2147483647):int {
                var a:Array = this;
                return a.AS3::lastIndexOf(searchVal, from);
            };

            prototype.map = function(callback:Function, receiver:* = null):Array {
                var a:Array = this;
                return a.AS3::map(callback, receiver);
            };

            prototype.pop = function():* {
                var a:Array = this;
                return a.AS3::pop();
            };

            prototype.push = function(... args):uint {
                var a:Array = this;
                return a.AS3::push.apply(a, args);
            };

            prototype.reverse = function():* {
                var a:Array = this;
                return a.AS3::reverse();
            };

            prototype.shift = function():* {
                var a:Array = this;
                return a.AS3::shift();
            };

            prototype.slice = function(start:* = 0, end:* = 4294967295):Array {
                var a:Array = this;
                return a.AS3::slice(start, end);
            };

            prototype.some = function(callback:Function, receiver:* = null):Boolean {
                var a:Array = this;
                return a.AS3::some(callback, receiver);
            };

            prototype.sort = function(... rest):* {
                var a:Array = this;
                return a.AS3::sort.apply(a, rest);
            };

            prototype.sortOn = function(fieldNames:*, options:* = 0, ... rest):* {
                var a:Array = this;
                return a.AS3::sortOn(fieldNames, options);
            };

            prototype.splice = function(... rest):* {
                var a:Array = this;
                return a.AS3::splice.apply(a, rest);
            };

            prototype.toLocaleString = function():String {
                var a:Array = this;
                var result:String = "";
                var arrayLength:uint = a.length;

                for(var i:uint = 0; i < arrayLength; i ++) {
                    if (a[i] === void 0 || a[i] === null) {
                        result += a[i];
                    } else {
                        result += a[i].toLocaleString();
                    }

                    if (i != arrayLength - 1) {
                        result += ",";
                    }
                }

                return result;
            };

            prototype.toString = function():String {
                var a:Array = this;
                return a.AS3::join(",");
            };

            prototype.unshift = function(... rest):uint {
                var a:Array = this;
                return a.AS3::unshift.apply(a, rest);
            };

            prototype.setPropertyIsEnumerable("concat", false);
            prototype.setPropertyIsEnumerable("every", false);
            prototype.setPropertyIsEnumerable("filter", false);
            prototype.setPropertyIsEnumerable("forEach", false);
            prototype.setPropertyIsEnumerable("indexOf", false);
            prototype.setPropertyIsEnumerable("join", false);
            prototype.setPropertyIsEnumerable("lastIndexOf", false);
            prototype.setPropertyIsEnumerable("map", false);
            prototype.setPropertyIsEnumerable("pop", false);
            prototype.setPropertyIsEnumerable("push", false);
            prototype.setPropertyIsEnumerable("reverse", false);
            prototype.setPropertyIsEnumerable("shift", false);
            prototype.setPropertyIsEnumerable("slice", false);
            prototype.setPropertyIsEnumerable("some", false);
            prototype.setPropertyIsEnumerable("sort", false);
            prototype.setPropertyIsEnumerable("sortOn", false);
            prototype.setPropertyIsEnumerable("splice", false);
            prototype.setPropertyIsEnumerable("toLocaleString", false);
            prototype.setPropertyIsEnumerable("toString", false);
            prototype.setPropertyIsEnumerable("unshift", false);
        }

        // Constructor (defined in Rust)
        public native function Array(... rest);

        // Instance methods
        AS3 native function concat(... rest):Array;

        AS3 native function every(callback:Function, receiver:* = null):Boolean;

        AS3 native function filter(callback:Function, receiver:* = null):Array;

        AS3 native function forEach(callback:Function, receiver:* = null):void;

        AS3 native function indexOf(searchVal:*, from:* = 0):int;

        [API("708")]
        AS3 native function insertAt(index:int, element:*):void;

        AS3 native function join(separator:* = void 0):String;

        AS3 native function lastIndexOf(searchVal:*, from:* = 2147483647):int;

        public native function get length():uint;

        public native function set length(length:uint):*;

        AS3 native function map(callback:Function, receiver:* = null):Array;

        AS3 native function pop():*;

        AS3 native function push(... rest):uint;

        [API("708")]
        AS3 native function removeAt(index:int):*;

        AS3 native function reverse():Array;

        AS3 native function shift():*;

        AS3 native function slice(start:* = 0, end:* = 4294967295):Array;

        AS3 function some(callback:*, receiver:Object = null):Boolean {
            return _some(callback, receiver);
        }

        private native function _some(callback:Function, receiver:Object):Boolean;

        AS3 native function sort(... rest):*;

        AS3 native function sortOn(fieldNames:*, options:* = 0, ... rest):*;

        AS3 native function splice(... rest):*;

        AS3 native function unshift(... rest):uint;

        public static const length:int = 1;
    }
}
