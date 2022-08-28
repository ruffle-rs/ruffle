package {
	[Ruffle(InstanceAllocator)]
	public dynamic class Error {
		public var name:String;
		public var message:String;
		private var _id:int;

		public function Error(message:String = "", id:int = 0) {
			this.name = "Error";
			this.message = message;
			this._id = id;
		}

		public function get errorID():int {
			return this._id;
		}

		public native function getStackTrace():String;
	}
}
