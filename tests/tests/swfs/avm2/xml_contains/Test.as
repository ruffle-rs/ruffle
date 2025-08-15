package {
    import flash.display.Sprite;
    public class Test extends Sprite {}
}

var list1 = new XMLList("<item>A</item><item>B</item>");
var list2 = new XMLList("<item>A</item>");
var list3 = new XMLList("<item>B</item>");

var x = new XML("<item>A</item>");
var x1 = <item>A</item>;
var x2 = new XML("<item>B</item>");

var values = [null, undefined, "A", "B", "C", list1, list2, list3, x, x1, x2];

for (var i = 0; i < values.length; i++) {
    for (var j = 0; j < values.length; j++) {
        test(values[i], values[j]);
    }
}

function test(self, a) {
    if (!(self is XMLList) && !(self is XML)) {
        return;
    }

    trace(repr(self) + ".contains(" + repr(a) + ")");
    try {
        trace(self.contains(a));
    } catch (ex) {
        trace("! " + ex);
    }
    trace();
}

function repr(value: *) {
    if (value === undefined) {
        return "undefined";
    } else if (value === null) {
        return "null";
    } else if (value === list1) {
        return "list1";
    } else if (value === list2) {
        return "list2";
    } else if (value === list3) {
        return "list3";
    } else if (value === x) {
        return "x";
    } else if (value === x1) {
        return "x1";
    } else if (value === x2) {
        return "x2";
    } else if (value is String) {
        return escapeString(value);
    } else {
        return typeof(value) + " " + value;
    }
}

function escapeString(input: String): String {
    var output:String = "\"";
    for (var i:int = 0; i < input.length; i++) {
        var char:String = input.charAt(i);
        switch (char) {
            case "\\":
                output += "\\\\";
                break;
            case "\"":
                output += "\\\"";
                break;
            case "\n":
                output += "\\n";
                break;
            case "\r":
                output += "\\r";
                break;
            case "\t":
                output += "\\t";
                break;
            default:
                output += char;
        }
    }
    return output + "\"";
}