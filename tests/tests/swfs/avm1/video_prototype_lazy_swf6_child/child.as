// See test.as.
function desc(o, name)
{
   return !o.hasOwnProperty(name) ? "-" : typeof o[name];
}
function show(o)
{
   return "width=" + desc(o,"width") + " height=" + desc(o,"height");
}
if(this.started == undefined)
{
   this.started = true;
   stop();
   var P = _root.P;
   trace("child: same Video class as parent: " + (Video == _root.parentVideo));
   Video.prototype = P;
   this.gotoAndStop(2);
   trace("child placed Video on P: " + show(P) + ", v.__proto__ == P: " + (this.v2.__proto__ == P));
   var R = {};
   Video.prototype = R;
   this.gotoAndStop(1);
   this.gotoAndStop(2);
   trace("child placed Video on R: " + show(R) + ", v.__proto__ == R: " + (this.v2.__proto__ == R));
   delete R.width;
   trace("child deleted R.width: " + show(R));
   _root.R = R;
   _root.childDone = true;
}
