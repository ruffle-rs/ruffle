package {
    import flash.display.MovieClip;
    import flash.display.Shader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.text.TextField;
    import flash.utils.ByteArray;
    import flash.utils.setTimeout;

    [SWF(width="420", height="150", frameRate="24", backgroundColor="#DDDDDD")]
    public class Test extends Sprite {
        // In test.swf, frame 2 of timelineProbe uses PlaceObject3 to move
        // the existing target and set its timeline blend mode to Multiply.
        public var timelineProbe:MovieClip;
        [Embed(source="Tint.pbj", mimeType="application/octet-stream")]
        private static const TintBytes:Class;

        public function Test() {
            addEventListener(Event.ENTER_FRAME, setup);
        }

        private function setup(event:Event):void {
            removeEventListener(Event.ENTER_FRAME, setup);
            var timeline:MovieClip = getChildByName("timelineProbe") as MovieClip;
            timeline.gotoAndStop(1);
            var placed:MovieClip = timeline.getChildByName("target") as MovieClip;
            var scripted:Sprite = new Sprite();
            scripted.x = 210;
            addChild(scripted);
            paint(placed);
            paint(scripted);

            var status:TextField = new TextField();
            status.width = 420;
            status.height = 85;
            status.y = 65;
            addChild(status);
            status.text = "Before: " + placed.blendMode + " / " + scripted.blendMode;

            setTimeout(function():void {
                timeline.gotoAndStop(2);
                scripted.blendMode = "normal";
                status.appendText("\nTimeline: " + placed.blendMode
                    + " | frame=" + timeline.currentFrame
                    + " | same instance=" + (timeline.getChildByName("target") === placed)
                    + "\nActionScript: " + scripted.blendMode);
                trace(status.text);
            }, 1500);
        }

        private function paint(object:Sprite):void {
            object.graphics.beginFill(0x000000);
            object.graphics.drawRect(20, 15, 70, 40);
            object.graphics.endFill();
            var shader:Shader = new Shader(new TintBytes() as ByteArray);
            shader.data.tint.value = [0, 0, 1, 1];
            object.blendShader = shader;
        }
    }
}
