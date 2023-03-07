package {
	public class Test {
		import flash.display.MovieClip;
		import flash.display.Shape;
		import flash.display.Sprite;
		
		public function Test(main:MovieClip) {
			var outer = new Sprite();
			var middle = new Sprite();
			middle.mouseEnabled = false;
			middle.mouseChildren = false;
			var shape = new Shape();
			shape.graphics.beginFill(0x0);
			shape.graphics.drawRect(0, 0, 100, 100);
			shape.graphics.endFill();
			
			outer.name = "Outer";
			middle.name = "Middle";
			shape.name = "Shape";
			
			main.addChild(outer);
			outer.addChild(middle);
			middle.addChild(shape);
		
			var outer2 = new Sprite();
			var middle2 = new Sprite();
		
			var inner = new Sprite();
			inner.graphics.beginFill(0x00FF00);
			inner.graphics.drawRect(0, 0, 100, 100);
			inner.graphics.endFill();
			
			middle2.mouseEnabled = false;
			middle2.mouseChildren = false;
			inner.mouseEnabled = false;
			inner.mouseChildren = false;
			
			outer2.name = "Outer2";
			middle2.name = "Middle2";
			inner.name = "Inner";
		
			outer2.x = 300;
			
			main.addChild(outer2);
			outer2.addChild(middle2);
			middle2.addChild(inner);
			
			main.stage.addEventListener("mouseDown", function(e) {
				trace("mouseDown: target=" + e.target.name + " stageX= "+ e.stageX + " stageY=" + e.stageY);
			});		
		}
	}
}