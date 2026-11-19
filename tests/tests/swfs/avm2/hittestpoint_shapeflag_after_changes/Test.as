package {
    import flash.display.*;
    import flash.events.Event;

    // hitTestPoint(x, y, true) right after a shape is changed, in the same frame,
    // before the change is rendered. Each case starts with a 30x30 square that has
    // been rendered. Each row of "#" (hit) and "." (miss) is one row of test points.
    // "replace" clears the shape's graphics and puts in a new shape.
    [SWF(width="800", height="720", frameRate="30")]
    public class Test extends Sprite {
        private const STEP:Number = 7;
        private const TOP:Number = 22;
        private const BOTTOM:Number = 118;
        private const SIZE:Number = 30;
        private const LEFT:Number = 46;
        private const UP:Number = 34;

        private const CASES:Array = [
            {name: "moved 90 px right", steps: ["move 90"], end: {moveX: 90}},
            {name: "moved 15 px right", steps: ["move 15"], end: {moveX: 15}},
            {name: "scaled 2x", steps: ["scale 2"], end: {scale: 2}},
            {name: "flipped left-right", steps: ["flip"], end: {flip: true}},
            {name: "rotated 45 degrees", steps: ["rotate 45"], end: {rotate: 45}},
            {name: "its container moved 90 px right", parent: true, steps: ["parent 90"], end: {moveX: 90}},
            {name: "moved, then copied to a bitmap", steps: ["move 90", "bitmap"], end: {moveX: 90}},
            {name: "scaled 2x, then copied to a bitmap", steps: ["scale 2", "bitmap"], end: {scale: 2}},
            {name: "replaced by a circle", steps: ["replace circle"], end: {shape: "circle"}},
            {name: "replaced by a triangle", steps: ["replace triangle"], end: {shape: "triangle"}},
            {name: "replaced by a square 90 px right", steps: ["replace far"], end: {shape: "far"}},
            {name: "replaced by a bigger square", steps: ["replace big"], end: {shape: "big"}},
            {name: "replaced by a circle, then moved", steps: ["replace circle", "move 90"], end: {shape: "circle", moveX: 90}},
            {name: "replaced by a circle, then scaled 2x", steps: ["replace circle", "scale 2"], end: {shape: "circle", scale: 2}},
            {name: "moved, then replaced by itself", steps: ["move 90", "replace square"], end: {moveX: 90}},
            {name: "replaced by itself, then moved", steps: ["replace square", "move 90"], end: {moveX: 90}}
        ];

        private var boxes:Array = [];
        private var targets:Array = [];
        private var frames:int = 0;

        public function Test() {
            for (var i:int = 0; i < CASES.length; i++) {
                var box:Sprite = new Sprite();
                box.x = 4 + (i % 4) * 199;
                box.y = 4 + int(i / 4) * 176;
                addChild(box);
                boxes.push(box);

                var target:Sprite = new Sprite();
                drawShape(target.graphics, "square");
                if (CASES[i].parent) {
                    var holder:Sprite = new Sprite();
                    place(holder, {});
                    holder.addChild(target);
                    box.addChild(holder);
                } else {
                    place(target, {});
                    box.addChild(target);
                }
                targets.push(target);
            }
            addEventListener(Event.ENTER_FRAME, onFrame);
        }

        private function onFrame(e:Event):void {
            // Wait until every square has been rendered.
            if (++frames < 3) return;
            removeEventListener(Event.ENTER_FRAME, onFrame);
            for (var i:int = 0; i < CASES.length; i++) run(i);
        }

        private function run(i:int):void {
            var c:Object = CASES[i];
            var box:Sprite = boxes[i];
            var target:Sprite = targets[i];

            for each (var step:String in c.steps) {
                var parts:Array = step.split(" ");
                if (parts[0] == "move") target.x += Number(parts[1]);
                if (parts[0] == "scale") target.scaleX = target.scaleY = Number(parts[1]);
                if (parts[0] == "flip") target.scaleX = -1;
                if (parts[0] == "rotate") target.rotation = Number(parts[1]);
                if (parts[0] == "parent") target.parent.x += Number(parts[1]);
                if (parts[0] == "bitmap") new BitmapData(200, 200, true, 0).draw(target);
                if (parts[0] == "replace") {
                    target.graphics.clear();
                    drawShape(target.graphics, parts[1]);
                }
            }
            var hits:Array = scan(box, target);

            // Fresh copies of the shape before and after the change. They have
            // never been rendered, so they are tested where they are.
            var before:Sprite = new Sprite();
            place(before, {});
            drawShape(before.graphics, "square");
            var after:Sprite = new Sprite();
            place(after, c.end);
            drawShape(after.graphics, c.end.shape || "square");
            box.addChild(before);
            box.addChild(after);
            var beforeHits:Array = scan(box, before);
            var afterHits:Array = scan(box, after);
            box.removeChild(before);
            box.removeChild(after);

            trace("// " + (i + 1) + ". " + c.name + ": " + describe(hits, beforeHits, afterHits));
            var k:int = 0;
            for (var y:Number = TOP; y < BOTTOM; y += STEP) {
                var row:String = "";
                for (var x:Number = 5; x < 192; x += STEP) {
                    row += hits[k++] ? "#" : ".";
                }
                trace(row);
            }
        }

        private function scan(box:Sprite, o:DisplayObject):Array {
            var hits:Array = [];
            for (var y:Number = TOP; y < BOTTOM; y += STEP) {
                for (var x:Number = 5; x < 192; x += STEP) {
                    hits.push(o.hitTestPoint(box.x + x, box.y + y, true));
                }
            }
            return hits;
        }

        private function describe(hits:Array, beforeHits:Array, afterHits:Array):String {
            var any:Boolean = false;
            var isBefore:Boolean = true;
            var isAfter:Boolean = true;
            for (var k:int = 0; k < hits.length; k++) {
                if (hits[k]) any = true;
                if (hits[k] != beforeHits[k]) isBefore = false;
                if (hits[k] != afterHits[k]) isAfter = false;
            }
            if (!any) return "nothing";
            if (isAfter) return "the new shape";
            if (isBefore) return "the old shape";
            return "part of the old shape";
        }

        private function place(o:DisplayObject, end:Object):void {
            o.x = LEFT + (end.moveX || 0);
            o.y = UP;
            o.scaleX = o.scaleY = end.scale || 1;
            if (end.flip) o.scaleX = -o.scaleX;
            o.rotation = end.rotate || 0;
        }

        private function drawShape(g:Graphics, kind:String):void {
            g.beginFill(0x808080);
            if (kind == "square") {
                g.drawRect(0, 0, SIZE, SIZE);
            } else if (kind == "far") {
                g.drawRect(90, 0, SIZE, SIZE);
            } else if (kind == "big") {
                g.drawRect(0, 0, SIZE * 2, SIZE * 2);
            } else if (kind == "circle") {
                // Placed so that no test point is near its edge, where Flash and
                // Ruffle draw curves very slightly differently.
                g.drawCircle(15.5, 15, 12.5);
            } else if (kind == "triangle") {
                g.moveTo(0, SIZE);
                g.lineTo(SIZE / 2, 0);
                g.lineTo(SIZE, SIZE);
                g.lineTo(0, SIZE);
            }
            g.endFill();
        }
    }
}
