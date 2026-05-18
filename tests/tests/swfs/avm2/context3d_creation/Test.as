package {
    import flash.display.MovieClip;
    import flash.display.Stage3D;
    import flash.events.Event;

    public class Test extends MovieClip {
        public function Test() {
            var s3d:Stage3D = stage.stage3Ds[0];;
            var self:Test = this;

            trace(this.currentFrame);

            addEventListener("enterFrame", function():void {
                trace("Running enterFrame, currentFrame=" + self.currentFrame);
            });
            addEventListener("frameConstructed", function():void {
                trace("Running frameConstructed, currentFrame=" + self.currentFrame);
            });
            addEventListener("exitFrame", function():void {
                trace("Running exitFrame, currentFrame=" + self.currentFrame);
            });

            s3d.addEventListener("context3DCreate", context3DCreateHandler);
            s3d.requestContext3D();
        }
        
        public function context3DCreateHandler(e:Event):void {
            trace("context3DCreate dispatched, currentFrame=" + this.currentFrame);
            trace("Stack trace from within context3DCreate: " + new Error().getStackTrace());
        }
    }
}
