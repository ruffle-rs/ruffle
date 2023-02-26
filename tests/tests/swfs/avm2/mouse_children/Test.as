package {
	import flash.display.MovieClip;
	import flash.text.TextField;
	
	public class Test {
		public static function runTest(main:MovieClip) {
			var outerMovies = permuteMouseModes();
			var middleMovies = permuteMouseModes();
			var innerMovies = permuteMouseModes();
			
			var startX = 0;
			var startY = 0;
			
			var xPos = startX;
			var yPos = startY;
			
									
			main.stage.addEventListener("mouseDown", function(e) {
				trace("MouseDown: target=" + e.target.name + " stageX= " + e.stageX + " stageY = " + e.stageY);
			})
					
			var id = 0;
					
			for each (var outerData in outerMovies) {
				for each (var middleData in middleMovies) {
					for each (var innerData in innerMovies) {
						
						var outer = outerData.create();
						var middle = middleData.create();
						var inner = innerData.create();
						
						outer.graphics.beginFill(0xFF0000);
						outer.graphics.drawRect(0, 0, 60, 60);
						outer.graphics.endFill();
						outer.x = xPos;
						outer.y = yPos;
						outer.name = "outer_ id=" + id + " mouseEnabled=" + outer.mouseEnabled + " mouseChildren = " + outer.mouseChildren;
						
						middle.graphics.beginFill(0x00FF00);
						middle.graphics.drawRect(0, 0, 40, 40);
						middle.graphics.endFill();
						middle.name = "middle id=" + id + " mouseEnabled=" + middle.mouseEnabled + " mouseChildren = " + middle.mouseChildren;
						
						inner.graphics.beginFill(0x0000FF);
						inner.graphics.drawRect(0, 0, 20, 20);
						inner.graphics.endFill();
						inner.name = "inner_ id=" + id + " mouseEnabled=" + inner.mouseEnabled + " mouseChildren = " + inner.mouseChildren;
						
						var text = new TextField();
						text.text = "O: m=" + outer.mouseEnabled + " c=" + outer.mouseChildren + "\n" +
						"M: m=" + middle.mouseEnabled + " c=" + middle.mouseChildren + "\n" +
						"I: m=" + inner.mouseEnabled + " c=" + inner.mouseChildren;
						
						text.x = xPos;
						text.y = yPos + 55;
					
						
						main.addChild(outer);
						outer.addChild(middle);
						middle.addChild(inner);
						text.mouseEnabled = false;
						
						main.addChild(text);
						
						id += 1;
						
						xPos += 110;
						if (xPos >= 800) {
							xPos = startX;
							yPos += 110;
						}
					}
				}
			}
		}
	}
}
import flash.display.MovieClip;

class MovieData {
	var mouseEnabled: Boolean
	var mouseChildren: Boolean
	
	function create():Sprite {
		var movie = new MovieClip();
		movie.mouseEnabled = mouseEnabled;
		movie.mouseChildren = mouseChildren;
		return movie;
	}
}

import flash.display.Sprite;

function permuteMouseModes() {
	var movies = [];
	for each (var mouseEnabled in [true, false]) {
		for each (var mouseChildren in [true, false]) {
			var data = new MovieData();
			data.mouseEnabled = mouseEnabled;
			data.mouseChildren = mouseChildren;
			movies.push(data);
		}
	}
	return movies;
}