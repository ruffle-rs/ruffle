package {
	import flash.display.MovieClip;
	import flash.media.Video;
	import flash.net.NetStream;
	import flash.net.NetConnection;
	import flash.utils.setTimeout;
	import flash.media.SoundMixer;
	import flash.utils.ByteArray;
	import flash.geom.Point;


	public class Test extends MovieClip {
		// Every tone has this frequency, regardless of channel count and sampling rate.
		static const FREQ = 444.0;

		public static function startTone(mc:MovieClip, file:String) {
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
			mc.addChild(video);
		}

		public static function measureChannel(ba: ByteArray) : Point {
			var sinSum = 0.0;
			var cosSum = 0.0;
			// The first multiplication by 2 is because of the buggy stretch factor
			// (we only get every second sample).
			var coeff = FREQ * 2 * Math.PI * 2 / 44100.0;

			for (var i:uint=0; i<256; i++) {
				var sp = ba.readFloat();
				var arg = i * coeff;
				sinSum += sp * Math.sin(arg);
				cosSum += sp * Math.cos(arg);
			}
			sinSum /= 256.0;
			cosSum /= 256.0;

			// The returned complex number describes both the amplitude and the phase
			// of the tone within the time window we happened to capture it.
			return new Point(cosSum, sinSum);
		}

		// The inverse of measureChannel, assuming its input was a pure sine wave.
		public static function reconstructWave(phasor: Point) : ByteArray {
			var ba = new ByteArray();
			var coeff = FREQ * 2 * Math.PI * 2 / 44100.0;
			for (var i:uint=0; i<256; i++) {
				var arg = i * coeff;
				var val = Math.cos(arg) * phasor.x + Math.sin(arg) * phasor.y;
				// Need to compensate for the "every second sample only" bug here as well.
				ba.writeFloat(val * 2.0);
			}
			ba.position = 0;

			return ba;
		}

		public static function compareWaves(reconstr: ByteArray, ba: ByteArray) {
			for (var i:uint=0; i<256; i++) {
				var diff = Math.abs(reconstr.readFloat() - ba.readFloat());
				if (diff >= 0.025) {
					trace("FAIL: Wave mismatch!", i, diff);
				}
			}
		}

		public static function captureToneAmplitude(ba: ByteArray) : Number {
			// The cursor (position) within the ByteArray is what splits the channel data.
			ba.position = 0;
			var leftPhasor = measureChannel(ba);
			var rightPhasor = measureChannel(ba);

			if (Math.abs(leftPhasor.x - rightPhasor.x) >= 0.01
				|| Math.abs(leftPhasor.y - rightPhasor.y) >= 0.01) {
				trace("FAIL: Channel phase mismatch!");
			}

			var leftAmpl = leftPhasor.length;
			var rightAmpl = rightPhasor.length;

			if (Math.abs(leftAmpl - rightAmpl) >= 0.01) {
				trace("FAIL: Channel mismatch!");
			}

			var leftReconstr = reconstructWave(leftPhasor);
			var rightReconstr = reconstructWave(rightPhasor);

			ba.position = 0;
			// The reconstructed waves should be identical to the original waves.
			compareWaves(leftReconstr, ba);
			compareWaves(rightReconstr, ba);

			// Just in case...
			var avgAmpl = (leftAmpl + rightAmpl) / 2;

			// The default FFmpeg amplitude is 1/8, and we multiply that by 5.
			// Then, with stereo, it gets lowered by 3 dB to split the power between channels.

			// The first multiplication by two is to compensate for the stretch factor bug
			// (us only getting every second sample), the 8 is undoing the default
			// sine amplitude of FFmpeg, and the 5 undoes our own volume scaling.
			return avgAmpl * 2 * 8 / 5;
		}

		function Test() {
			var flvs = [
				"tone_mono_22050hz.flv",
				"tone_mono_44100hz.flv",
				"tone_mono_48000hz.flv",
				"tone_stereo_22050hz.flv",
				"tone_stereo_44100hz.flv",
				"tone_stereo_48000hz.flv",
			];

			for (var i in flvs) {
				// This funky inline function is a workaround to get a proper closure in AS3.
				(function(mc:MovieClip, i:int) {

					setTimeout(function(e){
						startTone(mc, flvs[i]);
					}, i*2000);

					setTimeout(function(e){
						trace("Measuring...");

						var ba:ByteArray = new ByteArray();
						// We could use the `FFTMode` parameter, if it wasn't buggy
						// to the degree that it's mathematically almost meaningless.
						SoundMixer.computeSpectrum(ba);

						var ampl = captureToneAmplitude(ba);
						if (i >= 3) {
							ampl *= Math.pow(10, 3/20); // compensating for the stereo upmix power division
						}

						if (ampl >= 0.95 && ampl <= 1.05) {
							trace("PASS");
						}
						else {
							trace("FAIL");
						}
					}, i*2000 + 800);

				})(this, i);
			}

		}
	}
}