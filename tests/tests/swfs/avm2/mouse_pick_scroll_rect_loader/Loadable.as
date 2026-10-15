package {
	import flash.display.Sprite;
	import flash.text.TextField;
	import flash.text.TextFieldType;

	public class Loadable extends Sprite {
		public function Loadable() {
			var tf:TextField = new TextField();
			tf.width = 3000;
			tf.height = 3000;
			tf.selectable = true;
			tf.wordWrap = false;
			tf.type = TextFieldType.INPUT;
			tf.background = true;
			tf.backgroundColor = 0;
			tf.textColor = 0xFF0000;
			tf.text = "loaded content";
			addChild(tf);
		}
	}
}
