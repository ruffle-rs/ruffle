package  {
	
	import flash.display.MovieClip;
	import flash.filters.DisplacementMapFilter;
	import flash.geom.Point;
	import flash.display.BitmapData;
	
	
	public class Test extends MovieClip {
		public function describeFilter(name: String, filter: DisplacementMapFilter) {
			trace("// " + name + ".alpha");
			trace(filter.alpha);
			trace("");
			
			trace("// " + name + ".color");
			trace(filter.color);
			trace("");
			
			trace("// " + name + ".componentX");
			trace(filter.componentX);
			trace("");
			
			trace("// " + name + ".componentY");
			trace(filter.componentY);
			trace("");
			
			trace("// " + name + ".mapBitmap");
			trace(filter.mapBitmap);
			trace("");
			
			trace("// " + name + ".mapPoint");
			trace(filter.mapPoint);
			trace("");
			
			trace("// " + name + ".mode");
			trace(filter.mode);
			trace("");
			
			trace("// " + name + ".scaleX");
			trace(filter.scaleX);
			trace("");
			
			trace("// " + name + ".scaleY");
			trace(filter.scaleY);
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
			
			trace("// test.filters = [new DisplacementMapFilter()];");
			test.filters = [new DisplacementMapFilter()];
			describeFilters();
			
			trace("// test.filters = [new DisplacementMapFilter([])];");
			test.filters = [new DisplacementMapFilter(null, new Point(1.5, -2), 3, 4, 1.1, 2.2, "ignore", 0xABCDEF, 0.2)];
			describeFilters();
		}
	}
	
}
