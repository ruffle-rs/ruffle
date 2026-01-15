package {
    [Ruffle(ConstructOnCall)]
    [Ruffle(CustomConstructor)]
    public dynamic class Object {
        public function Object() {
            // Unreachable due to custom constructor
        }

        private static var _isPrototypeInitialized:Boolean = false;

        // We reuse the undocumented `init` method for lazily initializing Object's
        // prototype (since we can only do so once the Function class is initialized)
        internal static function init():* {
            if (_isPrototypeInitialized) return;
            _isPrototypeInitialized = true;

            prototype.isPrototypeOf = function(obj:* = void 0):Boolean {
                return this.AS3::isPrototypeOf(obj);
            };

            prototype.hasOwnProperty = function(prop:* = void 0):Boolean {
                return this.AS3::hasOwnProperty(prop);
            };

            prototype.propertyIsEnumerable = function(prop:* = void 0):* {
                return this.AS3::propertyIsEnumerable(prop);
            };

            // Prototype-only functions
            prototype.setPropertyIsEnumerable = function(prop:String, isEnumerable:Boolean):void {
                Object._setPropertyIsEnumerable(this, prop, isEnumerable);
            };

            prototype.toString = function():String {
                return Object._toString(this);
            };
            prototype.toLocaleString = prototype.toString;

            prototype.valueOf = function():* {
                return this;
            };

            prototype.setPropertyIsEnumerable("isPrototypeOf", false);
            prototype.setPropertyIsEnumerable("hasOwnProperty", false);
            prototype.setPropertyIsEnumerable("propertyIsEnumerable", false);
            prototype.setPropertyIsEnumerable("setPropertyIsEnumerable", false);
            prototype.setPropertyIsEnumerable("toString", false);
            prototype.setPropertyIsEnumerable("toLocaleString", false);
            prototype.setPropertyIsEnumerable("valueOf", false);
        }

        // These are called from prototype methods
        private static native function _setPropertyIsEnumerable(self:Object, prop:String, isEnumerable:Boolean):void;
        private static native function _toString(self:Object):String;

        // Normal instance methods
        AS3 native function isPrototypeOf(obj:* = void 0):Boolean;
        AS3 native function hasOwnProperty(prop:* = void 0):Boolean;
        AS3 native function propertyIsEnumerable(prop:* = void 0):Boolean;

        public static const length:int = 1;
    }
}
