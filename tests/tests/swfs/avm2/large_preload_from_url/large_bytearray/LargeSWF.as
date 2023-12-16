package  {
	
	import flash.display.MovieClip;
	
	
	public class LargeSWF extends MovieClip {
		
		[Embed(source = "data1.bin", mimeType="application/octet-stream")]
		public static var DATA1: Class;
		
		[Embed(source = "data2.bin", mimeType="application/octet-stream")]
		public static var DATA2: Class;
		
		[Embed(source = "data3.bin", mimeType="application/octet-stream")]
		public static var DATA3: Class;
		
		[Embed(source = "data4.bin", mimeType="application/octet-stream")]
		public static var DATA4: Class;
		
		[Embed(source = "data5.bin", mimeType="application/octet-stream")]
		public static var DATA5: Class;
		
		public function LargeSWF() {
			trace("Calling super() in LargeSWF()");
			super();
			trace("Called super() in LargeSWF()");
		}
	}
	
}
