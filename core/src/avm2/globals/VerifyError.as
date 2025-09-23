package {
    [Ruffle(ConstructOnCall)]
    public dynamic class VerifyError extends Error {
        prototype.name = "VerifyError";

        public function VerifyError(message:String = "", id:int = 0) {
            super(message, id);
            this.name = prototype.name;
        }

        public static const length:int = 1;
    }
}
