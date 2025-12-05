package flash.utils {
    import __ruffle__.stub_constructor;

    [Ruffle(InstanceAllocator)]
    public dynamic class Dictionary {
        prototype.toJSON = function(r:String):* {
            return "Dictionary";
        };
        prototype.setPropertyIsEnumerable("toJSON", false);

        public function Dictionary(weakKeys:Boolean = false) {
            if (weakKeys) {
                stub_constructor("flash.utils.Dictionary", "with weak keys");
            }
        }
    }
}
