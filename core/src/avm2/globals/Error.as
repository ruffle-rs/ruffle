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

        public static function throwError(type:Class, index:uint, ...rest):* {
            var template:String = getErrorMessage(index);
            var message:String = template.replace(/%([0-9])/g, function(match:*, group:*, pos:*, string:*):String {
                var param:int = int(group) - 1;

                // For some reason, Flash supports only 6 parameters and
                // ignores the rest.
                if (param < rest.length && param >= 0 && param < 6) {
                    return rest[param];
                }

                return "";
            });
            throw new type(message, index);
        }
    }
}
