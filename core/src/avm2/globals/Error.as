package {
    [Ruffle(ConstructOnCall)]
    [Ruffle(InstanceAllocator)]
    public dynamic class Error {
        private static native function initCustomPrototype();

        {
            initCustomPrototype();

            prototype.name = "Error";
            prototype.message = "Error";

            prototype.toString = function():String {
                return this.message !== "" ? this.name + ": " + this.message : this.name;
            };
        }

        public static native function getErrorMessage(id:int):String;

        public var name:*;

        public var message:*;

        private var _id:int;

        public function Error(message:* = "", id:* = 0) {
            this.message = message;
            this._id = id;
            this.name = prototype.name;
        }

        public function get errorID():int {
            return this._id;
        }

        public native function getStackTrace():String;

        public static const length:int = 1;
    }
}
