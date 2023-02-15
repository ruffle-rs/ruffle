package {
	import flash.display.MovieClip;
	import flash.display.Sprite;
	import flash.display.Shape;
	
	public class Test {

		public static function run(main: MovieClip) {
			var outer = new Sprite();
			var normalChild1 = new Sprite();
			var noMouseChild1 = new Sprite();
			var noMouseChild2 = new Sprite();
			
			outer.name = "outer";
			normalChild1.name = "normalChild1";
			noMouseChild1.name = "noMouseChild1";
			noMouseChild2.name = "noMouseChild2";
			
			outer.graphics.beginFill(0xdddddd);
			outer.graphics.drawRect(0, 0, 200, 200);
			outer.graphics.endFill();
			
			
			noMouseChild1.mouseEnabled = false;
			noMouseChild2.mouseEnabled = false;
			
			noMouseChild1.x = 10;
			noMouseChild2.x = 60;
			
			addShape(normalChild1, 0xFF0000, 100);
			addShape(noMouseChild1, 0x00FF00, 30);
			addShape(noMouseChild2, 0x0000FF, 30);
						
			var subChild = new Sprite();
			subChild.name = "subChild";
			addShape(subChild, 0xaabbcc, 20);
			noMouseChild2.addChild(subChild);
			
			var shapeChild = new Shape();
			shapeChild.graphics.beginFill(0xaabb00);
			shapeChild.graphics.drawCircle(0, 0, 40);
			shapeChild.graphics.endFill();
			
			shapeChild.x = 50;
			shapeChild.y = 110;
			
			outer.addChild(normalChild1);
			outer.addChild(noMouseChild1);
			outer.addChild(noMouseChild2);
			outer.addChild(shapeChild);
			

			
			main.addChild(outer);
			
			main.stage.addEventListener("mouseDown", function(e) {
				trace("Mouse down: " + e.target.name + " " + e.target + " stageX=" + e.stageX + " stageY=" + e.stageY);
			})
		}

		static function addShape(target: Sprite, color: uint, length: uint) {
			var shape = new Shape();
			shape.graphics.beginFill(color);
			shape.graphics.drawRect(0, 0, length, length);
			shape.graphics.endFill();
			target.addChild(shape);
		}		
	}
}