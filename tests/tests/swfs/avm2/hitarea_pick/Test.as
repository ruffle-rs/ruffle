package {
	import flash.display.DisplayObject;
	import flash.display.Sprite;
	import flash.events.MouseEvent;

	[SWF(width="400", height="300")]
	public class Test extends Sprite {
		public function Test() {
			// Own drawing is a 20x20 rect at the slot origin. The hit area is a
			// 200x100 rect offset to (100, 0), so the two regions do not overlap.
			makeSlot("invis", 0, false);
			makeSlot("vis", 160, true);

			stage.addEventListener(MouseEvent.CLICK, function(e:MouseEvent):void {
				var target:DisplayObject = e.target as DisplayObject;
				var name:String = target != null ? target.name : "null";
				trace("click " + name + " (" + e.stageX + ", " + e.stageY + ")");
			});
		}

		private function makeSlot(slotName:String, y:Number, areaVisible:Boolean):void {
			var slot:Sprite = new Sprite();
			slot.name = slotName;
			slot.y = y;
			slot.graphics.beginFill(0x00FF00);
			slot.graphics.drawRect(0, 0, 20, 20);
			slot.graphics.endFill();

			var area:Sprite = new Sprite();
			area.name = slotName + "Area";
			area.graphics.beginFill(0xFF0000);
			area.graphics.drawRect(100, 0, 200, 100);
			area.graphics.endFill();
			area.visible = areaVisible;
			slot.addChild(area);
			slot.hitArea = area;
			addChild(slot);
		}
	}
}
