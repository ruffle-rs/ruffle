function codeUnits(value:String):String {
    var result:String = "";

    for (var i:Number = 0; i < value.length; i++) {
        var hex:String = value.charCodeAt(i).toString(16).toUpperCase();

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

function dump(label:String, value:String):Void {
    trace(label + " (" + value.length + " UTF-16 units): " +
        codeUnits(value));
}

// XML metacharacters, newline and carriage return.
var special:String = "<>&'\"\n\r";

// A broad range of Unicode characters.
var wide:String = String.fromCharCode(
    0x00A0, 0x00E9, 0x00F1, // Latin-1
    0x03A9,                  // Greek
    0x0416,                  // Cyrillic
    0x05D0,                  // Hebrew
    0x0627,                  // Arabic
    0x0905,                  // Devanagari
    0x0301,                  // Combining acute accent
    0x3042,                  // Japanese
    0x4E2D,                  // Chinese
    0xAC00,                  // Korean
    0x20AC,                  // Euro sign
    0xD83D, 0xDE00,          // Emoji: U+1F600
    0xD83D, 0xDE80,          // Rocket: U+1F680
    0xD840, 0xDC00           // U+20000
);

// Include unpaired surrogates.
wide += String.fromCharCode(0xD800);
wide += "|";
wide += String.fromCharCode(0xDC00);

var input:String = special + wide;

dump("Source", input);

// Construct XML directly so the input reaches the serializer
// without going through XML parsing or character normalization.
var xml:XML = new XML("<root/>");
var root:XMLNode = xml.firstChild;

root.attributes.value = input;

var textNode:XMLNode = xml.createTextNode(input);
root.appendChild(textNode);

// Test the individual text node.
dump("Text node toString", textNode.toString());

// Test serialization of the element, including its attribute
// and text child.
dump("Root toString", root.toString());

// Also test the XML object's public stringification.
dump("XML toString", xml.toString());
