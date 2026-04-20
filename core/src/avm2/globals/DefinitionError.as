package {
    [Ruffle(ConstructOnCall)]
    public dynamic class DefinitionError extends Error {
        prototype.name = "DefinitionError";

        public function DefinitionError(message:String = "", id:int = 0) {
            super(message, id);
            this.name = prototype.name;
        }

        public static const length:int = 1;
    }
}
