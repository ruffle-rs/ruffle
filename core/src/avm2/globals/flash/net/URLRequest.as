package flash.net {
	public final class URLRequest {
		// NOTE - when implementing properties (e.g. `contentType`, `data`, etc.)
		// be sure to also check for them in `URLLoader`

		// FIXME - this should be a getter/setter for consistency with Flash
		public var url:String;
		private var _method:String = URLRequestMethod.GET;
		private var _data:Object;

		public function URLRequest(url:String = null) {
			this.url = url;
		}

		public function get method():String {
			return this._method;
		}

		public function set method(newMethod:String):void {
			// FIXME - perform validation here
			this._method = newMethod;
		}

		public function get data():Object {
			return this._data;
		}

		public function set data(newData:Object):void {
			if (newData !== null) {
				throw new Error("URLRequest.data setter is not yet implemented!");
			}
			this._data = newData;
		}
	}
}
