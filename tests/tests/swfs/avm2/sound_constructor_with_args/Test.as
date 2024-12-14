package {
    import flash.display.MovieClip;
    import flash.media.Sound;
    import flash.media.SoundChannel;
    import flash.events.Event;
    import flash.events.ProgressEvent;
    import flash.net.URLRequest;

    public class Test extends MovieClip {
        public function Test() {
            var sound:Sound = new Sound(new URLRequest("noise.mp3"));
            sound.addEventListener(Event.OPEN, OnOpen);
            sound.addEventListener(ProgressEvent.PROGRESS, OnProgress)
            sound.addEventListener(Event.COMPLETE, OnComplete);
            trace("Loaded sound");
            var sound_channel:SoundChannel = sound.play();
            sound_channel.addEventListener(Event.SOUND_COMPLETE, this.PlaybackFinished);
            trace("Playing sound");
        }

        private function OnOpen(evt:Event):void {
            trace("Callback: Open");
        }

        private function OnProgress(evt:Event):void {
            if (evt.target.bytesLoaded >= evt.target.bytesTotal) {
                trace("Callback: Progress - " + evt.target.bytesLoaded + " / " + evt.target.bytesTotal);
            }
        }

        private function OnComplete(evt:Event):void {
            trace("Callback: Complete - " + evt.target.bytesLoaded + " / " + evt.target.bytesTotal);
        }

        private function PlaybackFinished(evt:Event):void {
            trace("Callback: Finished playback");
        }
    }
}
