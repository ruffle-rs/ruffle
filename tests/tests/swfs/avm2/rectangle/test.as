package {

	public class test {
	}
}

import flash.geom.Rectangle;
import flash.geom.Point;

function dump(rect, desc)
{
   var _loc1_ = "(";
   _loc1_ += "top=" + rect.top + ", ";
   _loc1_ += "right=" + rect.right + ", ";
   _loc1_ += "bottom=" + rect.bottom + ", ";
   _loc1_ += "left=" + rect.left + ", ";
   _loc1_ += "topLeft=" + rect.topLeft + ", ";
   _loc1_ += "bottomRight=" + rect.bottomRight + ", ";
   _loc1_ += "width=" + rect.width + ", ";
   _loc1_ += "height=" + rect.height + ", ";
   _loc1_ += "size=" + rect.size + ", ";
   _loc1_ += "x=" + rect.x + ", ";
   _loc1_ += "y=" + rect.y + ")";
   trace(" // " + desc);
   trace(_loc1_);
   trace("");
}
function setAndDump(rect, key, value)
{
   rect[key] = value;
   dump(rect,"rect." + key + " = " + value);
}
function tryValues(key, values)
{
   trace("");
   trace("/// " + key);
   trace("");
   var _loc2_ = new Rectangle(1,3,5,7);
   dump(_loc2_,"before modifications");
   var _loc1_ = 0;
   while(_loc1_ < values.length)
   {
      _loc2_ = new Rectangle(1,3,5,7);
      setAndDump(_loc2_,key,values[_loc1_]);
      _loc1_ = _loc1_ + 1;
   }
}
function tryMethod(name, argsList, isRect, dumpOrig)
{
   trace("");
   trace("/// " + name);
   trace("");
   var rectList = [new Rectangle(), new Rectangle(1, 3, 5, 7), new Rectangle(-1, -3, 5, 7), new Rectangle(1, 3, -5, 7)];
   var i = 0;
   while(i < rectList.length) {
      var _loc5_ = rectList[i].clone();
      dump(_loc5_,"rect");
      var _loc4_ = 0;
      while(_loc4_ < argsList.length)
      {
         _loc5_ = rectList[i].clone();

         var _loc3_ = argsList[_loc4_];
         var _loc1_ = "";
         var _loc2_ = 0;
         while(_loc2_ < _loc3_.length)
         {
            if(_loc1_.length > 0)
            {
               _loc1_ += ", ";
            }
            _loc1_ += _loc3_[_loc2_];
            _loc2_ = _loc2_ + 1;
         }
         var _loc6_ = _loc5_[name].apply(_loc5_,_loc3_);
         if(isRect)
         {
            dump(_loc6_,"rect." + name + "(" + _loc1_ + ")");
         }
         else
         {
            trace("// rect." + name + "(" + _loc1_ + ")");
            trace(_loc6_);
            trace("");
         }
         if(dumpOrig)
         {
            dump(_loc5_,"rect");
         }
         _loc4_ = _loc4_ + 1;
      }
      i = i + 1;
   }
}
trace("/// Constructor");
trace("");
dump(new Rectangle(),"new Rectangle()");
dump(new Rectangle(1),"new Rectangle(1)");
dump(new Rectangle(1,2),"new Rectangle(1, 2)");
dump(new Rectangle(1,2,3),"new Rectangle(1, 2, 3)");
dump(new Rectangle(1,2,3,4),"new Rectangle(1, 2, 3, 4)");
var numberValues = [0,100,-200,NaN,Infinity];
tryValues("top",numberValues);
tryValues("right",numberValues);
tryValues("left",numberValues);
tryValues("bottom",numberValues);
tryValues("width",numberValues);
tryValues("height",numberValues);
tryValues("x",numberValues);
tryValues("y",numberValues);
var pointValues = [new Point(0,0),new Point(-100,-200),new Point(100,200),new Point(Infinity,Infinity),new Point(NaN,NaN)];
tryValues("topLeft",pointValues);
tryValues("bottomRight",pointValues);
tryValues("size",pointValues);
trace("");
trace("/// clone");
trace("");
var orig = new Rectangle(1,3,5,7);
var cloned = orig.clone();
dump(orig,"orig");
dump(cloned,"cloned");
trace("// orig == cloned");
trace(orig == cloned);
trace("");
trace("// orig.equals(cloned)");
trace(orig.equals(cloned));
trace("");
trace("");
trace("/// copyFrom");
trace("");
var orig = new Rectangle(1,3,5,7);
var other = new Rectangle(2, 1, 3, 7);
dump(orig,"orig");
dump(other,"other");
trace("// other.copyFrom(orig)");
other.copyFrom(orig);
dump(other,"other");
trace("// orig == other");
trace(orig == other);
trace("");
trace("");
trace("/// equals");
trace("");
var orig = new Rectangle(1,3,5,7);
dump(orig,"orig");
trace("// orig.equals(new Rectangle(1, 3, 5, 7))");
trace(orig.equals(new Rectangle(1,3,5,7)));
trace("");
trace("");
trace("/// isEmpty");
trace("");
trace("// new Rectangle().isEmpty()");
trace(new Rectangle().isEmpty());
trace("");
trace("// new Rectangle(0, 0, 0, 0).isEmpty()");
trace(new Rectangle(0,0,0,0).isEmpty());
trace("");
trace("// new Rectangle(1, 2, 3, 0).isEmpty()");
trace(new Rectangle(1,2,3,0).isEmpty());
trace("");
trace("// new Rectangle(1, 2, 0, 4).isEmpty()");
trace(new Rectangle(1,2,0,4).isEmpty());
trace("");
trace("// new Rectangle(1, 2, 3, 4).isEmpty()");
trace(new Rectangle(1,2,3,4).isEmpty());
trace("");
trace("// new Rectangle(1, 2, Infinity, Infinity).isEmpty()");
trace(new Rectangle(1,2,Infinity,Infinity).isEmpty());
trace("");
trace("// new Rectangle(1, 2, NaN, NaN).isEmpty()");
trace(new Rectangle(1,2,NaN,NaN).isEmpty());
trace("");
trace("// new Rectangle(1, 2, undefined, undefined).isEmpty()");
trace(new Rectangle(1,2,undefined,undefined).isEmpty());
trace("");
trace("// new Rectangle(1, 2, -1, -2).isEmpty()");
trace(new Rectangle(1,2,-1,-2).isEmpty());
trace("");
trace("");
trace("/// setEmpty");
trace("");
var orig = new Rectangle(1,3,5,7);
dump(orig,"orig");
trace("// orig.setEmpty()");
trace(orig.setEmpty());
trace("");
dump(orig,"orig");
tryMethod("contains",[[1,2],[1,3],[1.1,3.1],[6,10],[5.9,9.9],[4,NaN],[5,"5"],[5,Infinity]],false,false);
tryMethod("containsPoint",[[new Point()],[new Point(1)],[new Point(1,2)],[new Point(1,3)],[new Point(1.1,3.1)],[new Point(6,10)],[new Point(5.9,9.9)]],false,false);
tryMethod("containsRect",[[new Rectangle(0.9,2.9,5,7)],[new Rectangle(1,3,5.1,7.1)],[new Rectangle(5,5,NaN,1)]],false,false);
tryMethod("inflate",[[1,2],[-3,-4],[Infinity,5],[5,NaN]],false,true);
tryMethod("inflatePoint",[[new Point(1,2)]],false,true);
tryMethod("intersection",[[new Rectangle(3,5,7,9)],[new Rectangle(-1,-3,5,7)],[new Rectangle(30,50,1,1)]],true, false);
tryMethod("intersects",[[new Rectangle(3,5,7,9)],[new Rectangle(-1,-3,5,7)]], false, false);
tryMethod("offset",[[1,2],[-3,-4],[Infinity,5],[NaN,6]],false,true);
tryMethod("offsetPoint",[[new Point(1,2)]],false,true);
tryMethod("union",[[new Rectangle(3,5,7,9)],[new Rectangle(3,5,7,9)],[new Rectangle(-1,-3,NaN,7)]],true, false);
tryMethod("setTo",[[3,5,7,9],[-1,-3,5,7]],false, true);
