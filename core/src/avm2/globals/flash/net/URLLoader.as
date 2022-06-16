package flash.net {
	import flash.events.EventDispatcher;
	import flash.net.URLRequest;
	public class URLLoader extends EventDispatcher {
		public var data: *;
		public var dataFormat: String = "text";

		public function URLLoader(request:URLRequest = null) {
			if (request != null) {
				this.load(request);
			}
		}

		// FIXME - this should be a normal property for consistency with Flash
		public function get bytesTotal():uint {
			if (this.data) {
				return this.data.length;
			}
			return 0;
		}

		// FIXME - this should be a normal property for consistency with Flash
		public function get bytesLoaded():uint {
			// TODO - update this as the download progresses
			return this.bytesTotal
		}
		public native function load(request:URLRequest):void;
	}
}
