package  {
	
	import flash.display.MovieClip;
	import flash.xml.XMLDocument;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			var xml = "<a><ab>abv</ab>x<ab>abv</ab></a>y<b>b</b>z<!-- d -->z<d></d>";
			var document = new XMLDocument(xml);
			trace("// document.childNodes.length");
			trace(document.childNodes.length);
			trace("");
			
			for (var i = 0; i < document.childNodes.length; i++) {
				trace("// document.childNodes[" + i + "]");
				trace(document.childNodes[i]);
				trace("");
				
				trace("// document.childNodes[" + i + "].attributes");
				trace(document.childNodes[i].attributes);
				trace("");

				trace("// document.childNodes[" + i + "].childNodes");
				trace(document.childNodes[i].childNodes);
				trace("");

				trace("// document.childNodes[" + i + "].firstChild");
				trace(document.childNodes[i].firstChild);
				trace("");

				trace("// document.childNodes[" + i + "].lastChild");
				trace(document.childNodes[i].lastChild);
				trace("");

				trace("// document.childNodes[" + i + "].localName");
				trace(document.childNodes[i].localName);
				trace("");

				trace("// document.childNodes[" + i + "].namespaceURI");
				trace(document.childNodes[i].namespaceURI);
				trace("");

				trace("// document.childNodes[" + i + "].nextSibling");
				trace(document.childNodes[i].nextSibling);
				trace("");

				trace("// document.childNodes[" + i + "].nodeName");
				trace(document.childNodes[i].nodeName);
				trace("");

				trace("// document.childNodes[" + i + "].nodeType");
				trace(document.childNodes[i].nodeType);
				trace("");

				trace("// document.childNodes[" + i + "].nodeValue");
				trace(document.childNodes[i].nodeValue);
				trace("");

				trace("// document.childNodes[" + i + "].parentNode");
				trace(document.childNodes[i].parentNode);
				trace("");

				trace("// document.childNodes[" + i + "].prefix");
				trace(document.childNodes[i].prefix);
				trace("");

				trace("// document.childNodes[" + i + "].previousSibling");
				trace(document.childNodes[i].previousSibling);
				trace("");

			}
		}
	}
	
}
