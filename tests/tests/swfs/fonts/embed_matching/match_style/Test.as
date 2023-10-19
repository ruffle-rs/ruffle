package  {
	
	import flash.display.MovieClip;
	import flash.text.TextField;
	import flash.text.TextFormat;
	
	
	public class Test extends MovieClip {
		const NUM_ROWS: int = 4;

		public function Test() {
			addTextField("Source Code Pro Regular", "Source Code Pro", false, false, 0);
			addTextField("Source Code Pro Bold", "Source Code Pro", true, false, 1);
			addTextField("Source Code Pro Italic", "Source Code Pro", false, true, 2);
			addTextField("Source Code Pro Bold Italic", "Source Code Pro", true, true, 3);
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
