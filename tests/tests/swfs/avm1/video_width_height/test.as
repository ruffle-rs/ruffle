// Test the `width` and `height` properties of `Video`.
//
// `width` and `height` are the size of the decoded video stream, while
// `_width` and `_height` are the on-stage bounds of the `Video` clip.
//
// test.swf is compiled from test.swfasm: SWF 8, with a 160x120
// `DefineVideoStream` placed as `the_video`. test_video.flv is a 320x233
// H.263 video, a copy of `avm1/netstream_play_flv/test_video.flv`.

function sz(label) {
	trace(label + ": width=" + the_video.width + " (" + typeof the_video.width + ") height=" + the_video.height + " (" + typeof the_video.height + ") _width=" + the_video._width + " _height=" + the_video._height);
}

trace("proto.hasOwnProperty(width): " + Video.prototype.hasOwnProperty("width"));
trace("proto.hasOwnProperty(height): " + Video.prototype.hasOwnProperty("height"));
trace("inst.hasOwnProperty(width): " + the_video.hasOwnProperty("width"));
trace("inst.hasOwnProperty(height): " + the_video.hasOwnProperty("height"));

sz("unattached");
var nc = new NetConnection();
nc.connect(null);
var ns = new NetStream(nc);
ns.onMetaData = function(info) {};
the_video.attachVideo(ns);
sz("after attachVideo");
ns.play("test_video.flv");
sz("after play");

var lastSize = "0x0";
var frames = 0;
this.onEnterFrame = function() {
	frames++;
	var cur = the_video.width + "x" + the_video.height;
	if (cur != lastSize) {
		sz("size changed");
		lastSize = cur;
	}
	if (frames == 30) {
		sz("before scaling");
		the_video._width = 640;
		the_video._height = 360;
		sz("after _width=640 _height=360");
		the_video._xscale = 50;
		the_video._yscale = 200;
		sz("after _xscale=50 _yscale=200");
		the_video._rotation = 90;
		sz("after _rotation=90");
		the_video._rotation = 0;
		the_video._xscale = 100;
		the_video._yscale = 100;
		sz("after reset");

		// Read-only: assignments are ignored, and there is nothing to delete.
		the_video.width = 99;
		the_video.height = 77;
		sz("after assigning width=99 height=77");
		trace("inst.hasOwnProperty(width): " + the_video.hasOwnProperty("width"));
		delete the_video.width;
		delete the_video.height;
		sz("after delete");

		// The last decoded size is kept after the stream is closed and detached.
		ns.close();
		sz("after ns.close");
		the_video.attachVideo(null);
		sz("after attachVideo(null)");
		trace("done");
		delete this.onEnterFrame;
	}
};
