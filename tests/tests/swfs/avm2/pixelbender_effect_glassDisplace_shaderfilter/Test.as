package {
	import flash.display.BitmapData;
	import flash.display.ShaderJob;
	import flash.display.Shader;
	import flash.display.Bitmap;
	import flash.display.MovieClip;
	import flash.display.ShaderParameter;
	import flash.geom.Rectangle;
	import flash.geom.Point;
	import flash.filters.ShaderFilter;
	
	public class Test {
	
		[Embed(source = "YellowFlowers.png")]
		public static var FLOWERS: Class;
		
		[Embed(source = "mandelbrot.png")]
		public static var MANDELBROT: Class;
		
		// Shader from 
		[Embed(source = "glassDisplace.pbj", mimeType="application/octet-stream")]
		public static var GLASSDISPLACE_BYTES: Class;

		public function Test(main: MovieClip) {
			//main.stage.scaleMode = "noScale";
			var flowers: Bitmap = new FLOWERS();
			var mandelbrot: Bitmap = new MANDELBROT();
			var shader = glassDisplace(mandelbrot.bitmapData);

			var width = Math.max(flowers.width, mandelbrot.width);
			var height = Math.max(flowers.height, mandelbrot.height);
				
			
			trace("Flowers rect: " + flowers.bitmapData.rect);
			
			var out1 = new Bitmap(flowers.bitmapData.clone());
			var out2 = new Bitmap(new BitmapData(width, height, true, 0xFF0000FF));
			//var out2 = new Bitmap(new BitmapData(flowers.bitmapData.width, flowers.bitmapData.height, true, 0xFF0000FF));
			
			var filter = new ShaderFilter(shader);
			
			out1.filters = [filter];
			
			trace("ShaderFilter equal: " + (out1.filters[0] === filter));
			trace("Shader equal: " + (out1.filters[0].shader === filter.shader));
			
			trace("Dest rect: " + out2.bitmapData.generateFilterRect(new Rectangle(100, 10, 400, 20), filter));
			out2.bitmapData.applyFilter(flowers.bitmapData, new Rectangle(0, 0, 20, 20), new Point(0, 0), filter);
			out2.y = 390;
			
			
			main.addChild(out1);
			main.addChild(out2);
		}

		private function glassDisplace(input2: BitmapData): Shader {
			// This should be unused, since it's bounded to the first image input
			// (which gets overwritten when applying ShaderFilter)
			var fake = new BitmapData(300, 100, true, 0xFFFF0000);
			var shader = new Shader(new GLASSDISPLACE_BYTES());
			shader.data.center.value = [80, 420];
			shader.data.stretch.value = [180, 20];
			
				
			// Uncomment the following lines to simplify the shader output
			// to make comparisons between Ruffle and Flash easier.
			//shader.data.center.value =[0, 0];
			//shader.data.stretch.value = [0, 0];

			shader.data.alpha.value = [1.0];
			shader.data.src.input = fake;
			shader.data.src2.input = input2;
			return shader;
		}
	}
}