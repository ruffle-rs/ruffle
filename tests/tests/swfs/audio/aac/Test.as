package {
	import flash.events.Event;
	import flash.display.MovieClip;
	import flash.media.Video;
	import flash.net.URLRequest;
	import flash.net.NetStream;
	import flash.net.NetConnection;
	import flash.utils.setTimeout;
	import flash.media.SoundMixer;
	import flash.utils.ByteArray;

	public class Test {
		
		public static function startTone(main:MovieClip, file:String) {
			trace("Starting tone:", file);
			
			var connection:NetConnection = new NetConnection();
			connection.connect(null);

			var Client:Object = new Object();
			Client.onMetaData = function(e) {};
			
			var stream = new NetStream(connection);
			stream.client = Client;
			stream.play(file);

			var video = new Video();
			video.attachNetStream(stream);
			main.addChild(video);
		}
		
		public static function measureChannel(ba: ByteArray, f: Number) : Number {
			var sinSum = 0.0;
			var cosSum = 0.0;
			// The first multiplication by 2 is because of the buggy stretch factor
			// (we only get every second sample).
			var coeff = f * 2 * Math.PI * 2 / 44100.0;
			
            for (var j:uint=0; j<256; j++) {
                var sp = ba.readFloat();
				var arg = j * coeff;
				sinSum += sp * Math.sin(arg);
				cosSum += sp * Math.cos(arg);
            }
			sinSum /= 256.0;
			cosSum /= 256.0;
			
			return Math.sqrt(sinSum * sinSum + cosSum * cosSum);
		}
		
		public static function getFreq(ba: ByteArray, f: Number) : Number {
			ba.position = 0;
			
			// The default FFmpeg amplitude is 1/8, and we multiply that by 5.
			// Then, with stereo, it gets lowered by 3 dB to split the power between channels.
			
			var leftAmpl = measureChannel(ba, f);
			var rightAmpl = measureChannel(ba, f);
			
			if (Math.abs(leftAmpl - rightAmpl) >= 0.01) {
				trace("FAIL: Channel mismatch!");
			}
			// Just in case...
			var avgAmpl = (leftAmpl + rightAmpl) / 2;

			// The first multiplication by two is to compensate for the stretch factor bug
			// (us only getting every second sample), the 8 is undoing the default
			// sine amplitude of FFmpeg, and the 5 undoes our own volume scaling.
			return avgAmpl * 2 * 8 / 5;
		}
		
		public static function doTest(main: MovieClip) {
			var flvs = [
				"tone_mono_22050hz.flv",
				"tone_mono_44100hz.flv",
				"tone_mono_48000hz.flv",
				"tone_stereo_22050hz.flv",
				"tone_stereo_44100hz.flv",
				"tone_stereo_48000hz.flv",
			];
			
			for (var i in flvs) {
				(function(i:int) {
				 
					setTimeout(function(e){
						startTone(main, flvs[i]);
					}, i*2000);
					
					setTimeout(function(e){
						trace("Measuring...");
						
        				var byteArr:ByteArray = new ByteArray();
                		SoundMixer.computeSpectrum(byteArr);
						
						var fakeFactors = [
						  2,
						  2 * 2,
						  48000.0/44100.0,
						  2 * 48000.0/44100.0,
						  4 * 48000.0/44100.0,
						  0.5 * 48000.0/44100.0,
						  0.25 * 48000.0/44100.0,
						];
						for (var ff in fakeFactors) {
							var freq1 = 444 * fakeFactors[ff];
							var freq2 = 444 / fakeFactors[ff];
							
							trace("Fake freqs:", Math.round(freq1), Math.round(freq2));
							
							var meas1 = getFreq(byteArr, freq1);
							var meas2 = getFreq(byteArr, freq2);
							if (i >= 3) {
								// compensating for the stereo upmix power division
								meas1 *= Math.pow(10, 3/20);
								meas2 *= Math.pow(10, 3/20);
							}
							
							// No frequency can be absent entirely, and no frequency other than the main
							// one can be too strong.
							if (meas1 < 1e-6 || meas2 < 1e-6 || meas1 >= 0.9 || meas2 >= 0.9) {
								trace("FAIL");
							}
							else {
								trace("PASS");
							}
						}
						
						var meas = getFreq(byteArr, 444);
						if (i >= 3) {
							meas *= Math.pow(10, 3/20); // compensating for the stereo upmix power division
						}
						trace("Actual freq:");
						if (meas >= 0.95 && meas <= 1.05) {
							trace("PASS");
						}
						else {
							trace("FAIL");
						}
					}, i*2000 + 800);
					
				})(i);
			}
			
		}
	}
}