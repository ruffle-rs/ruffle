package {
	import flash.display.BitmapData;
	import flash.display.ShaderJob;
	import flash.display.Shader;
	import flash.display.Bitmap;
	import flash.display.MovieClip;
	import flash.display.ShaderParameter;
	
	public class Test {
	
		[Embed(source = "Canyonlands.png")]
		public static var CANYONLANDS: Class;
		
		// Shader from 
		[Embed(source = "twirl.pbj", mimeType="application/octet-stream")]
		public static var TWIRL_BYTES: Class;

		public function Test(main: MovieClip) {
			var canyonLands: Bitmap = new CANYONLANDS();
			main.addChild(new Bitmap(twirl(canyonLands.bitmapData)));
		}

		private function twirl(input: BitmapData): BitmapData {
			var out = new BitmapData(input.width, input.height, true, 0xFF00FF00);
			var shader = new ShaderJob(new Shader(new TWIRL_BYTES()), out);
			shader.shader.data.oImage.input = input;
			shader.shader.data.radius.value = [143.453];
			shader.shader.data.center.value = [348.16, 122.88];
			shader.shader.data.twirlAngle.value = [266.4];
			shader.shader.data.gaussOrSinc.value = [0];
			shader.start(true);
			return out;
		}
	}
}