package {
    public dynamic class RangeError extends Error {
        prototype.name = "RangeError"

        public function RangeError(message:String = "", code:* = 0) {
            super(message, code);
            this.name = prototype.name;
        }
    }
}