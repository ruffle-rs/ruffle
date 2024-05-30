package {
    import flash.display.Sprite;
    public class Test extends Sprite {}
}

var xml = new XML("<child></child>");
xml.appendChild("First Text");
xml.appendChild("Second Text");
xml.appendChild(new XML("<empty></empty>"));
xml.appendChild("Third Text");

var with_child = new XML("<root></root>");
with_child.appendChild("First Text");
with_child.appendChild("Second Text");
with_child.appendChild(xml.copy());

var whitespace = new XML("<root></root>");
whitespace.appendChild("       ");
whitespace.appendChild("\t \r \n");

var empty = new XML("<root></root>");
empty.appendChild("");
empty.appendChild("");
empty.appendChild("");
empty.appendChild("");

var values = [xml, with_child, whitespace, empty];
for (var i = 0; i < values.length; i++) {
    test(values[i]);
}

function test(xml) {
    var copy = xml.copy();

    trace(repr(xml) + ".normalize() (XML)");
    trace(" Before: " + copy.*.length());
    checkChild(copy.*);
    copy.normalize();
    trace(" After: " + copy.*.length());
    checkChild(copy.*);
    trace();

    var list = xml.copy().*;

    trace(repr(xml) + ".normalize() (XMLList)");
    trace(" Before: " + list.length());
    checkChild(list);
    list.normalize();
    trace(" After: " + list.length());
    checkChild(list);
    trace();
}

function checkChild(list) {
    for (var i = 0; i < list.length(); i++) {
        var child = list[i];
        
        if (child.localName() == "child") {
            trace("  child: " + child.*.length());
        }
    }
}

function repr(value: *) {
    if (value === xml) {
        return "xml";
    } else if (value === with_child) {
        return "with_child";
    } else if (value == whitespace) {
        return "whitespace";
    } else if (value === empty) {
        return "empty";
    } else {
        return typeof(value) + " " + value;
    }
}