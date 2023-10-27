package  {
	
	import flash.display.MovieClip;
	import flash.utils.ByteArray;
	import flash.utils.Endian;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			var le = new ByteArray();
			var leBytes = [0xff, 0xfe, 0x0, 0x22, 0x78, 0x0];
			for each (var byte in leBytes) {
				le.writeByte(byte);
			}
			trace("Little endian: " + le);
		
			var be = new ByteArray();
			var beBytes = [0xfe, 0xff, 0x22, 0x0, 0x0, 0x78];
			for each (var byte in beBytes) {
				be.writeByte(byte);
			}
			trace("Big endian: " + be);
		}
	}
	
}
