package
{
	import flash.utils.Endian;
	import avm2.intrinsics.memory.si8;
	import avm2.intrinsics.memory.li8;
	import avm2.intrinsics.memory.sf32;
	import avm2.intrinsics.memory.li32;
	import avm2.intrinsics.memory.sf64;
	import flash.system.ApplicationDomain;
	import flash.utils.getTimer;
	import flash.utils.ByteArray;
	import flash.text.TextFieldAutoSize;
	import flash.display.StageScaleMode;
	import flash.display.StageAlign;
	import flash.text.TextField;
	import flash.display.Sprite;
 
	public class Test extends Sprite
	{ 
		public function Test()
		{
			init();
		}
 
		private function init(): void
		{
			const SIZE:uint = 10000000;

			var domainMemory:ByteArray = new ByteArray();
			domainMemory.length = SIZE*4 + SIZE*8;
			domainMemory.endian = Endian.LITTLE_ENDIAN;
			ApplicationDomain.currentDomain.domainMemory = domainMemory;
 
            si8(10, 4);
            trace("// li8()");
            trace(li8(4));
		}
	}
}