package {
	import flash.display.BitmapData;
	import flash.display.ShaderJob;
	import flash.display.Shader;
	import flash.display.Bitmap;
	import flash.display.MovieClip;
	import flash.display.ShaderParameter;
	
	public class Test {
	
		[Embed(source = "YellowFlowers.png")]
		public static var FLOWERS: Class;
		
		[Embed(source = "mandelbrot.png")]
		public static var MANDELBROT: Class;
		
		// Shader from 
		[Embed(source = "glassDisplace.pbj", mimeType="application/octet-stream")]
		public static var GLASSDISPLACE_BYTES: Class;

		public function Test(main: MovieClip) {
			main.stage.scaleMode = "noScale";
			var flowers: Bitmap = new FLOWERS();
			var mandelbrot: Bitmap = new MANDELBROT();
			main.addChild(new Bitmap(glassDisplace(flowers.bitmapData.clone(), mandelbrot.bitmapData.clone())));
		}

		private function glassDisplace(input1: BitmapData, input2): BitmapData {
			var out = new BitmapData(Math.max(input1.width, input2.width), Math.max(input1.height, input2.height), true, 0xFF00FF00);
			var shader = new ShaderJob(new Shader(new GLASSDISPLACE_BYTES()), out);
			shader.shader.data.center.value = [80, 420];
			shader.shader.data.stretch.value = [180, 20];
			shader.shader.data.alpha.value = [0.27];
			shader.shader.data.src.input = input1;
			shader.shader.data.src2.input = input2;
			shader.start(true);
			return out;
		}
	}
}