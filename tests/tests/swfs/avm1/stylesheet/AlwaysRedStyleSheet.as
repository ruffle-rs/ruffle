import TextField.StyleSheet;

class AlwaysRedStyleSheet extends StyleSheet {
	public function transform(style) {
		trace("// transform(" + style + ")");
		var result = new TextFormat();

		result.color = 0xFF0000;

		return result;
	}
}