package {
    import flash.display.*;
    import flash.media.Video;

    public class Test extends MovieClip {
        public var placedShape:DisplayObject;

        public var placedMorph:DisplayObject;

        public var placedSprite:DisplayObject;

        public var placedVideo:DisplayObject;

        public var placedButton:DisplayObject;

        public var placedText:DisplayObject;

        public function Test() {
            super();

            trace(this.placedShape);
            trace(this.placedMorph);
            trace(this.placedSprite);
            trace(this.placedVideo);
            trace(this.placedButton);
            trace(this.placedText);

            for (var i = 0; i < this.numChildren; i ++) {
                var child:DisplayObject = this.getChildAt(i);
                trace(child);
                if (child != null) {
                    traceInstanceName(child.name);
                }
            }

            var c:DisplayObject = new Bitmap();
            traceInstanceName(c.name);

            c = new Shape();
            traceInstanceName(c.name);

            c = new Video();
            traceInstanceName(c.name);
        }

        static function traceInstanceName(name:String):void {
            if (name.indexOf("instance") == 0) {
                trace("instanceXX");
            } else {
                trace(name);
            }
        }
    }
}

