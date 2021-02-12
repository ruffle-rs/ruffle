package {
    public class Test {
        public function Test() {}
    }
}

import flash.utils.ByteArray;
import flash.utils.Endian;

var test = new ByteArray();
test.writeUTFBytes("HELLO TEST");
test.position = 0;
trace(test.readByte());
trace(test.readByte());
trace(test.readByte());
trace(test.readByte());
trace(test.readFloat());
test.position -= 4;
test.endian = Endian.LITTLE_ENDIAN;
trace(test.readFloat());
test.clear();
test.writeUTFBytes("Test");
test.position = 0;
trace(test.readUTFBytes(4));
test.position = 3;
test.writeBytes(test);
trace(test.toString());
test.position = 0;
var test2 = new ByteArray();
test.readBytes(test2);
trace(test2.toString());
trace(test2.position);
trace(test.position);
trace(test2.bytesAvailable);
trace(test2.readMultiByte(5, "shift-jis"));
trace(test2.readShort());
test2.clear();
test2.writeMultiByte("次 滋 治 爾 璽 痔 磁 示 而 耳 自 蒔 辞 汐 鹿 ", "shift-jis");
test2.position = 0;
trace(test2.readMultiByte(6, "shift-jis"));
test2.clear();
test2.writeUTF("THIS IS A TEST UTF STRING");
test2.position = 0;
trace(test2.readUTF());
trace(test2[1]);
test2[0] = 90;
trace(test2.toString());