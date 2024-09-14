package {
	import flash.utils.ByteArray;
	import flash.utils.getQualifiedClassName;

	public class Test {
		public function Test() {
			roundtrip(Vector.<uint>([100, 200, 300]));
			roundtrip(fixed(Vector.<uint>([500, 600])));
			roundtrip(Vector.<uint>([]));
			
			roundtrip(Vector.<int>([-1, -200, 4]))
			roundtrip(fixed(Vector.<int>([-100])));
			
			roundtrip(Vector.<Number>([-0.0, 0.0, -1, Infinity, 5]));
			
			roundtrip(Vector.<Object>([new Object(), 30, null, undefined, true, "Hello"]));
			roundtrip(Vector.<*>([new Object(), 30, null, undefined, true, "Hello"]));

			
			var first = Vector.<String>(["One", "Two"]);
			var second = Vector.<String>(["Three", "Four"]);
			var vec = Vector.<Vector.<String>>([first, second]);
			
			roundtrip(vec);
			
			roundtrip(Vector.<String>(["First string", "Second string"]));
			
			roundtrip(Vector.<NoAliasClass>([new NoAliasClass("First"), new NoAliasClass("Second")]));
			roundtrip(Vector.<AliasClass>([new AliasClass("Third"), new AliasClass("Fourth")]));
		}
	
		private function fixed(vec: Object): Object {
			vec.fixed = true;
			return vec;
		}
	
		private function roundtrip(v: Object) {
			trace("Original: [" + v + "] fixed: " + v.fixed + " class: " + getQualifiedClassName(v));
			var out = new ByteArray();
			out.writeObject(v);
			out.position = 0;
			
			var bytes = []
			for (var i = 0; i < out.length; i++) {
				bytes.push(out.readUnsignedByte());
			}
			trace("Serialized: " + bytes);
			out.position = 0;
			var readBack = out.readObject();
			trace("Deserialized: [" + readBack + "] fixed: " + readBack.fixed + " class: " + getQualifiedClassName(readBack));
		}
	}
}
import flash.net.registerClassAlias;

dynamic class NoAliasClass {
	public var myField: String;
	
	public function NoAliasClass(myField: String = null) {
		this.myField = myField;
		this.dynamicField = "Dynamic field: " + myField;
	}
}

class AliasClass {
	public var otherField:String;
	
	public function AliasClass(otherField: String = null) {
		this.otherField = otherField;
	}
	
	public function toString() {
		trace("AliasClass(otherField=" + this.otherField + ")");
	}
}

flash.net.registerClassAlias("MyAlias", AliasClass);