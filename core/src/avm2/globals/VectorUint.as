package __AS3__.vec {
    [Ruffle(CallHandler)]
    [Ruffle(InstanceAllocator)]
    internal final dynamic class Vector$uint {
        {
            prototype.concat = function(... rest):* {
                var v:Vector$uint = this;
                return v.AS3::concat.apply(v, rest);
            };

            prototype.every = function(callback:*, receiver:* = void 0):Boolean {
                var v:Vector$uint = this;
                return v.AS3::every(callback, receiver);
            };

            prototype.filter = function(callback:*, receiver:* = void 0):* {
                var v:Vector$uint = this;
                return v.AS3::filter(callback, receiver);
            };

            prototype.forEach = function(callback:*, receiver:* = void 0):* {
                var v:Vector$uint = this;
                v.AS3::forEach(callback, receiver);
            };

            prototype.indexOf = function(searchVal:*, from:* = void 0):* {
                var v:Vector$uint = this;
                return v.AS3::indexOf(searchVal, from);
            };

            prototype.join = function(separator:* = void 0):* {
                if (separator == void 0) {
                    separator = ",";
                }

                var v:Vector$uint = this;
                return v.AS3::join(separator);
            };

            prototype.lastIndexOf = function(searchVal:*, from:* = void 0):* {
                if (from == void 0) {
                    from = Infinity;
                }

                var v:Vector$uint = this;
                return v.AS3::lastIndexOf(searchVal, from);
            };

            prototype.map = function(callback:*, receiver:* = void 0):* {
                var v:Vector$uint = this;
                return v.AS3::map(callback, receiver);
            };

            prototype.pop = function():* {
                var v:Vector$uint = this;
                return v.AS3::pop();
            };

            prototype.push = function(... rest):* {
                var v:Vector$uint = this;
                return v.AS3::push.apply(v, rest);
            };

            prototype.reverse = function():* {
                var v:Vector$uint = this;
                return v.AS3::reverse();
            };

            prototype.shift = function():* {
                var v:Vector$uint = this;
                return v.AS3::shift();
            };

            prototype.slice = function(start:* = void 0, end:* = void 0):* {
                if (start == void 0) {
                    start = 0;
                }
                if (end == void 0) {
                    end = 2147483647;
                }

                var v:Vector$uint = this;
                return v.AS3::slice(start, end);
            };

            prototype.some = function(callback:*, receiver:* = void 0):Boolean {
                var v:Vector$uint = this;
                return v.AS3::some(checker, receiver);
            };

            prototype.sort = function(func:*):* {
                var v:Vector$uint = this;
                return v.AS3::sort(func);
            };

            prototype.splice = function(start:*, deleteCount:*, ... items):* {
                var argsList:Array = [start, deleteCount];
                argsList = argsList.AS3::concat(items);

                var v:Vector$uint = this;
                return v.AS3::splice.apply(v, argsList);
            };

            prototype.toLocaleString = function():* {
                var v:Vector$uint = this;
                return v.AS3::join(",");
            };

            prototype.toString = function():* {
                var v:Vector$uint = this;
                return v.AS3::join(",");
            };

            prototype.unshift = function(... rest):* {
                var v:Vector$uint = this;
                return v.AS3::unshift.apply(v, rest);
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
            prototype.setPropertyIsEnumerable("splice", false);
            prototype.setPropertyIsEnumerable("toLocaleString", false);
            prototype.setPropertyIsEnumerable("toString", false);
            prototype.setPropertyIsEnumerable("unshift", false);
        }

        // Constructor (defined in Rust)
        public native function Vector$uint(length:uint = 0, isFixed:Boolean = false);

        // Instance methods
        public native function get fixed():Boolean;

        public native function set fixed(isFixed:Boolean):*;

        [Ruffle(FastCall)]
        public native function get length():uint;

        public native function set length(length:uint):*;

        AS3 native function concat(... rest):Vector$uint;

        AS3 native function every(callback:Function, receiver:Object = null):Boolean;

        AS3 native function filter(callback:Function, receiver:Object = null):Vector$uint;

        AS3 native function forEach(callback:Function, receiver:Object = null):void;

        AS3 native function indexOf(searchVal:uint, from:Number = 0):Number;

        [API("708")]
        AS3 native function insertAt(index:int, element:uint):void;

        AS3 native function join(separator:String = ","):String;

        AS3 native function lastIndexOf(searchVal:uint, from:Number = 2147483647):Number;

        AS3 native function map(callback:Function, receiver:Object = null):*;

        AS3 native function pop():uint;

        AS3 native function push(... rest):uint;

        [API("708")]
        // For some reason the parameter is a uint for Vector$uint specifically
        AS3 native function removeAt(index:uint):uint;

        AS3 native function reverse():Vector$uint;

        AS3 native function shift():uint;

        AS3 native function slice(start:Number = 0, end:Number = 2147483647):Vector$uint;

        AS3 function some(callback:*, receiver:Object = null):Boolean {
            return _some(callback, receiver);
        }

        private native function _some(callback:Function, receiver:Object):Boolean;

        AS3 native function sort(func:*):Vector$uint;

        AS3 native function splice(start:Number, deleteLen:Number, ... rest):Vector$uint;

        AS3 function toLocaleString():String {
            return this.AS3::join(",");
        }

        AS3 function toString():String {
            return this.AS3::join(",");
        }

        AS3 native function unshift(... rest):uint;
    }
}
