package  {
	
	import flash.display.MovieClip;
	import flash.text.Font;
	import flash.utils.describeType;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			trace("// var fonts = enumerateFonts(false)");
			var fonts = Font.enumerateFonts(false);
			
			trace("// fonts.length");
			trace(fonts.length);
			trace("");
			
			for (var i = 0; i < fonts.length; i++) {
				dumpFont("fonts[" + i + "]", fonts[i]);
			}
			
			dumpFont("new Font1()", new Font1());
			dumpFont("new Font()", new Font());
		}
		
		function dumpFont(name: String, font: Font) {
			trace("// " + name + ".fontName");
			trace(font.fontName);
			trace("");
			
			trace("// " + name + ".fontStyle");
			trace(font.fontStyle);
			trace("");
			
			trace("// " + name + ".fontType");
			trace(font.fontType);
			trace("");
			
			trace("// " + name + " is Font1");
			trace(font is Font1);
			trace("");
			
			trace("");
		}
	}
	
}
