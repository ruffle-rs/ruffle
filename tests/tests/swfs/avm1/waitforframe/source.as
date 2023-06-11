// All `ifFrameLoaded` must compile to `ActionWaitForFrame`.

ifFrameLoaded(0) {
   trace("/// frame 0 marked as loaded");
}

ifFrameLoaded(1) {
   trace("/// frame 1 marked as loaded");
}

ifFrameLoaded(2) {
   trace("/// frame 2 marked as loaded");
}

ifFrameLoaded(16000) {
   trace("/// frame 16000 marked as loaded");
}

ifFrameLoaded(16001) {
   trace("/// frame 16001 marked as loaded"); // won't trace
}

ifFrameLoaded(16002) {
   trace("/// frame 16002 marked as loaded"); // won't trace
}

tellTarget(target) {
   ifFrameLoaded(0) {
      trace("/// target frame 0 marked as loaded");
   }

   ifFrameLoaded(1) {
      trace("/// target frame 1 marked as loaded");
   }

   ifFrameLoaded(2) {
      trace("/// target frame 2 marked as loaded");
   }
}
