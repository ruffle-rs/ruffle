package {
    [Ruffle(ConstructOnCall)]
    [Ruffle(InstanceAllocator)]
    public dynamic class Error {
        {
            prototype.name = "Error";
            prototype.message = "Error";

            prototype.toString = function():String {
                var self:Error = this;
                return self.message !== "" ? self.name + ": " + self.message : self.name;
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
