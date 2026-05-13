// Probe: flash.media.Video.width/height under rotation.
// Verifies that the rotation-aware AABB-spreading formula applies to
// Video just like to other DisplayObjects.

package {
    import flash.display.MovieClip;
    import flash.media.Video;
    import flash.system.fscommand;

    public class Test extends MovieClip {
        public function Test() {
            run();
            fscommand("quit");
        }

        private function run():void {
            section("--- getter after rotation ---");
            getterRow("rot=0",   0);
            getterRow("rot=30",  30);
            getterRow("rot=45",  45);
            getterRow("rot=60",  60);
            getterRow("rot=90",  90);
            getterRow("rot=180", 180);

            section("--- setter after rotation: width=200 on Video(100,100) ---");
            setterRow("rot=0",   0,   200);
            setterRow("rot=30",  30,  200);
            setterRow("rot=45",  45,  200);
            setterRow("rot=90",  90,  200);
            setterRow("rot=180", 180, 200);

            section("--- setter after rotation: height=200 on Video(100,100) ---");
            setterHRow("rot=0",   0,   200);
            setterHRow("rot=45",  45,  200);
            setterHRow("rot=90",  90,  200);

            section("--- non-square Video(160,120) ---");
            var v:Video = new Video(160, 120);
            trace("Video(160,120) rot=0  -> w=" + v.width.toFixed(2) + " h=" + v.height.toFixed(2));
            v.rotation = 45;
            trace("Video(160,120) rot=45 -> w=" + v.width.toFixed(2) + " h=" + v.height.toFixed(2));
            var v2:Video = new Video(160, 120);
            v2.rotation = 45;
            v2.width = 200;
            trace("Video(160,120) rot=45; w=200 -> w=" + v2.width.toFixed(2) + " h=" + v2.height.toFixed(2) + " sx=" + v2.scaleX.toFixed(2) + " sy=" + v2.scaleY.toFixed(2));

            section("--- bounds vs width readback under rotation ---");
            var v3:Video = new Video(100, 100);
            v3.rotation = 45;
            var b:* = v3.getBounds(v3);
            trace("Video(100,100) rot=45 bounds=(" + Number(b.x).toFixed(2) + "," + Number(b.y).toFixed(2) + "," + Number(b.width).toFixed(2) + "," + Number(b.height).toFixed(2) + ") width=" + v3.width.toFixed(2));
        }

        private function getterRow(label:String, rot:Number):void {
            var v:Video = new Video(100, 100);
            v.rotation = rot;
            trace(label + " -> w=" + v.width.toFixed(2) + " h=" + v.height.toFixed(2) + " sx=" + v.scaleX.toFixed(2) + " sy=" + v.scaleY.toFixed(2));
        }

        private function setterRow(label:String, rot:Number, w:Number):void {
            var v:Video = new Video(100, 100);
            v.rotation = rot;
            v.width = w;
            trace(label + " -> w=" + v.width.toFixed(2) + " h=" + v.height.toFixed(2) + " sx=" + v.scaleX.toFixed(2) + " sy=" + v.scaleY.toFixed(2));
        }

        private function setterHRow(label:String, rot:Number, h:Number):void {
            var v:Video = new Video(100, 100);
            v.rotation = rot;
            v.height = h;
            trace(label + " -> w=" + v.width.toFixed(2) + " h=" + v.height.toFixed(2) + " sx=" + v.scaleX.toFixed(2) + " sy=" + v.scaleY.toFixed(2));
        }

        private function section(s:String):void {
            trace("");
            trace(s);
        }
    }
}
