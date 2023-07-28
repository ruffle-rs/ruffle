package {
    public dynamic class UninitializedError extends Error {
        UninitializedError.prototype.name = "UninitializedError";
        
        public function UninitializedError(message:String = "", code:* = 0) {
            super(message, code);
            this.name = prototype.name;
        }

        public static const length:int = 1;
    }
}
