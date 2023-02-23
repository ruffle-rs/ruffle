package  {
	
	import flash.display.MovieClip;
	import flash.filters.BevelFilter;
	
	
	public class Test extends MovieClip {
		public function describeFilter(name: String, filter: BevelFilter) {
			trace("// " + name + ".angle");
			trace(filter.angle);
			trace("");
			
			trace("// " + name + ".blurX");
			trace(filter.blurX);
			trace("");
			
			trace("// " + name + ".blurY");
			trace(filter.blurY);
			trace("");
			
			trace("// " + name + ".distance");
			trace(filter.distance);
			trace("");
			
			trace("// " + name + ".highlightAlpha");
			trace(filter.highlightAlpha);
			trace("");
			
			trace("// " + name + ".highlightColor");
			trace(filter.highlightColor);
			trace("");
			
			trace("// " + name + ".knockout");
			trace(filter.knockout);
			trace("");
			
			trace("// " + name + ".quality");
			trace(filter.quality);
			trace("");
			
			trace("// " + name + ".shadowAlpha");
			trace(filter.shadowAlpha);
			trace("");
			
			trace("// " + name + ".shadowColor");
			trace(filter.shadowColor);
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
			
			trace("// test.filters = [new BevelFilter()];");
			test.filters = [new BevelFilter()];
			describeFilters();
			
			trace("// test.filters = [new BevelFilter(1.0, 15.5, 0xABCDEF, 0.5, 0xFEDBCA, 0.2, 3, 5, 2, 1, \"outer\", true)];");
			test.filters = [new BevelFilter(1.0, 15.5, 0xABCDEF, 0.5, 0xFEDBCA, 0.2, 3, 5, 2, 1, "outer", true)];
			describeFilters();
		}
	}
	
}
