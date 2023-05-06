package {
    public dynamic class ReferenceError extends Error {
        ReferenceError.prototype.name = "ReferenceError";
        
        public function ReferenceError(message:String = "", code:* = 0) {
            super(message, code);
            this.name = prototype.name;
        }

        public static const length:int = 1;
    }
}
