package  {
	
	import flash.display.MovieClip;
	import flash.filters.BlurFilter;
	
	
	public class Test extends MovieClip {
		public function describeFilter(name: String, filter: BlurFilter) {
			trace("// " + name + ".blurX");
			trace(filter.blurX);
			trace("");
			
			trace("// " + name + ".blurY");
			trace(filter.blurY);
			trace("");
			
			trace("// " + name + ".quality");
			trace(filter.quality);
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
			
			trace("// test.filters = [new BlurFilter()];");
			test.filters = [new BlurFilter()];
			describeFilters();
			
			trace("// test.filters = [new BlurFilter(-1.2, 3.4, 5)];");
			test.filters = [new BlurFilter(-1.2, 3.4, 5)];
			describeFilters();
		}
	}
	
}
