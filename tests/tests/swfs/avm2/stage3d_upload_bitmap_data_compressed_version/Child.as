package {
    import flash.display.MovieClip;
    import flash.events.Event;

    public class Child extends MovieClip {
        public function Child() {
            addEventListener(Event.ADDED_TO_STAGE, function(event:Event):void {
                Uploads.run("child SWF 17", stage.stage3Ds[0].context3D);
            });
        }
    }
}
