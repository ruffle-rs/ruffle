package {
    import flash.display.*;
    import flash.events.*;
    import flash.utils.getTimer;
    import flash.net.URLRequest;

    // Based on https://help.adobe.com/en_US/as3/dev/WS84753F1C-5ABE-40b1-A2E4-07D7349976C4.html
    public class Test extends Sprite {
        // plane vertex coordinates (and t values)
        var x1:Number = -100, y1:Number = -100, z1:Number = 0, t1:Number = 0;
        var x2:Number = 100, y2:Number = -100, z2:Number = 0, t2:Number = 0;
        var x3:Number = 100, y3:Number = 100, z3:Number = 0, t3:Number = 0;
        var x4:Number = -100, y4:Number = 100, z4:Number = 0, t4:Number = 0;
        var focalLength:Number = 200;
        // 2 triangles for 1 plane, indices will always be the same
        var indices:Vector.<int>;

        var container:Sprite;

        var bitmapData:BitmapData; // texture
        var ticks:uint = 0;

        public function Test():void {
            indices = new Vector.<int>();
            indices.push(0, 1, 3, 1, 2, 3);

            container = new Sprite(); // container to draw triangles in
            container.x = 200;
            container.y = 200;
            addChild(container);
            this.loadImage();
        }

        function onImageLoaded(event:Event):void {
            var loader:Loader = Loader(event.target.loader);
            var info:LoaderInfo = LoaderInfo(loader.contentLoaderInfo);
            this.bitmapData = (info.content as Bitmap).bitmapData;
            // animate every frame
            addEventListener(Event.ENTER_FRAME, rotatePlane);
        }
        function rotatePlane(event:Event):void {
            this.ticks += 1;
            // rotate vertices over time
            var ticker = this.ticks / 5;
            z2 = z3 = -(z1 = z4 = 100 * Math.sin(ticker));
            x2 = x3 = -(x1 = x4 = 100 * Math.cos(ticker));

            // calculate t values
            t1 = focalLength / (focalLength + z1);
            t2 = focalLength / (focalLength + z2);
            t3 = focalLength / (focalLength + z3);
            t4 = focalLength / (focalLength + z4);

            // determine triangle vertices based on t values
            var vertices:Vector.<Number> = new Vector.<Number>();
            vertices.push(x1 * t1, y1 * t1, x2 * t2, y2 * t2, x3 * t3, y3 * t3, x4 * t4, y4 * t4);

            var shiftedVertices:Vector.<Number> = new Vector.<Number>();
            shiftedVertices.push(100 + x1 * t1, 100 + y1 * t1, 100 + x2 * t2, 100 + y2 * t2, 100 + x3 * t3, 100 + y3 * t3, 100 + x4 * t4, 100 + y4 * t4);

            // set T values allowing perspective to change
            // as each vertex moves around in z space
            var uvtData:Vector.<Number> = new Vector.<Number>();
            uvtData.push(0, 0, t1, 1, 0, t2, 1, 1, t3, 0, 1, t4);

            var uvData:Vector.<Number> = new Vector.<Number>();
            uvData.push(0, 0, 1, 0, 1, 1, 0, 1);

            // draw
            container.graphics.clear();

            container.graphics.beginBitmapFill(bitmapData);
            container.graphics.moveTo(0, 0);
            container.graphics.lineTo(300, 0);

            container.graphics.drawTriangles(vertices, indices, uvtData);
            container.graphics.drawTriangles(shiftedVertices, indices, uvData);

            container.graphics.lineTo(300, 100);
            container.graphics.lineTo(0, 100);
        }

        public function loadImage():void {
            var loader:Loader = new Loader();
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, onImageLoaded);
            loader.contentLoaderInfo.addEventListener(IOErrorEvent.IO_ERROR, onIoError);

            var req:URLRequest = new URLRequest("ocean.jpg");
            loader.load(req);
        }

        private function onIoError(event:IOErrorEvent):void {
            trace("onIoError: " + event);
        }
    }
}