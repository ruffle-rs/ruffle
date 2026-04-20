package flash.errors{
    public dynamic class ScriptTimeoutError extends Error {
        prototype.name = "ScriptTimeoutError";

        public function ScriptTimeoutError(message:String = "", id:int = 0) {
            super(message, id);
        }
    }
}
