package flash.net {
	public final class URLRequest {
		// NOTE - when implementing properties (e.g. `contentType`, `data`, etc.)
		// be sure to also check for them in `URLLoader`

		// FIXME - this should be a getter/setter for consistency with Flash
		public var url:String;
		public function URLRequest(url:String = null) {
			this.url = url;
		}
	}
}
