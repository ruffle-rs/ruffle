package
{
	import flash.utils.Endian;
	import avm2.intrinsics.memory.si8;
	import avm2.intrinsics.memory.si16;
	import avm2.intrinsics.memory.si32;
	import avm2.intrinsics.memory.sf32;
	import avm2.intrinsics.memory.sf64;
	import avm2.intrinsics.memory.li8;
	import avm2.intrinsics.memory.li16;
	import avm2.intrinsics.memory.li32;
	import avm2.intrinsics.memory.lf32;
	import avm2.intrinsics.memory.lf64;
	import avm2.intrinsics.memory.sxi1;
	import avm2.intrinsics.memory.sxi8;
	import avm2.intrinsics.memory.sxi16;
	import flash.system.ApplicationDomain;
	import flash.utils.ByteArray;
	import flash.display.Sprite;
 
	public class Test extends Sprite
	{ 
		public function Test()
		{
			init();
		}
 
		private function init(): void
		{
            si8(65, 0);
            trace("// li8(0) after si8(65, 0)");
            trace(li8(0));

			si8(255, 0);
            trace("// li8(0) after si8(255, 0)");
            trace(li8(0));

			si16(256, 0);
            trace("// li8(0) after si16(256, 0)");
            trace(li8(0));
			trace("// li8(1) after si16(256, 0)");
            trace(li8(1));
			trace("// li16(0) after si16(256, 0)");
            trace(li16(0));

			si16(0xFFFF, 0);
            trace("// li8(0) after si16(0xFFFF, 0)");
            trace(li8(0));
			trace("// li8(1) after si16(0xFFFF, 0)");
            trace(li8(1));
			trace("// li16(0) after si16(0xFFFF, 0)");
            trace(li16(0));

			si32(0xFFFFFFFF, 0);
            trace("// li8(0) after si32(0xFFFFFFFF, 0)");
            trace(li8(0));
			trace("// li8(1) after si32(0xFFFFFFFF, 0)");
            trace(li8(1));
			trace("// li8(2) after si32(0xFFFFFFFF, 0)");
            trace(li8(2));
			trace("// li8(3) after si32(0xFFFFFFFF, 0)");
            trace(li8(3));
			trace("// li32(0) after si32(0xFFFFFFFF, 0)");
            trace(li32(0));

			sf32(0xFFFFFFFF, 0);
            trace("// li8(0) after sf32(0xFFFFFFFF, 0)");
            trace(li8(0));
			trace("// li8(1) after sf32(0xFFFFFFFF, 0)");
            trace(li8(1));
			trace("// li8(2) after sf32(0xFFFFFFFF, 0)");
            trace(li8(2));
			trace("// li8(3) after sf32(0xFFFFFFFF, 0)");
            trace(li8(3));
			trace("// li32(0) after sf32(0xFFFFFFFF, 0)");
            trace(li32(0));
			trace("// lf32(0) after sf32(0xFFFFFFFF, 0)");
            trace(lf32(0));

			sf32(1234.7654321, 0);
            trace("// li8(0) after sf32(1234.7654321, 0)");
            trace(li8(0));
			trace("// li8(1) after sf32(1234.7654321, 0)");
            trace(li8(1));
			trace("// li8(2) after sf32(1234.7654321, 0)");
            trace(li8(2));
			trace("// li8(3) after sf32(1234.7654321, 0)");
            trace(li8(3));
			trace("// li32(0) after sf32(1234.7654321, 0)");
            trace(li32(0));
			trace("// lf32(0) after sf32(1234.7654321, 0)");
            trace(lf32(0));

			sf64(999999.9999999999, 0);
            trace("// li8(0) after sf64(999999.9999999999, 0)");
            trace(li8(0));
			trace("// li8(1) after sf64(999999.9999999999, 0)");
            trace(li8(1));
			trace("// li8(2) after sf64(999999.9999999999, 0)");
            trace(li8(2));
			trace("// li8(3) after sf64(999999.9999999999, 0)");
            trace(li8(3));
			trace("// li8(4) after sf64(999999.9999999999, 0)");
            trace(li8(4));
			trace("// li8(5) after sf64(999999.9999999999, 0)");
            trace(li8(5));
			trace("// li8(6) after sf64(999999.9999999999, 0)");
            trace(li8(6));
			trace("// li8(7) after sf64(999999.9999999999, 0)");
            trace(li8(7));
			
			trace("// li32(0) after sf64(999999.9999999999, 0)");
            trace(li32(0));
			trace("// li32(4) after sf64(999999.9999999999, 0)");
            trace(li32(4));
			trace("// lf32(0) after sf64(999999.9999999999, 0)");
            trace(lf32(0));
			trace("// lf32(4) after sf64(999999.9999999999, 0)");
            trace(lf32(0));
			trace("// lf64(0) after sf64(999999.9999999999, 0)");
            trace(lf64(0));

            trace("// sxi1(0)");
            trace(sxi1(0));

			trace("// sxi1(1)");
            trace(sxi1(1));

			trace("// sxi1(10)");
            trace(sxi1(10));

			trace("// sxi1(255)");
            trace(sxi1(255));

			trace("// sxi8(0)");
            trace(sxi8(0));

			trace("// sxi8(1)");
            trace(sxi8(1));

			trace("// sxi8(10)");
            trace(sxi8(10));

			trace("// sxi8(255)");
            trace(sxi8(255));

			trace("// sxi16(0)");
            trace(sxi16(0));

			trace("// sxi16(1)");
            trace(sxi16(1));

			trace("// sxi16(10)");
            trace(sxi16(10));

			trace("// sxi16(255)");
            trace(sxi16(255));
		}
	}
}
