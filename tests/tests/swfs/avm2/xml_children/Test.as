package {
	public class Test {
		public static function test() {
			var outer = <outer>
				<child kind="A">First Child</child>
				<child kind="B">Second Child</child>
				<child kind="A">Third Child: <p>Inner element</p></child>
			</outer>;
			
			trace("Children length: " + outer.children().length());
			
			trace("'child' in outer: " + ('child' in outer));
			
			for each (var child in outer.children()) {
				trace("Child kind= "  + child.@kind);
			}
		
			for each (var innerChild in outer.children().children()) {
				trace("Inner child localName " + innerChild.localName());
			}
		
			var empty = <myelem/>;
			trace("Empty children: " + empty.children().length());
		
			var filterA = outer.child.(@kind == "A");
			trace("filterA.length() = " + filterA.length());
			trace("filterA[0].toString() = " + filterA[0].toString());
			trace("filterA[1].name() = " + filterA[1].name());
		
			var filterB = outer.child.(@kind == "B");
			trace("filterB.length() = " + filterB.length());
			trace("filterB[0].toString() = " + filterB[0].toString());
		
			var filterC = outer.child.(@kind == "C");
			trace("filterC.length() = " + filterC.length());
		}
	}
}