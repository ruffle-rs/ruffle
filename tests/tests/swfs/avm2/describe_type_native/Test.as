package {
    import flash.display.Sprite;
    import flash.utils.describeType;
    import flash.events.Event;

    public class Test extends Sprite {
        public function Test() {
            trace(this.normalizeXML(describeType(new Event(""))));
        }

        public function normalizeXML(data:XML, indent:uint = 0):* {
            var i:* = undefined;
            var attr:* = undefined;
            var child:* = undefined;
            var childStr:* = undefined;
            var output:* = "";
            i = 0;
            while (i < indent) {
                output += " ";
                i++;
            }
            output += "<" + data.name();
            for each (attr in data.attributes()) {
                output += " " + attr.name() + "=\"" + attr + "\"";
            }
            if (data.children().length() == 0) {
                return output + "/>";
            }
            output += ">\n";
            var childStrs:* = [];
            for each (child in data.children()) {
                childStrs.push(this.normalizeXML(child, indent + 2));
            }
            childStrs.sort();
            for each (childStr in childStrs) {
                i = 0;
                while (i < indent) {
                    output += " ";
                    i++;
                }
                output += childStr;
                output += "\n";
            }
            i = 0;
            while (i < indent) {
                output += " ";
                i++;
            }
            return output + ("</" + data.name() + ">");
        }
    }
}

