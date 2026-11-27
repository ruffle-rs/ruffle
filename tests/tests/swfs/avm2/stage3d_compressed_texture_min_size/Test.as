package {
    import flash.display.MovieClip;
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.events.Event;

    // Flash Player throws Error #3745 when creating a compressed texture or
    // cube texture with a side smaller than 4, the size of a compression
    // block. Other formats can be as small as 1x1.
    public class Test extends MovieClip {
        public function Test() {
            var stage3D:Stage3D = stage.stage3Ds[0];
            stage3D.addEventListener(Event.CONTEXT3D_CREATE, function(event:Event):void {
                var ctx:Context3D = stage3D.context3D;
                ctx.enableErrorChecking = true;
                ctx.configureBackBuffer(64, 64, 0, false);

                for each (var format:String in ["compressed", "compressedAlpha", "bgra"]) {
                    for each (var size:Array in [[1, 1], [2, 2], [4, 4], [8, 2], [2, 8], [8, 4]]) {
                        attempt(format + " texture " + size[0] + "x" + size[1], function():void {
                            ctx.createTexture(size[0], size[1], format, false);
                        });
                    }
                    for each (var cube:int in [1, 2, 4]) {
                        attempt(format + " cube " + cube, function():void {
                            ctx.createCubeTexture(cube, format, false);
                        });
                    }
                }
                trace("done");
            });
            stage3D.requestContext3D("auto", "standard");
        }

        private static function attempt(name:String, f:Function):void {
            try {
                f();
                trace(name + ": ok");
            } catch (e:Error) {
                trace(name + ": " + Object(e).constructor + " " + e.errorID + " " + e.message);
            }
        }
    }
}
