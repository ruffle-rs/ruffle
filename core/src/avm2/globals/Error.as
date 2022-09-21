package {
	[Ruffle(InstanceAllocator)]
	public dynamic class Error {
		Error.prototype.name = "Error"

		public var name:String = "Error";
		public var message:String;
		private var _id:int;

		public function Error(message:String = "", id:int = 0) {
			this.message = message;
			this._id = id;
		}

		public function get errorID():int {
			return this._id;
		}

		public native function getStackTrace():String;
	}
}
