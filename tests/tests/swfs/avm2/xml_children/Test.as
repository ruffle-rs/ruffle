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
		}
	}
}