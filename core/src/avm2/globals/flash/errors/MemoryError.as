package flash.errors {
    public dynamic class MemoryError extends Error {
        prototype.name = "MemoryError";

        public function MemoryError(message:String = "", id:int = 0) {
            super(message, id);
        }
    }
}
