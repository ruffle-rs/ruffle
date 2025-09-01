package {
    [Ruffle(ConstructOnCall)]
    public dynamic class EvalError extends Error {
        prototype.name = "EvalError";

        public function EvalError(message:String = "", id:int = 0) {
            super(message, id);
            this.name = prototype.name;
        }

        public static const length:int = 1;
    }
}
