// All `ifFrameLoaded` must compile to `ActionWaitForFrame2`.

ifFrameLoaded(0) {
   trace("/// frame 0 marked as loaded");
}

ifFrameLoaded(-43) {
   trace("/// frame -43 marked as loaded");
}

ifFrameLoaded(_totalframes) {
   trace("/// _totalframes marked as loaded");
}

ifFrameLoaded(_totalframes + 5) {
   trace("/// _totalframes + 5 marked as loaded");
}

ifFrameLoaded(Infinity) {
   trace("/// frame Infinity marked as loaded");
}

ifFrameLoaded(NaN) {
   trace("/// frame NaN marked as loaded");
}

ifFrameLoaded("abc") {
   trace("/// frame abc marked as loaded");
}

ifFrameLoaded(undefined) {
   trace("/// frame undefined marked as loaded");
}

ifFrameLoaded(null) {
   trace("/// frame null marked as loaded");
}

ifFrameLoaded(16000) {
   trace("/// frame 16000 marked as loaded");
}

ifFrameLoaded(16001) {
   trace("/// frame 16001 marked as loaded");
}

ifFrameLoaded(16002) {
   trace("/// frame 16002 marked as loaded"); // won't trace
}

ifFrameLoaded(16002.5) {
   trace("/// frame 16002.5 marked as loaded");
}

var n = 2147483647;

ifFrameLoaded(n) {
   trace("/// frame " + n + " marked as loaded"); // won't trace
}

ifFrameLoaded(n + 1) {
   trace("/// frame " + (n + 1) + " marked as loaded"); // won't trace
}

ifFrameLoaded(n + 2) {
   trace("/// frame " + (n + 2) + " marked as loaded");
}


tellTarget(target) {
   ifFrameLoaded(0) {
      trace("/// target frame 0 marked as loaded");
   }

   ifFrameLoaded(-43) {
      trace("/// target frame -43 marked as loaded");
   }

   ifFrameLoaded(Infinity) {
      trace("/// target frame Infinity marked as loaded");
   }
}
