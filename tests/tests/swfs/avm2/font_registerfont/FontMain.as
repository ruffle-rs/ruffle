package  {
	
	import flash.display.MovieClip;
	import flash.text.Font;
	
	
	public class FontMain extends MovieClip {
		
		
		public function FontMain() {
			dumpFonts();
			
			trace("[fonts.swf] // Font.registerFont(CustomFont)");
			Font.registerFont(CustomFont);
			
			dumpFonts();
		}
		
		function dumpFonts() {
			trace("[fonts.swf] // var fonts = enumerateFonts(false)");
			var fonts = Font.enumerateFonts(false);
			
			trace("[fonts.swf] // fonts.length");
			trace("[fonts.swf] "+ fonts.length);
			trace("[fonts.swf]");
			
			for (var i = 0; i < fonts.length; i++) {
				dumpFont("[fonts.swf] fonts[" + i + "]", fonts[i]);
			}
			trace("[fonts.swf]");
		}
		
		function dumpFont(name: String, font: Font) {
			trace("[fonts.swf] // " + name + ".fontName");
			trace("[fonts.swf] " + font.fontName);
			trace("[fonts.swf]");
			
			trace("[fonts.swf] // " + name + ".fontStyle");
			trace("[fonts.swf] " + font.fontStyle);
			trace("[fonts.swf]");
			
			trace("[fonts.swf] // " + name + ".fontType");
			trace("[fonts.swf] " + font.fontType);
			trace("[fonts.swf]");
			
			trace("[fonts.swf]");
		}
	}
	
}
