package {
    public dynamic class ArgumentError extends Error {
        prototype.name = "ArgumentError"

        public function ArgumentError(message:String = "", code:* = 0) {
            super(message, code)
            this.name = prototype.name
        }
    }
}