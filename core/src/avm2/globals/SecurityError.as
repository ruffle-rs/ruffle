package {
    [Ruffle(ConstructOnCall)]
    public dynamic class SecurityError extends Error {
        prototype.name = "SecurityError";

        public function SecurityError(message:String = "", id:int = 0) {
            super(message, id);
            this.name = prototype.name;
        }

        public static const length:int = 1;
    }
}
