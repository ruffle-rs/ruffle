package {
    public dynamic class ReferenceError extends Error {
        prototype.name = "ReferenceError";
        
        public function ReferenceError(message:String = "", code:* = 0) {
            super(message, code);
            this.name = prototype.name;
        }
    }
}