package  {
	
	import flash.display.MovieClip;
	import flash.filters.DropShadowFilter;
	
	
	public class Test extends MovieClip {
		public function describeFilter(name: String, filter: DropShadowFilter) {
			trace("// " + name + ".alpha");
			trace(filter.alpha);
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
			
			trace("// " + name + ".color");
			trace(filter.color);
			trace("");
			
			trace("// " + name + ".distance");
			trace(filter.distance);
			trace("");
			
			trace("// " + name + ".hideObject");
			trace(filter.hideObject);
			trace("");
			
			trace("// " + name + ".inner");
			trace(filter.inner);
			trace("");
			
			trace("// " + name + ".knockout");
			trace(filter.knockout);
			trace("");
			
			trace("// " + name + ".quality");
			trace(filter.quality);
			trace("");
			
			trace("// " + name + ".strength");
			trace(filter.strength);
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
			
			trace("// test.filters = [new DropShadowFilter()];");
			test.filters = [new DropShadowFilter()];
			describeFilters();
			
			trace("// test.filters = [new DropShadowFilter(-3.2, 51.23, 0xABCDEF, 0.2, -1, 2.5, 1.2, 2, true, false, true)];");
			test.filters = [new DropShadowFilter(-3.2, 51.23, 0xABCDEF, 0.2, -1, 2.5, 1.2, 2, true, false, true)];
			describeFilters();
		}
	}
	
}
