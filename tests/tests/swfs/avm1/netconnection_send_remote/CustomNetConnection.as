class CustomNetConnection extends NetConnection {
	public function call() {
		// To make sure `super` works for NativeObjects
		super.call.apply(super,arguments);
	}
}