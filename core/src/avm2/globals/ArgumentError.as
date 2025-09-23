package {
    [Ruffle(ConstructOnCall)]
    public dynamic class ArgumentError extends Error {
        ArgumentError.prototype.name = "ArgumentError"

        public function ArgumentError(message:String = "", code:* = 0) {
            super(message, code)
            this.name = prototype.name
        }

        public static const length:int = 1;
    }
}
