package {

	public class Test {
	}
}
import flash.utils.ByteArray;

var ba = new ByteArray();


trace("Invalid UTF8 sequences")
ba.writeByte(0xed);
ba.writeByte(0x80);
trace(escape(ba));

ba.clear();
ba.writeByte(0xed);
ba.writeByte(0x70);
ba.writeByte(0x80);
trace(escape(ba));

ba.clear();
ba.writeByte(0xed);
ba.writeByte(0);
ba.writeByte(0x80);
ba.writeByte(0x80);
trace(escape(ba));

ba.clear();
ba.writeByte(0xed);
ba.writeByte(0x80);
ba.writeByte(0x70);
trace(escape(ba));


trace("\nIllegal UTF8 sequences")
ba.clear();
ba.writeByte(0xf9);
ba.writeByte(0x80);
ba.writeByte(0x80);
ba.writeByte(0x80);
ba.writeByte(0x80);
trace(escape(ba))

ba.clear();
ba.writeByte(0xc1);
ba.writeByte(0x80);
trace(escape(ba));

ba.clear();
ba.writeByte(0xed);
ba.writeByte(0xa0);
ba.writeByte(0xbd);
trace(escape(ba));

ba.clear();
ba.writeByte(0xed);
ba.writeByte(0xa0);
ba.writeByte(0xbd);
ba.writeByte(0xed);
ba.writeByte(0xb0);
ba.writeByte(0x8c);
trace(ba);
trace(escape(ba));