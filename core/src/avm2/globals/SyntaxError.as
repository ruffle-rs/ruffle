package {
    [Ruffle(ConstructOnCall)]
    public dynamic class SyntaxError extends Error {
        prototype.name = "SyntaxError";

        public function SyntaxError(message:String = "", id:int = 0) {
            super(message, id);
            this.name = prototype.name;
        }

        public static const length:int = 1;
    }
}
