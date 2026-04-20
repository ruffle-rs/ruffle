package flash.display {
    import flash.utils.ByteArray;

    [Ruffle(InstanceAllocator)]
    [API("662")]
    public final dynamic class ShaderData {
        public function ShaderData(bytecode:ByteArray) {
            this._setByteCode(bytecode);
        }

        private native function _setByteCode(bytecode:ByteArray);
    }
}
