package {
    public dynamic class TypeError extends Error {
        TypeError.prototype.name = "TypeError"

        public function TypeError(message:String = "", code:* = 0) {
            super(message, code)
            this.name = prototype.name
        }
    }
}
