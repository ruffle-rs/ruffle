package flash.system {
    import flash.utils.ByteArray;

    [Ruffle(InstanceAllocator)]
    public final class ApplicationDomain {
        public static native function get currentDomain():ApplicationDomain;

        public function ApplicationDomain(parentDomain:ApplicationDomain = null) {
            this.init(parentDomain)
        }

        private native function init(parentDomain:ApplicationDomain):void;

        public native function get domainMemory():ByteArray;
        public native function set domainMemory(value:ByteArray):void;
        public native function get parentDomain():ApplicationDomain;

        public native function getDefinition(name:String):Object;
        public native function hasDefinition(name:String):Boolean;

        public native function getQualifiedDefinitionNames():Vector.<String>;

        public static function get MIN_DOMAIN_MEMORY_LENGTH():uint {
            return 1024;
        }

    }
}
