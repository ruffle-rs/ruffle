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

class Base implements BaseInterfaceOne, BaseInterfaceTwo {
	public function Base(optParam:* = null) {}
	public var baseProp:Object;
	public function baseMethod(): Boolean { return true }
	public function overridenMethod(firstParam: *, secondParam: Dictionary, thirdParam: DisplayObject = null): Object { return null; }
	private function basePrivate() {}
	//WeirdNS function baseWeirdNS() {}
	AS3 function as3Method() {}
	AS3 function get as3BaseGetter(): Boolean { return true; }
	AS3 function set as3BaseSetter(val: Boolean) { }

	public static function baseNormalStatic() {}
	AS3 static function baseAS3Static() {}
}

class Subclass extends Base implements BaseInterfaceOne, OtherInterface {
	public var subProp:Object;
	public function subMethod() {}
	private function subPrivate() {}
	AS3 function subAs3() {}
	//WeirdNS function subWeirdNS() {}
	public override function overridenMethod(firstParam: *, secondParam: Dictionary, thirdParam: DisplayObject = null): Object { return null; }
	AS3 function get as3SubGetter(): Boolean { return true; }
	AS3 function set as3SubSetter(val: Boolean) { }

	public static function subNormalStatic() {}
	AS3 static function subAS3Static() {}
}

interface BaseInterfaceOne {}
interface BaseInterfaceTwo {}
interface OtherInterface {}

class HasVector {
    public var vec1: Vector.<int>;
}

describeXMLNormalized(Object);
describeXMLNormalized(new Object());
describeXMLNormalized(Base);
describeXMLNormalized(new Base());
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
