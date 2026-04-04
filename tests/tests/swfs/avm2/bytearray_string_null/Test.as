package {
	import flash.display.Sprite;
	import flash.utils.ByteArray;

	public class Test extends Sprite {
		public function Test() {
			var trailingNull = new ByteArray();
			trailingNull.writeUTFBytes("ABC");
			trailingNull.writeByte(0);
			trailingNull.position = 0;
			
			trace("trailingNull: " + trailingNull);
			trace("trailingNull.toString().length = " + trailingNull.toString().length);
			var readWithNull = trailingNull.readMultiByte(4, "utf-8");
			trace("readWithNull: " + readWithNull); 
			trace("readWithNull.length: " + readWithNull.length);
			
			trailingNull.position = 0;
			var readWithNullUTFBytes = trailingNull.readUTFBytes(4);
			trace("readWithNullUTFBytes: " + readWithNullUTFBytes); 
			trace("readWithNullUTFBytes.length: " + readWithNullUTFBytes.length);

			var nullUTF16LE = new ByteArray();
			nullUTF16LE.writeShort(0x4100);
			nullUTF16LE.writeShort(0x0000);
			nullUTF16LE.position = 0;
			var readWithNullUTF16LEBytes: String = nullUTF16LE.readMultiByte(4, "utf-16le");
			trace("readWithNullUTF16LEBytes: " + readWithNullUTF16LEBytes);
			trace("readWithNullUTF16LEBytes.length: " + readWithNullUTF16LEBytes.length);

			var nullUTF16BE = new ByteArray();
			nullUTF16BE.writeShort(0x0041);
			nullUTF16BE.writeShort(0x0000);
			nullUTF16BE.position = 0;
			var readWithNullUTF16BEBytes: String = nullUTF16BE.readMultiByte(4, "utf-16be");
			trace("readWithNullUTF16BEBytes: " + readWithNullUTF16BEBytes);
			trace("readWithNullUTF16BEBytes.length: " + readWithNullUTF16BEBytes.length);

			var multipleNull = new ByteArray();
			multipleNull.writeUTFBytes("ABC");
			multipleNull.writeByte(0);
			multipleNull.writeUTFBytes("DEF");
			multipleNull.writeByte(0);
			multipleNull.writeByte(0);
			multipleNull.writeUTFBytes("GHI");
			multipleNull.position = 0;
			
			trace("Multiple null: " + multipleNull);
			trace("multipleNull.toString().length = " + multipleNull.toString().length);
			var readWithMultipleNull = multipleNull.readMultiByte(multipleNull.length, "utf-8");
			trace("readWithMultipleNull: " + readWithMultipleNull); 
			trace("readWithmultipleNull.length: " + readWithMultipleNull.length);
			
			multipleNull.position = 0;
			var readWithMultipleNullUTFBytes = multipleNull.readUTFBytes(multipleNull.length);
			trace("readWithMultipleNullUTFBytes: " + readWithMultipleNullUTFBytes); 
			trace("readWithMultipleNullUTFBytes.length: " + readWithMultipleNullUTFBytes.length);
			
			var lenPrefixAndNull = new ByteArray();
			lenPrefixAndNull.writeUTF("ABCZ");
			lenPrefixAndNull.position -= 1;
			lenPrefixAndNull.writeByte(0);
			lenPrefixAndNull.position = 0;
			
			var readWithLenPrefix = lenPrefixAndNull.readUTF();
			trace("readWithLenPrefix: " + readWithLenPrefix);
			trace("readWithLenPrefix.length: " + readWithLenPrefix.length);
			
			multipleNull.position = 0;
			try {
				multipleNull.readUTFBytes(200)
			} catch (e) {
				trace("Caught error: " + e);
			}
		}
	}
}