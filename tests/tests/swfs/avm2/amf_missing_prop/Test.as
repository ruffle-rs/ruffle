package  {
	
	import flash.display.MovieClip;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
		}
	}
	
}

import flash.utils.ByteArray;
import flash.net.registerClassAlias;
import flash.net.getClassByAlias;
import flash.net.ObjectEncoding;


class MyFirstClass {
	public var myProp: String;
	public var myOtherProp: String;
	public var oneGoodProp: String;
	public function MyFirstClass(val: String) {
		this.myProp = val;
		this.myOtherProp = "My other val";
		this.oneGoodProp = "The one good prop";
	}
}

class MySecondClass {
	public function set myOtherProp(newVal: String) {
		trace("Called setter with: " + newVal);
		throw new Error("Called myOtherProp setter with " + newVal);
	}

	public var oneGoodProp:String;

	public function toString(): String {
		return "MySecondClass(oneGoodProp = " + this.oneGoodProp + ")";
	}
}

function doRoundTrip(version: int) {
	trace("Roundtrip with AMF version: " + version);
	var bytes = new ByteArray();
	bytes.objectEncoding = version;
	registerClassAlias("MyClass", MyFirstClass);
	bytes.writeObject(new MyFirstClass("My value"));
	bytes.position = 0;
	registerClassAlias("MyClass", MySecondClass);
	trace("Current alias: " + getClassByAlias("MyClass"));
	var roundtrip = bytes.readObject();
	trace("Deserialized: " + roundtrip);
}

// FIXME - Ruffle AMF0 class serialization is broken
//doRoundTrip(ObjectEncoding.AMF0);
trace();
doRoundTrip(ObjectEncoding.AMF3);