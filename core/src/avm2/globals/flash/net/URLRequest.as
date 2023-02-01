package flash.net {

	import __ruffle__.stub_setter;

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
			stub_setter("flash.net.URLRequest", "method");
			this._method = newMethod;
		}

		public function get data():Object {
			return this._data;
		}

		public function set data(newData:Object):void {
			stub_setter("flash.net.URLRequest", "data");
			this._data = newData;
		}

		public function set contentType(value:String):void {
			stub_setter("flash.net.URLRequest", "contentType");
			this._contentType = value;
		}

		public function get contentType():String {
			return this._contentType;
		}
	}
}
