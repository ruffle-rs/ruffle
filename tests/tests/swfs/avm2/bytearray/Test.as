﻿package {
    public class Test {
        public function Test() {}
    }
}

function run() {
	import flash.utils.ByteArray;
	import flash.utils.Endian;
	import flash.utils.CompressionAlgorithm;

	var ba1 = new ByteArray();
	var ba2 = new ByteArray();
	ba1.writeUTFBytes("test data");
	ba2.writeUTFBytes("more data");
	ba1.writeBytes(ba2, 3, 2);
	trace("// ba1.writeBytes(ba2, 3, 2);");
	trace(ba1);
	ba1.position = 0;
	ba1.readBytes(ba2, 2, 3);
	trace("// ba1.readBytes(ba2, 2, 3);");
	trace(ba2);
	ba1.position = 0;
	ba1.position = 100;
	ba1.writeUnsignedInt(2);
	trace("// ba1.position = 100;");
	trace("// ba1.writeUnsignedInt(2);");
	trace(ba1.length);
	ba1.clear();
	trace("// ba1.clear();");
	trace(ba1.length);
	ba1.writeDouble(6.6);
	ba1.position = 3;
	trace("// ba1.writeDouble(6.6);");
	trace("// ba1.position = 3;")
	trace("// ba1.bytesAvailable;")
	trace(ba1.bytesAvailable);
	ba1.clear();
	ba1.writeMultiByte("次 滋 治 爾 璽 痔 磁 示 而 耳 自 蒔 辞 汐 鹿 ", "shift-jis");
	trace("// ba1.writeMultiByte(\"次 滋 治 爾 璽 痔 磁 示 而 耳 自 蒔 辞 汐 鹿 \", \"shift-jis\");");
	ba1.position = 0;
	trace("// ba1.readMultiByte(6, \"shift-jis\")");
	trace(ba1.readMultiByte(6, "shift-jis"));
	ba1.clear();

	ba1.writeUTFBytes("abc\x00");
	ba1.writeUTFBytes("def\x00");
	ba1.position = 0;
	var read = ba1.readUTFBytes(8);
	trace("pos", ba1.position);
	trace("bytes available",ba1.bytesAvailable);
	trace("length", read.length);
	ba1.clear();

	ba1.writeShort(8);
	ba1.writeUTFBytes("abc\x00");
	ba1.writeUTFBytes("def\x00");
	ba1.position = 0;
	read = ba1.readUTF();
	trace("pos", ba1.position);
	trace("bytes available",ba1.bytesAvailable);
	trace("length", read.length);
	ba1.clear();

	ba1.writeFloat(3);
	ba1.writeDouble(5);
	ba1.writeInt(-10);
	ba1.writeUnsignedInt(20);
	ba1.writeShort(40);
	ba1.writeShort(22);
	ba1.writeBoolean(false);
	ba1.writeBoolean(true);
	ba1.writeBoolean(10);
	ba1.writeByte(100);
	ba1.writeByte(255);
	ba1.position = 0;
	trace(ba1.readFloat());
	trace(ba1.readDouble());
	trace(ba1.readInt());
	trace(ba1.readUnsignedInt());
	trace(ba1.readShort());
	trace(ba1.readUnsignedShort());
	trace(ba1.readBoolean());
	trace(ba1.readBoolean());
	trace(ba1.readBoolean());
	trace(ba1.readByte());
	trace(ba1.readUnsignedByte());
	ba1.clear();
	ba1.writeFloat(3);
	ba1.writeDouble(5);
	ba1.writeInt(-10);
	ba1.writeUnsignedInt(20);
	ba1.writeShort(40);
	ba1.writeShort(22);
	ba1.writeBoolean(false);
	ba1.writeBoolean(true);
	ba1.writeBoolean(10);
	ba1.writeByte(100);
	ba1.writeByte(255);
	ba1.position = 0;
	trace(ba1.readUnsignedByte());
	trace(ba1.readByte());
	trace(ba1.readBoolean());
	trace(ba1.readBoolean());
	trace(ba1.readBoolean());
	trace(ba1.readUnsignedShort());
	trace(ba1.readShort());
	trace(ba1.readUnsignedInt());
	trace(ba1.readInt());

	var ba2 = new ByteArray();
	ba2.position = 100;
	trace("ba2.position: " + ba2.position);
	ba2.length = 2;
	trace("ba2.position: " + ba2.position);

	var pngHeader = new ByteArray();
	pngHeader.writeByte(137);
	pngHeader.writeByte(80);
	pngHeader.writeByte(78);
	pngHeader.writeByte(71);
	pngHeader.position = 0;

	var headerStr = pngHeader.readUTFBytes(4);
	for (var i = 0; i < headerStr.length; i++) {
		trace("Char code: " + headerStr.charCodeAt(i));
	}
}
run();