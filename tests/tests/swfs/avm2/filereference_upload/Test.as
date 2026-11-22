package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

import flash.events.DataEvent;
import flash.events.Event;
import flash.events.HTTPStatusEvent;
import flash.events.IOErrorEvent;
import flash.net.FileFilter;
import flash.net.FileReference;
import flash.net.URLRequest;
import flash.net.URLRequestMethod;
import flash.net.URLVariables;
import flash.utils.setTimeout;

var file = new FileReference();

// Each upload runs once the one before it has finished.
var uploads = [
    function() {
        trace("// upload: GET, whose URLVariables go in the URL, and a field name");
        var vars = new URLVariables();
        vars.s = "1.a-b c";
        var request = new URLRequest("upload_response.xml");
        request.method = URLRequestMethod.GET;
        request.data = vars;
        file.upload(request, "upfile");
    },
    function() {
        trace("// upload: POST with URLVariables");
        var vars = new URLVariables();
        vars.type = "background-image";
        var request = new URLRequest("upload_response.xml");
        request.method = URLRequestMethod.POST;
        request.data = vars;
        file.upload(request);
    },
    function() {
        trace("// upload: HTTP error");
        file.upload(new URLRequest("upload.php?debug-error-statuscode"));
    }
];

// Flash Player stalls if the next upload starts inside an event of the one
// before it.
function next() {
    setTimeout(startNext, 0);
}

function startNext() {
    if (uploads.length > 0) {
        uploads.shift()();
    } else {
        trace("// done");
    }
}

function onEvent(e) {
    if (e is DataEvent) {
        trace(e.type + ": " + e.data);
        next();
    } else if (e is HTTPStatusEvent) {
        // The status differs: Ruffle's test navigator fails with 0.
        trace(e.type);
    } else if (e is IOErrorEvent) {
        // Flash Player adds the absolute URL to the text.
        trace(e.type + " " + e.errorID);
        next();
    } else {
        trace(e.type);
    }
}

// Flash Player sends progress events while it sends the file, which a file
// this small usually skips, so they aren't traced.
for each (var type in [Event.OPEN, Event.COMPLETE,
        DataEvent.UPLOAD_COMPLETE_DATA, HTTPStatusEvent.HTTP_STATUS, IOErrorEvent.IO_ERROR]) {
    file.addEventListener(type, onEvent);
}

trace("// upload before browse");
try {
    file.upload(new URLRequest("upload_response.xml"));
} catch (e) {
    trace(e);
}

file.addEventListener(Event.SELECT, function(e) {
    trace("select: " + file.name + ", " + file.size + " bytes");
    next();
});
file.browse([new FileFilter("debug-select-success", "*.txt")]);
