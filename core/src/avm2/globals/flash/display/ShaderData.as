package flash.display {
    import flash.utils.ByteArray;
    
    [Ruffle(InstanceAllocator)]
    public final dynamic class ShaderData {
        public function ShaderData(bytecode:ByteArray) {
            this.init(bytecode);
        }

        private native function init(bytecode:ByteArray);
    }
}

