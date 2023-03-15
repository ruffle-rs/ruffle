package {
	import flash.utils.getQualifiedClassName;
	
	public class Test {
		public static function test() {
			var outer = <outer>
				<child kind="A">First Child</child>
				<child kind="B">Second Child</child>
				<child kind="A">Third Child: <p>Inner element</p></child>
			</outer>;
			
			var newChild = <child kind="D">Custom child</child>;
			outer.appendChild(newChild);
			trace("New child: " + outer.children()[3]);
			trace("Equal: " + (newChild === outer.children()[3]));
			
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
		
			var singleSimpleChild = filterB.children()
			trace("filterB.split(' ') = " + filterB.split(' '));
			trace("filterB.indexOf('e') = " + filterB.indexOf('e'));
		
			var normalList = new XMLList("<child name='First child'><nested>Normal Child</nested></child>");
			trace("normalList.nested = " + normalList.nested);
			
			var weirdList = new XMLList("<child name='First child'><name>Weird child</name></child>");
			trace("weirdList.name = " + weirdList.name);
			trace("weirdList.name() = " + weirdList.name());
			trace("weirdList.split.length() = " + weirdList.split.length());
			
			var otherWeirdList = new XMLList("<child name='First child'><split>Other weird child</split></child>");
			trace("weirdList.name = " + weirdList.name);
			trace("weirdList.name() = " + weirdList.name());
		
			// We're accessing this property, not calling it, so it shouldn't
			// be a method
			trace("getQualifiedClassName(filterB.split) = " + getQualifiedClassName(filterB.split));
			
			var simpleXML = new XML("My simple text");
			trace("simpleXML.split(' ') = " + simpleXML.split(' '));
			trace("getQualifiedClassName(simpleXML.split) = " + getQualifiedClassName(simpleXML.split));
			
			var xmlElement = new XML("<p>Inner content</p>");
			trace("xmlElement.split(' ') = " + xmlElement.split(' '));
			
			var sameName = new XMLList("<child attr='Outer'><child attr='Inner'></child></child>");
			trace("sameName.child.@attr = " + sameName.child.@attr);
			
			var weirdXML = new XML("<split>Weird content</split>");
			trace("weirdXMl.split(' ') = " + weirdXML.split(" "));
			
			var emptyList = new XMLList();
			try {
				emptyList.name()
			} catch (e) {
				trace("emptyList.name() threw: " + e);
			}
		
			var multiList = new XMLList("<p>First</p><p>Second</p>");
			try {
				multiList.name()
			} catch (e) {
				trace("multiList.name() threw: " + e);
			}
			
			var complexXML = new XML("<wrapper><p>One Two</p><p>Three Four</p></wrapper>");
			try {
				trace("complexXML.split(' ') = " + complexXML.split(' '));
			} catch (e) {
				// FIXME - Ruffle does not throw an AVM exception for the above error.
				// Uncomment this when it does
				//trace("Caught exception: " + e);
			}
		}
	}
}