package {
	import flash.display.Sprite;
	import flash.events.MouseEvent;
	import flash.geom.Rectangle;
	import flash.text.TextField;
	import flash.text.TextFieldType;

	public class Test extends Sprite {
		public function Test() {
			var container:Sprite = new Sprite();
			var tf:TextField = new TextField();
			tf.width = 3000;
			tf.height = 3000;
			tf.selectable = true;
			tf.wordWrap = false;
			tf.type = TextFieldType.INPUT;
			tf.textColor = 0xFF0000;
			tf.text = "scrollRect clip test";

			container.addChild(tf);
			addChild(container);
			container.scrollRect = new Rectangle(0, 0, 300, 200);

			stage.addEventListener(MouseEvent.MOUSE_DOWN, function(e:MouseEvent):void {
				trace("mouseDown: " + e.target + " at " + e.stageX + "," + e.stageY);
			});
		}
	}
}
