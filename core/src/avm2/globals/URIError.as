package {
    [Ruffle(ConstructOnCall)]
    public dynamic class URIError extends Error {
        prototype.name = "URIError";

        public function URIError(message:String = "", id:int = 0) {
            super(message, id);
            this.name = prototype.name;
        }

        public static const length:int = 1;
    }
}
