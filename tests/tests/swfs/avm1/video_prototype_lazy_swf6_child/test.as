// Flash Player defines width and height on the prototype of a Video display
// object when it is created, at most once per prototype object. This tests
// whether that is tracked separately for SWF 6 and SWF 8 movies, which have
// separate `_global` objects (and Video classes).
//
// test.swf (SWF 8) and child.swf (SWF 6), compiled from their .swfasm
// files, both place a Video as `v2` on frame 2. Going to frame 1 and back to
// frame 2 creates a new Video.
function desc(o, name)
{
   return !o.hasOwnProperty(name) ? "-" : typeof o[name];
}
function has(o)
{
   return "width=" + desc(o,"width") + " height=" + desc(o,"height");
}
if(this.started == undefined)
{
   this.started = true;
   stop();
   _root.parentVideo = Video;
   this.onEnterFrame = function()
   {
      if(this.phase == undefined)
      {
         this.phase = 1;
         var P = {};
         _root.P = P;
         Video.prototype = P;
         this.gotoAndStop(1);
         this.gotoAndStop(2);
         trace("parent placed Video on P: " + has(P) + ", v.__proto__ == P: " + (this.v2.__proto__ == P));
         delete P.width;
         trace("parent deleted P.width: " + has(P));
         this.createEmptyMovieClip("holder",10);
         this.holder.loadMovie("child.swf");
      }
      else if(this.phase == 1 && _root.childDone)
      {
         this.phase = 2;
         var R = _root.R;
         Video.prototype = R;
         this.gotoAndStop(1);
         this.gotoAndStop(2);
         trace("parent placed Video on R: " + has(R) + ", v.__proto__ == R: " + (this.v2.__proto__ == R));
         trace("done");
         delete this.onEnterFrame;
      }
   };
}
