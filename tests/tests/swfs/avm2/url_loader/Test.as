package {
	public class Test {
	}
}

import flash.net.URLLoader;
import flash.net.URLRequest;
import flash.net.URLLoaderDataFormat;
import flash.events.IOErrorEvent;
import flash.events.Event;
import flash.utils.setInterval;
import flash.utils.clearInterval;

var txtRequest:URLRequest = new URLRequest("data.txt");
var binRequest:URLRequest = new URLRequest("data.bin");
var missingRequest:URLRequest = new URLRequest("missingFile.bin");
var urlLoader:URLLoader = new URLLoader();
urlLoader.addEventListener(Event.OPEN, on_open);
urlLoader.addEventListener(Event.COMPLETE, on_complete);
urlLoader.addEventListener(IOErrorEvent.IO_ERROR, on_error);
urlLoader.load(txtRequest);

var state = "first";

function on_open(evt: Event):void {
	trace("Event.OPEN with: ", evt.target)
	trace("Got data: " + evt.target.data);
}

function on_complete(evt:Event):void {
	trace("Event.COMPLETE with: " + evt.target);
	trace("bytesTotal: " + evt.target.bytesTotal);
	if (state == "first") {
		trace("Loaded text: " + evt.target.data)
		state = "second";
		urlLoader.dataFormat = URLLoaderDataFormat.BINARY;
		urlLoader.load(binRequest);
	} else if (state == "second") {
		trace("Loaded binary with length: " + evt.target.data.bytesAvailable);
		while (evt.target.data.bytesAvailable != 0) {
			trace(evt.target.data.readByte());
		}
	
		state = "third";
		urlLoader.load(missingRequest);
	} else if (state == "third") {
		trace("ERROR: expected `missingRequest` to fail");
	}
}

function on_error(evt:IOErrorEvent):void {
	trace("IOErrorEvent.IO_ERROR: " + evt.target);
	// FIXME - this needs to be implemented in Ruffle
	trace("IOErrorEvent text: " + evt.text);
	trace("Old data: " + evt.target.data);
	
	// Now, perform a load that's started by the constructor
	var loader = new URLLoader(txtRequest);
	// FIXME - setInterval is not currently implemented,
	// so the rest of this test does not work under Ruffle
	/*var interval = setInterval(checkData, 100);

	function checkData() {
		if (loader.data != null) {
			trace("Loaded using constructor: " + loader.data);
			clearInterval(interval);
		}
	}*/
}