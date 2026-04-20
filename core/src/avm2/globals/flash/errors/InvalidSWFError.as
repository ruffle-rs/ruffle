package flash.errors {
    public dynamic class InvalidSWFError extends Error {
        prototype.name = "InvalidSWFError";

        public function InvalidSWFError(message:String = "", id:int = 0) {
            super(message, id);
        }
    }
}
