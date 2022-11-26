package flash.errors {
	public dynamic class IllegalOperationError extends Error {
		IllegalOperationError.prototype.name = "IllegalOperationError"

		// Despite what the documentation claims, user code can pass in an 'id'
		// parameter (which defaults to 0)
		public function IllegalOperationError(message:String = "", id:int = 0) {
			super(message, id);
			// Note that we do *not* set 'this.name' here (unlike in other error classes)
			// to match the Flash behavior
		}
	}
}
