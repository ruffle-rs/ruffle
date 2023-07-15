package {
	import flash.utils.ByteArray;

	public class Test {
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