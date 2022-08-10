package flash.errors {
	public dynamic class IllegalOperationError {
		// Despite what the documentation claims, user code can pass in an 'id'
		// parameter (which defaults to 0)
		public function IllegalOperationError(message:String = "", id:int = 0) {
			super(message, id);
		}
	}
}
