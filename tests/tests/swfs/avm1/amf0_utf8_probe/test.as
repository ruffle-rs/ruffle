function codeUnits(value:String):String {
    if (value == null) {
        return "(null)";
    }

    var result:String = "";

    for (var i:Number = 0; i < value.length; i++) {
        var hex:String =
            value.charCodeAt(i).toString(16).toUpperCase();

        while (hex.length < 4) {
            hex = "0" + hex;
        }

        if (i > 0) {
            result += " ";
        }

        result += hex;
    }

    return result;
}

var CONNECTION:String = "amf0_utf8_probe";

var high:String = String.fromCharCode(0xD800);
var low:String = String.fromCharCode(0xDC00);
var pair:String = String.fromCharCode(0xD83D, 0xDE00);
var nonAscii:String = String.fromCharCode(0x00E9);

trace("UTF8: source high = " + codeUnits(high));
trace("UTF8: source low = " + codeUnits(low));
trace("UTF8: source pair = " + codeUnits(pair));
trace("UTF8: source nonASCII = " + codeUnits(nonAscii));

// Construct XML through XMLNode methods rather than parsing a string.
var xml:XML = new XML("<root/>");
var root:XMLNode = xml.firstChild;

root.appendChild(
    xml.createTextNode("BMP=" + nonAscii + "; PAIR=" + pair)
);

root.appendChild(
    xml.createTextNode("HIGH=" + high + "; LOW=" + low)
);

trace("UTF8: source XML = " + codeUnits(xml.toString()));

var payload:Object = new Object();

payload.high = high;
payload.low = low;
payload.pair = pair;
payload.nonAscii = nonAscii;
payload.xml = xml;

// Also test a property name containing an unpaired surrogate.
var unusualKey:String = "key_" + high;
payload[unusualKey] = "property value";

var receiver:LocalConnection = new LocalConnection();

receiver.onProbe = function(data:Object):Void {
    trace("UTF8: received high = " + codeUnits(data.high));
    trace("UTF8: received low = " + codeUnits(data.low));
    trace("UTF8: received pair = " + codeUnits(data.pair));
    trace("UTF8: received nonASCII = " + codeUnits(data.nonAscii));

    for (var key in data) {
        if (key.indexOf("key_") == 0) {
            trace("UTF8: received property key = " + codeUnits(key));
        }
    }

    if (data.xml == undefined) {
        trace("UTF8: XML was not received");
    } else {
        trace("UTF8: received XML status = " + data.xml.status);

        var xmlRoot:XMLNode = data.xml.firstChild;

        if (xmlRoot == null) {
            trace("UTF8: XML has no root node");
        } else {
            var node:XMLNode = xmlRoot.firstChild;
            var index:Number = 0;

            while (node != null) {
                trace(
                    "UTF8: received XML text[" + index + "] = " +
                    codeUnits(node.nodeValue)
                );

                node = node.nextSibling;
                index++;
            }
        }
    }
};

trace("UTF8: connected = " + receiver.connect(CONNECTION));

var sender:LocalConnection = new LocalConnection();

sender.onStatus = function(info:Object):Void {
    trace("UTF8: send status = " + info.level +
        ", code = " + info.code);
};

sender.send(CONNECTION, "onProbe", payload);
