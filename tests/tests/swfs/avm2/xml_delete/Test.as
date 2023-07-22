package  {
	
	import flash.display.MovieClip;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			XML.prettyPrinting = false;
			
			var xml = <list><a>a</a><b>b</b><c>c</c></list>;
			var list = xml.children();
			var count = list.length();
			while (count > 0) {
				trace("// xml");
				trace(xml);
				trace("");
				
				trace("// list");
				trace(list);
				trace("");
				
				trace("// list[0]");
				trace(list[0]);
				trace("");
				
				trace("// delete list[0]");
				trace(delete list[0]);
				count--;
			}
			trace("// xml");
			trace(xml);
			trace("");
			
			trace("// list");
			trace(list);
			trace("");
			
			trace("// list[0]");
			trace(list[0]);
			trace("");
			
			trace("--------");
			
			xml = <list><item special="yep" id="a">a</item><item id="b">b</item><item special="very" id="c"><name>foo</name>c</item></list>;
			list = xml.item;
			
			trace("// xml");
			trace(xml);
			trace("");
			
			trace("// list");
			trace(list);
			trace("");
			
			trace("/// delete list.@special");
			trace(delete list.@special);
			trace("");
			
			trace("// xml");
			trace(xml);
			trace("");
			
			trace("// list");
			trace(list);
			trace("");
			
			trace("// delete list[5]");
			trace(delete list[5]);
			trace("");
			
			trace("// delete list[\"name\"]");
			trace(delete list["name"]);
			trace("");
			
			trace("// xml");
			trace(xml);
			trace("");
			
			trace("// list");
			trace(list);
			trace("");
			
			trace("--------");
			xml = <list><item special="yep" id="a">a</item><item id="b">b</item><item special="very" id="c"><name>foo</name>c</item></list>;
			list = xml..@special;
			
			trace("// xml");
			trace(xml);
			trace("");
			
			trace("// list");
			trace(list);
			trace("");
			
			trace("// delete list[0]");
			trace(delete list[0]);
			trace("");
			
			trace("// xml");
			trace(xml);
			trace("");
			
			trace("// list");
			trace(list);
			trace("");
			
			trace("--------");
			xml = <list><item special="yep" id="a">a</item><item id="b">b</item><item special="very" id="c"><name>foo</name>c</item></list>;
			list = xml.item;
			
			trace("// xml");
			trace(xml);
			trace("");
			
			trace("// list");
			trace(list);
			trace("");
			
			trace("// delete list.@*");
			trace(delete list.@*);
			trace("");
			
			trace("// xml");
			trace(xml);
			trace("");
			
			trace("// list");
			trace(list);
			trace("");
		}
	}
	
}
