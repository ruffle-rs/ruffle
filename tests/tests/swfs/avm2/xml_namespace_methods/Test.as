package  {
	import flash.display.MovieClip;
	import flash.xml.XMLNode;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			XML.prettyPrinting = false;
			
			var xml: XML =
			<root>
				<outer>
					<top xmlns:topns="https://example.org/top" topns:test="top with topns">
						<middle xmlns:middlens="https://example.org/middle" topns:test="middle with topns">
							<bottom xmlns:bottomns="https://example.org/bottom" topns:test="bottom with topns" bottomns:test="bottom with bottomns"/>
							<topns:namespacedTopnsSibling />
							<middlens:namespacedMiddlensSibling />
						</middle>
					</top>
				</outer>
			</root>;
			
			dump(xml);
			
			trace("");
			trace("top.addNamespace(new Namespace(\"topns\", \"https://example.org/top/but/replaced\"))");
			xml.outer[0].top[0].addNamespace(new Namespace("topns", "https://example.org/top/but/replaced"));
			trace("top.addNamespace(new Namespace(\"middlens\", \"https://example.org/middle/but/replaced\"))");
			xml.outer[0].top[0].addNamespace(new Namespace("middlens", "https://example.org/middle/but/replaced"));
			trace("top.addNamespace(new Namespace(\"newns\", \"https://example.org/new\"))");
			xml.outer[0].top[0].addNamespace(new Namespace("newns", "https://example.org/new"));
			trace("top.addNamespace(new Namespace(undefined, \"https://example.org/undefined\"))");
			xml.outer[0].top[0].addNamespace(new Namespace(undefined, "https://example.org/undefined"));
			
			trace("");
			dump(xml);
			trace("");
			trace("");
			trace(xml.toString());
		}
		
		function dump(node: XML) {
			trace("// " + node.localName() + " namespace()");
			traceNs(node.namespace());
			trace("");
			
			trace("// " + node.localName() + " inScopeNamespaces()");
			for each (var ns in node.inScopeNamespaces()) {
				traceNs(ns);
			}
			trace("");
			
			trace("// " + node.localName() + " inScopeNamespaces()");
			for each (var ns in node.inScopeNamespaces()) {
				traceNs(ns);
			}
			trace("");
			
			trace("// " + node.localName() + " namespaceDeclarations()");
			for each (var ns in node.namespaceDeclarations()) {
				traceNs(ns);
			}
			trace("");
			
			trace("// " + node.localName() + " namespace(\"middlens\")");
			trace(node.namespace("middlens"));
			trace("");
			
			for each (var child in node.children()) {
				dump(child);
			}
		}
		
		function traceNs(ns: Namespace) {
			trace(ns.prefix + " = " + ns.uri);
		}
	}
	
}
