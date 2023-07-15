import flash.display.BitmapData;
import flash.geom.Point;
import flash.geom.Rectangle;

var bmd: BitmapData = createImage();
var otherBmd: BitmapData = createImage();

// Testing bmd against bmd, aligns both images so both points overlap and checks for any opaque overlap
trace("/// hitTest with bmd");
test(bmd, new Point(0, 0), 0, bmd, new Point(0, 0), 0);
test(bmd, new Point(1, 1), 0xFF, bmd, new Point(3, 3), 0xA0);
test(bmd, new Point(2, 1), 0xA0, bmd, new Point(1, 3), 0xA0);
test(bmd, new Point(3, 1), 0xA0, bmd, new Point(1, 2), 0xFF);
test(bmd, new Point(0, 0), 0xA0, bmd, new Point(1, 0), 0xFF);
test(bmd, new Point(1, 1), 0xFF, bmd, new Point(1, 1), 0xFF);
test(bmd, new Point(-1, -1), 0xA0, bmd, new Point(1, 1), 0xA0);
trace("");

trace("/// hitTest with other bmd");
test(bmd, new Point(0, 0), 0, otherBmd, new Point(0, 0), 0);
test(bmd, new Point(1, 1), 0xFF, otherBmd, new Point(3, 3), 0xA0);
test(bmd, new Point(2, 1), 0xA0, otherBmd, new Point(1, 3), 0xA0);
test(bmd, new Point(3, 1), 0xA0, otherBmd, new Point(1, 2), 0xFF);
test(bmd, new Point(0, 0), 0xA0, otherBmd, new Point(1, 0), 0xFF);
test(bmd, new Point(1, 1), 0xFF, otherBmd, new Point(1, 1), 0xFF);
trace("");

// Testing bmd against rect, offsets the rect by -firstPoint and then looks for any opaque pixel inside rect
trace("/// hitTest with rect");
test(bmd, new Point(0, 0), 0xA0, new Rectangle(2, 2, 2, 2));
test(bmd, new Point(0, 0), 0xFF, new Rectangle(0, 0, 3, 4));
test(bmd, new Point(0, 0), 0xFF, new Rectangle(2, 2, 1, 1));
test(bmd, new Point(2, 2), 0xFF, new Rectangle(4, 4, 1, 1));
test(bmd, new Point(-1, 0), 0xA0, new Rectangle(2, 2, 5, 5));
test(bmd, new Point(-10, 10), 0x00, new Rectangle(0, 0, 1, 1));
trace("");

// Testing bmd against point, offsets the point by -firstPoint and then checks if that pixel is opaque
trace("/// hitTest with point");
test(bmd, new Point(0, 0), 0xA0, new Point(2, 2));
test(bmd, new Point(0, 0), 0xFF, new Point(0, 0));
test(bmd, new Point(0, 0), 0xFF, new Point(2, 2));
test(bmd, new Point(2, 2), 0xFF, new Point(4, 4));
test(bmd, new Point(3, 3), 0xFF, new Point(-1, -1));
test(bmd, new Point(-1, -1), 0xA0, new Point(2, 2));
test(bmd, new Point(-1, -1), 0xA0, new Point(0, 0));
test(bmd, new Point(-10, -10), 0x00, new Point(0, 0));
trace("");

var valueOfObject = {
	valueOf: function() { 
		trace("valueOf");
		return 0;
	}
};

trace("/// hitTest with duck-typed objects");
test(bmd, {x: 0, y:0}, 0x00, {x:2, y:2});
test(bmd, {x:valueOfObject, y:0}, 0x00, {x:2, y:2});
test(bmd, {x: 0, y:0}, 0x00, {x:0, y:0, width:1, height: 1});
test(bmd, new Point(0, 0), 0, otherBmd, {x:0, y:0}, 0);
trace("");

trace("/// Error cases")
test(bmd);
test(bmd, null, 0x00, bmd);
test(bmd, {x: 0}, 0x00, bmd); // missing y
test(bmd, {__proto__: {x: 0, y: 0}}, 0x00, bmd); // no proto crawling
test(bmd, new Point(0, 0), 0x00, {});
test(bmd, new Point(0, 0), 0x00, {y: 0}); // missing x
test(bmd, new Point(0, 0), 0x00, bmd, null);
otherBmd.dispose();
test(bmd, new Point(0, 0), 0, otherBmd, new Point(0, 0), 0);
bmd.dispose();
test(bmd, new Point(0, 0), 0, otherBmd, new Point(0, 0), 0);

// BMD looks like: ('-' is no alpha, 'x' is 0xA0, 'X' is 0xFF)
/*   0 1 2 3 4
 * 0 - - - - -
 * 1 - x x x -
 * 2 - x X x -
 * 3 - x x x -
 * 4 - - - - -
	 */
function createImage():BitmapData {
	var bmd: BitmapData = new BitmapData(5, 5, true, 0);
	for (var x = 1; x <= 3; x++) {
		for (var y = 1; y <= 3; y++) {
			bmd.setPixel32(x, y, 0xA0FFFFFF);
		}
	}
	bmd.setPixel32(2, 2, 0xFFFFFFFF);
	return bmd;
}

function formatPoint(point: Point): String {
	if (point) {
		return "new Point(" + point.x + ", " + point.y + ")";
	} else {
		return "null";
	}
}

function formatRectangle(rect: Rectangle): String {
	if (rect) {
		return "new Rectangle(" + rect.x + ", " + rect.y + ", " + rect.width + ", " + rect.height + ")";
	} else {
		return "null";
	}
}


function formatObject(bmd: BitmapData, object: Object): String {
	if (object === bmd) {
		return "bmd";
	} else if (object instanceof Point) {
		return formatPoint(Point(object));
	} else if (object instanceof Rectangle) {
		return formatRectangle(Rectangle(object));
	} else if (object instanceof BitmapData) {
		return "otherBitmapData";
	} else if (object === null) {
		return "null";
	} else if (object === undefined) {
		return "undefined";
	} else {
		var s = "{";
		var i = 0;
		for(var k in object) {
			if( i != 0 ) {
				s += ", ";
			}
			s += k + ":" + object[k];
			i++;
		}
		s += "}";
		return s;
	}
}

function test(bmd, firstPoint, firstAlphaThreshold, secondObject, secondBitmapDataPoint, secondAlphaThreshold) {
	trace("// bmd.hitTest(" + formatObject(bmd, firstPoint) + ", " + firstAlphaThreshold + ", " + formatObject(bmd, secondObject) + ", " + formatObject(bmd, secondBitmapDataPoint) + ", " + secondAlphaThreshold + ")");
	trace(bmd.hitTest(firstPoint, firstAlphaThreshold, secondObject, secondBitmapDataPoint, secondAlphaThreshold));
	trace("");
}
