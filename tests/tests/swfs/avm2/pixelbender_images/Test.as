package {
	import flash.display.BitmapData;
	import flash.display.ShaderJob;
	import flash.display.Shader;
	import flash.display.Bitmap;
	import flash.display.MovieClip;
	
	public class Test {
	
		[Embed(source = "mandelbrot.png")]
		public static var MANDELBROT: Class;
		
		// Shader from https://github.com/8bitavenue/Adobe-Pixel-Bender-Effects/blob/master/Donut%20Shader.cpp
		[Embed(source = "donut.pbj", mimeType="application/octet-stream")]
		public static var DONUT_BYTES: Class;

		public function Test(main: MovieClip) {
			var mandelbrot: Bitmap = new MANDELBROT();
			main.addChild(new Bitmap(donut(mandelbrot.bitmapData.clone())));
		}

		private function donut(input: BitmapData): BitmapData {
			var shader = new ShaderJob(new Shader(new DONUT_BYTES()), input);
			shader.shader.data.BlockCount.value = [56.5];
			shader.shader.data.Min.value = [0.29];
			shader.shader.data.Max.value = [0.51];
			shader.shader.data.Width.value = [100.0];
			shader.shader.data.Height.value = [100.0];
			shader.shader.data.color.value = [0.34, 0.1, 0.2, 1];
			shader.shader.data.src.input = input;
			shader.start(true);
			return input
		}
	}
}