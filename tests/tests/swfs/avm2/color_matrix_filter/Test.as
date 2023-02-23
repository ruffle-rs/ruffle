package  {
	
	import flash.display.MovieClip;
	import flash.filters.ColorMatrixFilter;
	
	
	public class Test extends MovieClip {
		public function describeFilter(name: String, filter: ColorMatrixFilter) {
			trace("// " + name + ".matrix");
			trace(filter.matrix);
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
			
			trace("// test.filters = [new ColorMatrixFilter()];");
			test.filters = [new ColorMatrixFilter()];
			describeFilters();
			
			trace("// test.filters = [new ColorMatrixFilter([])];");
			test.filters = [new ColorMatrixFilter([])];
			describeFilters();
		}
	}
	
}
