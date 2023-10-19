package  {
	
	import flash.display.MovieClip;
	import flash.text.TextField;
	import flash.text.TextFormat;
	
	
	public class Test extends MovieClip {
		const NUM_ROWS: int = 20;

		public function Test() {
			// Bonus points: all the font names here are arbitrarily capitalised to show that equality is case insensitive

			// SCP doesn't have Regular in this SWF, so see what that prefers to fall back to (spoiler: bold)
			addTextField("Source Code Pro Regular", "Source code Pro", false, false, 0);
			addTextField("Source Code Pro Bold", "Source code Pro", true, false, 1);
			addTextField("Source Code Pro Italic", "Source code Pro", false, true, 2);
			addTextField("Source Code Pro Bold Italic", "Source code Pro", true, true, 3);

			// NS only has regular in this SWF, so show that they all are regular
			addTextField("Noto Sans Regular", "Noto SANS", false, false, 4);
			addTextField("Noto Sans Bold", "Noto SANS", true, false, 5);
			addTextField("Noto Sans Italic", "Noto SANS", false, true, 6);
			addTextField("Noto Sans Bold Italic", "Noto SANS", true, true, 7);

			// UM NF doesn't have Regular or Bold in this SWF, so see what that prefers to fall back to (spoiler: italic)
			addTextField("UbuntuMono NF Regular", "ubuntumono nf", false, false, 8);
			addTextField("UbuntuMono NF Bold", "ubuntumono nf", true, false, 9);
			addTextField("UbuntuMono NF Italic", "ubuntumono nf", false, true, 10);
			addTextField("UbuntuMono NF Bold Italic", "ubuntumono nf", true, true, 11);

			// JBM NF only has bold italic in this SWF, so show they are all bold italic
			addTextField("JetBrainsMono NF Regular", "JETBRAINSMONO NF", false, false, 12);
			addTextField("JetBrainsMono NF Bold", "JETBRAINSMONO NF", true, false, 13);
			addTextField("JetBrainsMono NF Italic", "JETBRAINSMONO NF", false, true, 14);
			addTextField("JetBrainsMono NF Bold Italic", "JETBRAINSMONO NF", true, true, 15);

			// SUI doesn't have Regular or Italic, so see what the others fall back to (Bold & Bold Italic)
			addTextField("Segoe UI Regular", "Segoe ui", false, false, 16);
			addTextField("Segoe UI Bold", "Segoe ui", true, false, 17);
			addTextField("Segoe UI Italic", "Segoe ui", false, true, 18);
			addTextField("Segoe UI Bold Italic", "Segoe ui", true, true, 19);
		}

		function addTextField(text: String, font: String, bold: Boolean, italic: Boolean, y: int) {
			var textField: TextField = new TextField();
			var textFormat: TextFormat = new TextFormat();
			textFormat.font = font;
			textFormat.italic = italic;
			textFormat.bold = bold;
			textFormat.size = 30;

			textField.defaultTextFormat = textFormat;
			textField.embedFonts = true;
			textField.text = text;

			textField.y = Math.floor(stage.stageHeight / NUM_ROWS) * y;
			textField.width = stage.stageWidth;
			addChild(textField);
		}
	}
	
}
