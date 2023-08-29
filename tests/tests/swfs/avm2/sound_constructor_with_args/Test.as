package {
    import flash.display.MovieClip;
    import flash.media.Sound;
    import flash.media.SoundChannel;
    import flash.events.Event;
    import flash.net.URLRequest;

    public class Test extends MovieClip {
        public function Test() {
            var sound:Sound = new Sound(new URLRequest("noise.mp3"));
            trace("Loaded sound");
            var sound_channel:SoundChannel = sound.play();
            sound_channel.addEventListener(Event.SOUND_COMPLETE, this.PlaybackFinished);
            trace("Playing sound");
        }

        private function PlaybackFinished(evt:Event):void {
            trace("Finished playback");
        }
    }
}
