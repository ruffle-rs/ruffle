package {
	import flash.display.BitmapData;
	import flash.display.ShaderJob;
	import flash.display.Shader;
	import flash.display.Bitmap;
	import flash.display.MovieClip;
	import flash.display.ShaderParameter;
	
	public class Test {
	
		[Embed(source = "mandelbrot.png")]
		public static var MANDELBROT: Class;
		
		// Shader from 
		[Embed(source = "BlurredFocus.pbj", mimeType="application/octet-stream")]
		public static var BLURREDFOCUS_BYTES: Class;

		public function Test(main: MovieClip) {
			var mandelbrot: Bitmap = new MANDELBROT();
			main.addChild(new Bitmap(blurredFocus(mandelbrot.bitmapData)));
		}

		private function blurredFocus(input: BitmapData): BitmapData {
			var out = new BitmapData(input.width, input.height, true, 0xFF00FF00);
			var shader = new ShaderJob(new Shader(new BLURREDFOCUS_BYTES()), out);
			shader.shader.data.src.input = input;
			shader.shader.data.bBox.value = [160, 280, 200, 150];
			shader.shader.data.center.value = [0.4, 0.12];
			shader.shader.data.size.value = [1.04];
			shader.shader.data.exponent.value = [-0.4];
			shader.shader.data.factor.value = [3.2];
			shader.shader.data.maxBlur.value = [10];
			shader.start(true);
			return out;
		}
	}
}