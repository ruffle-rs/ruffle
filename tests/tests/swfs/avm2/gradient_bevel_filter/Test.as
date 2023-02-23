package  {
	
	import flash.display.MovieClip;
	import flash.filters.GradientBevelFilter;
	import flash.geom.Point;
	import flash.display.BitmapData;
	
	
	public class Test extends MovieClip {
		public function describeFilter(name: String, filter: GradientBevelFilter) {
			trace("// " + name + ".alphas");
			trace(filter.alphas);
			trace("");
			
			trace("// " + name + ".angle");
			trace(filter.angle);
			trace("");
			
			trace("// " + name + ".blurX");
			trace(filter.blurX);
			trace("");
			
			trace("// " + name + ".blurY");
			trace(filter.blurY);
			trace("");
			
			trace("// " + name + ".colors");
			trace(filter.colors);
			trace("");
			
			trace("// " + name + ".distance");
			trace(filter.distance);
			trace("");
			
			trace("// " + name + ".knockout");
			trace(filter.knockout);
			trace("");
			
			trace("// " + name + ".quality");
			trace(filter.quality);
			trace("");
			
			trace("// " + name + ".ratios");
			trace(filter.ratios);
			trace("");
			
			trace("// " + name + ".strength");
			trace(filter.strength);
			trace("");
			
			trace("// " + name + ".type");
			trace(filter.type);
			trace("");
		}
		
		public function describeFilters() {
			for (var i = 0; i < test.filters.length; i++) {
				describeFilter("test.filters[" + i + "]", test.filters[i]);
			}
		}
		
		public function Test() {
			trace("// test.filters");
			trace(test.filters);
			trace("");
			
			trace("// test.filters.length");
			trace(test.filters.length);
			trace("");
			
			describeFilters();
			
			test.filters = [];
			describeFilters();
			
			trace("// test.filters = [new GradientBevelFilter()];");
			test.filters = [new GradientBevelFilter()];
			describeFilters();
			
			trace("// test.filters = [new GradientBevelFilter(5, 2, [], 3, 4, 1, 2, \"outer\", true)];");
			test.filters = [new GradientBevelFilter(5, 2, [], [], [], 3, 4, 1, 2, "outer", true)];
			describeFilters();
			
			trace("// test.filters = [new GradientBevelFilter(5, 2, [], 3, 4, 1, 2, \"inner\", true)];");
			test.filters = [new GradientBevelFilter(5, 2, [], [1], [], 3, 4, 1, 2, "inner", true)];
			describeFilters();
		}
	}
	
}
