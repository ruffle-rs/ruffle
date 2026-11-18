package {
import flash.display.Sprite;
import flash.net.ObjectEncoding;
import flash.utils.ByteArray;

// The AMF0 spec doesn't list Date and XMLDocument as referenceable, but
// Flash Player gives them a reference index when writing and reading.
public class Test extends Sprite {
    public function Test() {
        var obj:Object = {x: 1};

        trace("== writing");
        write("[date, obj, obj]", [new Date(0), obj, obj]);
        write("[obj, date, obj]", [obj, new Date(0), obj]);

        trace("== reading [date, obj, reference]");
        for (var n:int = 0; n < 4; n++) {
            read(n, function(bytes:ByteArray):void {
                bytes.writeByte(0x0B); // date
                bytes.writeDouble(0);
                bytes.writeShort(0);
            });
        }

        trace("== reading [xml document, obj, reference]");
        for (n = 0; n < 4; n++) {
            read(n, function(bytes:ByteArray):void {
                bytes.writeByte(0x0F); // xml document
                bytes.writeUnsignedInt(4);
                bytes.writeUTFBytes("<a/>");
            });
        }
    }

    // Reads a strict array of [first, {x: 1}, reference to `index`].
    private static function read(index:int, writeFirst:Function):void {
        var bytes:ByteArray = new ByteArray();
        bytes.objectEncoding = ObjectEncoding.AMF0;
        bytes.writeByte(0x0A); // strict array
        bytes.writeUnsignedInt(3);
        writeFirst(bytes);
        bytes.writeByte(0x03); // object
        bytes.writeUTF("x");
        bytes.writeByte(0x00);
        bytes.writeDouble(1);
        bytes.writeShort(0);
        bytes.writeByte(0x09); // object end
        bytes.writeByte(0x07); // reference
        bytes.writeShort(index);
        bytes.position = 0;

        try {
            var result:Array = bytes.readObject();
            var value:* = result[2];
            var which:String = value === result ? "the array" : value === result[0] ? "the first element" : value === result[1] ? "the object" : String(value);
            trace("reference " + index + ": " + which);
        } catch (e:Error) {
            trace("reference " + index + ": " + e.name + " " + e.errorID);
        }
    }

    private static function write(name:String, value:*):void {
        var bytes:ByteArray = new ByteArray();
        bytes.objectEncoding = ObjectEncoding.AMF0;
        bytes.writeObject(value);

        var hex:Array = [];
        for (var i:int = 0; i < bytes.length; i++) {
            var b:String = bytes[i].toString(16);
            hex.push(b.length < 2 ? "0" + b : b);
        }

        // Flash writes the local timezone offset after each date (an element
        // key "0".."9" followed by the date marker), which depends on the machine.
        for (i = 3; i + 10 < hex.length; i++) {
            if (hex[i] == "0b" && hex[i - 3] == "00" && hex[i - 2] == "01" && hex[i - 1].charAt(0) == "3") {
                hex[i + 9] = "tz";
                hex[i + 10] = "tz";
            }
        }

        trace(name + ": " + hex.join(" "));
    }
}
}
