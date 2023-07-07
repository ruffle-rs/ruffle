// compiled with mxmlc

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
    	public function Test(){
    	}
    }
}

import flash.utils.describeType;
import flash.utils.getQualifiedClassName;
import flash.utils.getQualifiedSuperclassName;
import flash.utils.Dictionary;
import flash.display.DisplayObject;

// The order of elements in describeType(obj)) depends on the iteration order
// of the internal avmplus Traits hashtable.
// We don't currently reproduce this in Ruffle, so we can't just use 'toXMLString'
// to print the output. Instead, we use this function to re-implement 'toXMLString',
// and normalize the output by printing the children of an element in lexicographic
// order (by their stringified value)
function normalizeXML(data: XML, indent:uint = 0) {
	var output = "";
	for (var i = 0; i < indent; i++) {
		output += " ";
	};
	output += "<" + data.name();
	for each (var attr in data.attributes()) {
		output += " " + attr.name() + "=\"" + attr + "\"";
	}
	if (data.children().length() == 0) {
		output += "/>";
		return output;
	}
	output += ">\n";
	var childStrs = []
	for each (var child in data.children()) {
		childStrs.push(normalizeXML(child, indent + 2));
	}
	childStrs.sort()
	for each (var childStr in childStrs) {
		for (var i = 0 ; i < indent; i++) {
			output += " ";
		}
		output += childStr;
		output += "\n"
	}
	for (var i = 0; i < indent; i++) {
		output += " ";
	};
	output += "</" + data.name() + ">";
	return output;
}

function describeXMLNormalized(val: *) {
	trace(normalizeXML(describeType(val)));
}

class C {}

class Base {
	public function Base(optParam:* = null) {}
	public var baseProp:Object;
	public function baseMethod(): Boolean { return true }
	public function overridenMethod(firstParam: *, secondParam: Dictionary, thirdParam: DisplayObject = null): Object { return null; }
	AS3 function as3Method() {}
}

class Subclass extends Base {
	public var subProp:Object;
	public function subMethod() {}
	public override function overridenMethod(firstParam: *, secondParam: Dictionary, thirdParam: DisplayObject = null): Object { return null; }
}

class HasVector {
    public var vec1: Vector.<int>;
}

describeXMLNormalized(Object);
describeXMLNormalized(new Object());
describeXMLNormalized(Subclass);
describeXMLNormalized(new Subclass());
describeXMLNormalized(C);
describeXMLNormalized(new C());
describeXMLNormalized(int);
describeXMLNormalized(1);
describeXMLNormalized(Class);
describeXMLNormalized(Dictionary);
describeXMLNormalized(new Dictionary());
describeXMLNormalized(new HasVector());
