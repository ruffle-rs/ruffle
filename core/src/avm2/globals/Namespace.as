package {
    [Ruffle(InstanceAllocator)]
    [Ruffle(CallHandler)]
    public final class Namespace {
        prototype.toString = function():String {
            var n:Namespace = this;
            return n.uri;
        }
        prototype.valueOf = function():String {
            var n:Namespace = this;
            return n.uri;
        }

        prototype.setPropertyIsEnumerable("toString", false);
        prototype.setPropertyIsEnumerable("valueOf", false);

        public function Namespace(prefix:* = void 0, uri:* = void 0) {
            this.init(arguments);
        }

        private native function init(args:Array):void;

        public native function get prefix():*;
        public native function get uri():String;

        AS3 function toString():String {
            return this.uri;
        }

        AS3 function valueOf():String {
            return this.uri;
        }

        public static const length:* = 2;
    }
}
