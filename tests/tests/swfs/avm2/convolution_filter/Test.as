package  {
	
	import flash.display.MovieClip;
	import flash.filters.ConvolutionFilter;
	
	
	public class Test extends MovieClip {
		public function describeFilter(name: String, filter: ConvolutionFilter) {
			trace("// " + name + ".alpha");
			trace(filter.alpha);
			trace("");
			
			trace("// " + name + ".bias");
			trace(filter.bias);
			trace("");
			
			trace("// " + name + ".clamp");
			trace(filter.clamp);
			trace("");
			
			trace("// " + name + ".color");
			trace(filter.color);
			trace("");
			
			trace("// " + name + ".divisor");
			trace(filter.divisor);
			trace("");
			
			trace("// " + name + ".matrix");
			trace(filter.matrix);
			trace("");
			
			trace("// " + name + ".matrixX");
			trace(filter.matrixX);
			trace("");
			
			trace("// " + name + ".matrixY");
			trace(filter.matrixY);
			trace("");
			
			trace("// " + name + ".preserveAlpha");
			trace(filter.preserveAlpha);
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
			
			trace("// test.filters = [new ConvolutionFilter()];");
			test.filters = [new ConvolutionFilter()];
			describeFilters();
			
			trace("// test.filters = [new ConvolutionFilter(2, 2, [], 1.5, 0.1, false, false, 0xABCDEF, 0.2)];");
			test.filters = [new ConvolutionFilter(2, 2, [], 1.5, 0.1, false, false, 0xABCDEF, 0.2)];
			describeFilters();
			
			trace("// test.filters = [new ConvolutionFilter(3, 4, [], -1.5, -0.1, true, true, 0xFEDCBA, -1)];");
			test.filters = [new ConvolutionFilter(3, 4, [], -1.5, -0.1, true, true, 0xFEDCBA, -1)];
			describeFilters();
		}
	}
	
}
