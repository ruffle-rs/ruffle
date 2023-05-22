package  {
	
	import flash.display.MovieClip;
	import flash.utils.ByteArray;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			test_utf8bytes();
			test_utf8bytes_null();
			test_utf8();
			test_utf8_null();
		}
		
		function test_utf8bytes() {
			var bytearray: ByteArray = new ByteArray();
			bytearray.writeByte(0xEF);
			bytearray.writeByte(0xBB);
			bytearray.writeByte(0xBF);
			bytearray.writeUTFBytes("Text");
			bytearray.position = 0;
			var text: String = bytearray.readUTFBytes(bytearray.length);
			trace("// ba.readUTF8Bytes(" + bytearray.length + ")");
			trace(text);
			trace("// ba.readUTF8Bytes(" + bytearray.length + ").length");
			trace(text.length);
		}
	
		function test_utf8bytes_null() {
			var bytearray: ByteArray = new ByteArray();
			bytearray.writeByte(0xEF);
			bytearray.writeByte(0xBB);
			bytearray.writeByte(0xBF);
			bytearray.writeUTFBytes("Text");
			bytearray.writeByte(0);
			bytearray.position = 0;
			var text: String = bytearray.readUTFBytes(bytearray.length);
			trace("// ba.readUTF8Bytes(" + bytearray.length + ")");
			trace(text);
			trace("// ba.readUTF8Bytes(" + bytearray.length + ").length");
			trace(text.length);
		}
		
		function test_utf8() {
			var bytearray: ByteArray = new ByteArray();
			bytearray.writeShort(3 + 4); // length of BOM + text
			bytearray.writeByte(0xEF);
			bytearray.writeByte(0xBB);
			bytearray.writeByte(0xBF);
			bytearray.writeUTFBytes("Text");
			bytearray.position = 0;
			var text: String = bytearray.readUTF();
			trace("// ba.readUTF8()");
			trace(text);
			trace("// ba.readUTF8().length");
			trace(text.length);
		}
	
		function test_utf8_null() {
			var bytearray: ByteArray = new ByteArray();
			bytearray.writeShort(3 + 4); // length of BOM + text
			bytearray.writeByte(0xEF);
			bytearray.writeByte(0xBB);
			bytearray.writeByte(0xBF);
			bytearray.writeUTFBytes("Text");
			bytearray.writeByte(0);
			bytearray.position = 0;
			var text: String = bytearray.readUTF();
			trace("// ba.readUTF8()");
			trace(text);
			trace("// ba.readUTF8().length");
			trace(text.length);
		}
	}
	
}
