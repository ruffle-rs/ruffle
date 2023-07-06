package {
	import flash.display.BitmapData;
	import flash.display.ShaderJob;
	import flash.display.Shader;
	import flash.display.Bitmap;
	import flash.display.MovieClip;
	
	public class Test {
	
		[Embed(source = "YellowFlowers.png")]
		public static var FLOWERS: Class;
		
		// Shader from 
		[Embed(source = "tintype.pbj", mimeType="application/octet-stream")]
		public static var TINTYPE_BYTES: Class;

		public function Test(main: MovieClip) {
			var flowers: Bitmap = new FLOWERS();
			main.addChild(new Bitmap(tintype(flowers.bitmapData.clone())));
		}

		private function tintype(input: BitmapData): BitmapData {
			var shader = new ShaderJob(new Shader(new TINTYPE_BYTES()), input);
			shader.shader.data.grayScale.value = [
				0.9, 0.6094, 0.082,
				0.3086, 0.8, 0.082,
				0.3086, 1.2, 0.7];
			shader.shader.data.contrast.value = [1.83];
			shader.shader.data.mid.value = [1];
			shader.shader.data.src.input = input;
			shader.start(true);
			return input
		}
	}
}