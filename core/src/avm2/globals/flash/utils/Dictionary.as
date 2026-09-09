package flash.utils {
    [Ruffle(InstanceAllocator)]
    [Ruffle(CustomConstructor)]
    public dynamic class Dictionary {
        prototype.toJSON = function(r:String):* {
            return "Dictionary";
        };
        prototype.setPropertyIsEnumerable("toJSON", false);

        public function Dictionary(weakKeys:Boolean = false) {
            // Dispatched to dictionary_constructor in Rust via
            // [Ruffle(CustomConstructor)]. This AS-defined method does nothing.
        }
    }
}
