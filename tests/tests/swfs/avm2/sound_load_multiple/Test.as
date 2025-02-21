package {
    import flash.media.*;
    import flash.display.*;
    import flash.net.*;
    import flash.utils.*;

    public class Test extends MovieClip {
        [Embed(source = "sound.mp3", mimeType="application/octet-stream")]
        public static var Mp3ByteArray: Class;

        [Embed(source = "sound.pcm", mimeType="application/octet-stream")]
        public static var PcmByteArray: Class;

        public function Test() {
            var mp3ByteArrayLength = new Mp3ByteArray().length;
            var pcmByteArrayLength = new PcmByteArray().length;

            trace("1");
            var s = new Sound(new URLRequest("sound.mp3"));
            try {
                s.load(new URLRequest("sound.mp3"));
                trace("Passed");
            } catch (e) {
                trace("Failed: " + e);
            }

            trace("2");
            s = new Sound();
            s.load(new URLRequest("sound.mp3"));
            try {
                s.load(new URLRequest("sound.mp3"));
                trace("Passed");
            } catch (e) {
                trace("Failed: " + e);
            }

            trace("3");
            s = new Sound();
            s.loadCompressedDataFromByteArray(new Mp3ByteArray(), mp3ByteArrayLength);
            try {
                s.load(new URLRequest("sound.mp3"));
                trace("Passed");
            } catch (e) {
                trace("Failed: " + e);
            }

            trace("4");
            s = new Sound();
            s.load(new URLRequest("sound.mp3"));
            try {
                s.loadCompressedDataFromByteArray(new Mp3ByteArray(), mp3ByteArrayLength);
                trace("Passed");
            } catch (e) {
                trace("Failed: " + e);
            }

            trace("5");
            s = new Sound();
            s.loadPCMFromByteArray(new PcmByteArray(), pcmByteArrayLength / 8);
            try {
                s.load(new URLRequest("sound.mp3"));
                trace("Passed");
            } catch (e) {
                trace("Failed: " + e);
            }

            trace("6");
            s = new Sound();
            s.load(new URLRequest("sound.mp3"));
            try {
                s.loadPCMFromByteArray(new PcmByteArray(), pcmByteArrayLength / 8);
                trace("Passed");
            } catch (e) {
                trace("Failed: " + e);
            }

            trace("7");
            s = new Sound();
            s.loadCompressedDataFromByteArray(new Mp3ByteArray(), mp3ByteArrayLength);
            try {
                s.loadPCMFromByteArray(new PcmByteArray(), pcmByteArrayLength / 8);
                trace("Passed");
            } catch (e) {
                trace("Failed: " + e);
            }

            trace("8");
            s = new Sound();
            s.loadPCMFromByteArray(new PcmByteArray(), pcmByteArrayLength / 8);
            try {
                s.loadCompressedDataFromByteArray(new Mp3ByteArray(), mp3ByteArrayLength);
                trace("Passed");
            } catch (e) {
                trace("Failed: " + e);
            }

            trace("Finished");
        }
    }
}
