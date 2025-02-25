package {
    [Ruffle(InstanceAllocator)]
    [Ruffle(CallHandler)]
    public dynamic class Error {
        {
            prototype.name = "Error";

            prototype.toString = function():String {
                var self:Error = this;
                return self.message !== "" ? self.name + ": " + self.message : self.name;
            };
        }

        [Ruffle(NativeAccessible)]
        public var name:String = "Error";

        [Ruffle(NativeAccessible)]
        public var message:String;

        private var _id:int;

        public function Error(message:String = "", id:int = 0) {
            this.message = message;
            this._id = id;
        }

        public function get errorID():int {
            return this._id;
        }

        public native function getStackTrace():String;

        public static const length:int = 1;
    }
}
