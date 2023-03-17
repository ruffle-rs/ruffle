package  {
	
	import flash.display.MovieClip;
	import flash.text.TextFormat;
	import flash.text.TextField;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			var font = new SourceCodeProRegular();
			var font_default = new TextFormat();
			var source_code_pro = new TextFormat();
			
			font_default.size = 30;
			
			source_code_pro.font = font.fontName;
			source_code_pro.size = 30;
			
			var y = 0;
			addText(0, (y++) * 30, "", false, false, false);
			addText(0, (y++) * 30, "", false, true, false);
			addText(0, (y++) * 30, "", true, false, false);
			addText(0, (y++) * 30, "", true, true, false);
			y++;
			addText(0, (y++) * 30, font.fontName, false, false, false);
			addText(0, (y++) * 30, font.fontName, false, true, false);
			addText(0, (y++) * 30, font.fontName, true, false, false);
			addText(0, (y++) * 30, font.fontName, true, true, false);
			
			y = 0;
			addText(200, (y++) * 30, "", false, false, true);
			addText(200, (y++) * 30, "", false, true, true);
			addText(200, (y++) * 30, "", true, false, true);
			addText(200, (y++) * 30, "", true, true, true);
			y++;
			addText(200, (y++) * 30, font.fontName, false, false, true);
			addText(200, (y++) * 30, font.fontName, false, true, true);
			addText(200, (y++) * 30, font.fontName, true, false, true);
			addText(200, (y++) * 30, font.fontName, true, true, true);
		}
		
		function addText(x: uint, y: uint, font: String, bold: Boolean, italic: Boolean, embed: Boolean) {
			var format = new TextFormat();
			format.size = 30;
			format.font = font;
			format.bold = bold;
			format.italic = italic;
			var text = new TextField();
			text.embedFonts = embed;
			text.autoSize = "left";
			text.defaultTextFormat = format;
			text.text = "ABCDEF@";
			text.x = x;
			text.y = y;
			addChild(text);
		}
	}
	
}
