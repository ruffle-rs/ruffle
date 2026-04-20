package flash.errors {
    public dynamic class EOFError extends IOError {
        prototype.name = "EOFError";

        public function EOFError(message:String = "", id:int = 0) {
            super(message, id);
        }
    }
}
