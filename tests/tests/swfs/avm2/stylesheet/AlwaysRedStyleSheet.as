package  {
	import flash.text.StyleSheet;
	import flash.text.TextFormat;
	
	public class AlwaysRedStyleSheet extends StyleSheet {
		override public function transform(style: Object): TextFormat {
			var result = new TextFormat();

			result.color = "VERY RED";

			return result;
		}
	}
}