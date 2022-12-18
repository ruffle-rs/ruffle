package flash.net {

	import __ruffle__.log_warn;

	public final class URLRequest {
		// NOTE - when implementing properties (e.g. `contentType`, `data`, etc.)
		// be sure to also check for them in `URLLoader`

		// FIXME - this should be a getter/setter for consistency with Flash
		public var url:String;
		private var _contentType: String = "application/x-www-form-urlencoded"; // ignored

		public var digest:String;
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
				log_warn("URLRequest.data setter is not yet implemented");
			}
			this._data = newData;
		}

		public function set contentType(value:String):void {
			if (value !== this._contentType) {
				log_warn("URLRequest.contentType setter is not yet implemented");
			}
			this._contentType = value;
		}

		public function get contentType():String {
			return this._contentType;
		}
	}
}
