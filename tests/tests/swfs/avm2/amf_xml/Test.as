package {
	import flash.utils.ByteArray;

	public class Test {
		public function Test() {
			XML.prettyPrinting = false;
			
			var xml =
			<outer attr1="Foo" attr2="Bar">
				<inner attr="First inner">First content</inner>
				<inner attr="Second inner">Second content</inner>
			</outer>;
			
			var data = new ByteArray();
			data.writeObject(xml);
			data.position = 0;
			var bytes = []
			for (var i = 0; i < data.length; i++) {
				bytes.push(data.readUnsignedByte());
			}
			trace("Serialized: " + bytes);
			
			data.position = 0;
			var readBack = data.readObject();
			trace("Original:");
			trace(xml);
			trace("Deserialized:");
			trace(readBack);
			trace("Equal: " + (xml == readBack));
		}
	}
}