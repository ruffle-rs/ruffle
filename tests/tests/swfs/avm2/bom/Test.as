package  {
	
	import flash.display.MovieClip;
	import flash.utils.ByteArray;
	import flash.utils.Endian;
	import flash.net.URLLoader;
	import flash.net.URLLoaderDataFormat;
	import flash.net.URLRequest;
	import flash.events.Event;
	import flash.events.IOErrorEvent;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			var utf8 = new ByteArray();
			var utf8Bytes = [0xef, 0xbb, 0xbf, 0x46, 0x78];
			for each (var byte in utf8Bytes) {
				utf8.writeByte(byte);
			}
			trace("ByteArray UTF-8: " + utf8);
		
			var utf16le = new ByteArray();
			var utf16leBytes = [0xff, 0xfe, 0x0, 0x22, 0x78, 0x0];
			for each (var byte in utf16leBytes) {
				utf16le.writeByte(byte);
			}
			trace("ByteArray UTF-16 Little endian: " + utf16le);
		
			var utf16be = new ByteArray();
			var utf16beBytes = [0xfe, 0xff, 0x22, 0x0, 0x0, 0x78];
			for each (var byte in utf16beBytes) {
				utf16be.writeByte(byte);
			}
			trace("ByteArray UTF-16 Big endian: " + utf16be);

			var files = ["utf8", "utf16le", "utf16be", "utf8", "utf16le", "utf16be"];
			var current = files.shift();
			var urlLoader = new URLLoader();
			urlLoader.dataFormat = URLLoaderDataFormat.TEXT;
			urlLoader.addEventListener(IOErrorEvent.IO_ERROR, function(event:IOErrorEvent):void {
				trace("URLLoader IOError: " + event);
			});
			urlLoader.addEventListener(Event.COMPLETE, function(event:Event):void {
				trace("URLLoader dataFormat=" + urlLoader.dataFormat + " " + current + ": " + event.target.data);
				if (files.length > 0) {
					if (files.length == 3) {
						urlLoader.dataFormat = URLLoaderDataFormat.VARIABLES;
					}
					current = files.shift();
					urlLoader.load(new URLRequest(current));
				}
			});
			urlLoader.load(new URLRequest(current));
		}
	}
}
