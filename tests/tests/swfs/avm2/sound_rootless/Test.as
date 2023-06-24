package {
    import flash.display.MovieClip;
    import flash.display.Stage;
    import flash.media.Sound;
    import flash.events.Event;
    import flash.net.URLRequest;

    public class Test extends MovieClip {
        var actualStage:Stage;


        public function Test() {
    		    actualStage = stage;

            trace("Initial state");

    			  stage.removeChildAt(0);

    			  trace("Removed root");

            var sound_url:URLRequest = new URLRequest("noise.mp3");
            var sound:Sound = new Sound();
            sound.load(sound_url);

            trace("Loaded sound");

            var sound_channel = sound.play();
            sound_channel.addEventListener(Event.SOUND_COMPLETE, this.PlaybackFinished);
            sound_channel.soundTransform.volume = 0.5;

            trace("Playing sound");

            trace(sound_channel);
        }

        private function PlaybackFinished(evt:Event) {
            trace("Finished playback");
            actualStage.addChild(this);
            trace("Attached root");
        }
    }
}
