package {
	public class Test {
		public function Test() {}
	}
}

import flash.utils.getQualifiedClassName;
import flash.utils.ByteArray;
import flash.net.registerClassAlias;

class MyClass {
	public var secondProp: Object;
	public var firstProp: String;
	public var thirdProp: Number;
	private var privProp: String = "Default Private prop";
	
	public function MyClass(priv:String = "Constructor private prop") {
		this.privProp = priv;
	}
	
	public function toString() {
		trace("MyClass(firstProp= " + this.firstProp + " secondProp=" + this.secondProp + " thirdProp=" + this.thirdProp + " privProp=" + this.privProp);
	}
}

roundtrip(new Object());
var custom = new Object();
custom.first = "Hello";
roundtrip(custom);

registerClassAlias("MyClassAlias", MyClass);

var mycls = new MyClass("Overwritten private prop");
mycls.firstProp = "Hello";
mycls.secondProp = null;
mycls.thirdProp = -5.1;
// Note - Flash player appears to serialize properties in
// vtable order, which cannot in general reproduce. Our raw
// bytes match for this particular class definition, but all
// other tests should only test the bytes for single-field classes
// in order to make it easier to match the exact bytes from Flash Player
roundtrip(mycls);

function dump(obj: *) {
	var keys = [];
	for (var key in obj) {
		keys.push(key);
	}
	keys.sort();
	var out = ""
	for each (var key in keys) {
		out += key + "=" + obj[key] + ",";
	}
	trace(out);
}

function roundtrip(obj: Object) {
	trace("Original: [" + obj + "] class: " + getQualifiedClassName(obj));
	dump(obj);
	var out = new ByteArray();
	out.writeObject(obj);
	out.position = 0;

	var bytes = []
	for (var i = 0; i < out.length; i++) {
		bytes.push(out.readUnsignedByte());
	}
	trace("Serialized: " + bytes);
	out.position = 0;
	var readBack = out.readObject();
	trace("Deserialized: [" + readBack + "] class: " + getQualifiedClassName(readBack));
	dump(readBack);
	trace()
}