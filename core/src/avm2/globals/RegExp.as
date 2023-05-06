package {
    [Ruffle(InstanceAllocator)]
    [Ruffle(CallHandler)]
    public dynamic class RegExp {
        public function RegExp(re:* = undefined, flags:* = undefined) {
            this.init(re, flags)
        }

        private native function init(re:*, flags:*):void;

        public native function get dotall():Boolean;
        public native function get extended():Boolean;
        public native function get global():Boolean;
        public native function get ignoreCase():Boolean;
        public native function get multiline():Boolean;
        public native function get lastIndex():int;
        public native function set lastIndex(value:int):void;
        public native function get source():String;

        AS3 native function exec(str:String = ""):Object;
        AS3 native function test(str:String = ""):Boolean;

        prototype.exec = function(str:String = ""):Object {
            return this.AS3::exec(str);
        }

        prototype.test = function(str:String = ""):Boolean {
            return this.AS3::test(str);
        }

        prototype.toString = function():String {
            return this.valueOf();
        }
        
        prototype.setPropertyIsEnumerable("exec", false);
        prototype.setPropertyIsEnumerable("test", false);
        prototype.setPropertyIsEnumerable("toString", false);

        public static const length:int = 1;
    }
}
