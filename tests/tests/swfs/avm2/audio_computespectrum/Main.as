
package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.utils.ByteArray;
    import flash.media.SoundMixer;

    public class Main extends Sprite{
        private var byteArr:ByteArray = new ByteArray();

        // the sample levels in the embedded sound file
        private var levelNums = [0.031, 0.125, 0.25, 0.5, -0.031, -0.125, -0.25, -0.5];
        private var nextLevel = 0;

        public function Main() {
            var sound = new levels();
            sound.play(0, 1000);
            addEventListener(Event.ENTER_FRAME, onFrame);
        }

        private function onFrame(target) {
            // checking whether we are done with the test, if so, not doing anything
            if (nextLevel == levelNums.length)
                return;

            // getting raw output sound samples
            SoundMixer.computeSpectrum(byteArr, false, 0);

            // checking if all the samples are the expected level
            for (var i:uint=0; i<512; i++) {
                var sp = byteArr.readFloat()
                var rsp = (Math.round(sp * 1000) / 1000);
                if (rsp != levelNums[nextLevel])
                    return;
            }

            // if we are now at the next expected level of samples, compute the spectrum as well
            trace()
            trace("level " + nextLevel + ": " + levelNums[nextLevel]);

            // check a few different stretch parameter values
            for (var stretch:uint=0; stretch<4; stretch++) {
                trace();
                trace("stretch = " + stretch);

                SoundMixer.computeSpectrum(byteArr, true, stretch);
                var output:String = "";

                for (var i:uint=0; i<512; i++){
                    var sp = byteArr.readFloat()
                    var rsp = (Math.round(sp * 100) / 100);
                    output += " " + rsp;
                }

                trace(output);
            }

            trace();

            nextLevel += 1
        }
    }
}