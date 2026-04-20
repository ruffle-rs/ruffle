package flash.errors {
    public dynamic class StackOverflowError extends Error {
        prototype.name = "StackOverflowError";

        public function StackOverflowError(message:String = "", id:int = 0) {
            super(message, id);
        }
    }
}
