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
		[Embed(source = "smudge.pbj", mimeType="application/octet-stream")]
		public static var SMUDGE_BYTES: Class;

		public function Test(main: MovieClip) {
			var flowers: Bitmap = new FLOWERS();
			main.addChild(new Bitmap(smudge(flowers.bitmapData.clone())));
		}

		private function smudge(input: BitmapData): BitmapData {
			var shader = new ShaderJob(new Shader(new SMUDGE_BYTES()), input);
			shader.shader.data.bBox.value = [210, 200, 0, 260];
			shader.shader.data.exponent.value = [-7.2];
			shader.shader.data.factor.value = [-6.4];
			shader.shader.data.center.value = [-1.12, 0.5];
			shader.shader.data.size.value = [1.02];
			shader.shader.data.smudge.value = [0.38];
			shader.shader.data.src.input = input;
			shader.start(true);
			return input
		}
	}
}