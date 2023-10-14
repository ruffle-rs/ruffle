/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
/**
 File Name:    filter.es
 Description:  filter(object,checker,thisobj)
 calls checker on every vector element of object in increasing numerical index order,
 collecting all the vector elements for which checker returns a true value.
 checker is called with three arguments: the property value, the property index, and object
 itself. The thisobj is used as the this object in the call.
 returns a new vector object containing the elements that were collected in the order
 they were collected.
 */
// var SECTION="";
// var VERSION = "ECMA_1";



function EvenChecker(value,index,obj) {
    if (value%2==0)
        return true;
    return false;
}
var invalidchecker="a string";
function ThisChecker(value,index,obj):Boolean {
    msg+=this.message;
    return true;
}

var v1=new Vector.<int>();
var errormsg="";
try {
    var result=v1.filter();
} catch (e) {
    errormsg=e.toString();
}
Assert.expectEq(    "filter checker is undefined",
  "ArgumentError: Error #1063",
  Utils.parseError(errormsg,"ArgumentError: Error #1063".length));

var v1:Vector.<int>=new Vector.<int>(10);
for (var i=0;i<10;i++) v1[i]=i;
var errormsg="";
try {
    var result=v1.filter(invalidchecker);
} catch (e) {
    errormsg=e.toString();
}
Assert.expectEq(    "filter checker is not a function",
  "TypeError: Error #1034",
  Utils.parseError(errormsg,"TypeError: Error #1034".length));

var v1:Vector.<int>=new Vector.<int>();
var result=v1.filter(EvenChecker);
Assert.expectEq(    "filter empty vector",
  "",
  result.toString());

var v1:Vector.<int>=new Vector.<int>();
for (var i=0;i<10;i++) v1[i]=i;
var result=v1.filter(EvenChecker);
Assert.expectEq(    "filter small vector",
  "0,2,4,6,8",
  result.toString());

var vn:Vector.<Number>=new Vector.<Number>();
for (var i=0;i<10;i++) vn[i]=i;
var result=vn.filter(EvenChecker);
Assert.expectEq("filter small Number vector",
  "0,2,4,6,8",
  result.toString());

var vu:Vector.<uint>=new Vector.<uint>();
for (var i=0;i<10;i++) vu[i]=i;
var result=vu.filter(EvenChecker);
Assert.expectEq(    "filter small vector",
  "0,2,4,6,8",
  result.toString());

var v1:Vector.<int>=new Vector.<int>();
for (var i=0;i<3;i++) v1[i]=i;
var myobject=new Object();
myobject.message="message";
var msg="";
var result=v1.filter(ThisChecker,myobject);
Assert.expectEq(    "filter use thisobj",
  "messagemessagemessage",
  msg);

// Bugzilla https://bugzilla.mozilla.org/show_bug.cgi?id=513095
var items:Vector.<String> = new Vector.<String>;
items.push("one");
items.push("two");
items.push("three");

var filtered:Vector.<String> = items.filter(function(item:String, index:int,
                                                     source:Vector.<String>):Boolean
{
    return item == "two";
});
Assert.expectEq("Bug 513095: Type-check filter function",
  "two",
  filtered.toString()
);

class TestClass {
    private var myVal:Object;

    static public function over100(item:TestClass, index:int, vector:Vector.<TestClass>):Boolean {
        if (item.myVal > 100)
            return true;
        return false;
    }

    public function TestClass(v:Object):void {
        myVal = v;
    }

    public function toString():String {
        return myVal.toString();
    }


}

var v2 = new <TestClass> [new TestClass(150), new TestClass(40), new TestClass(-200), new TestClass(400)];
var v2filtered = v2.filter(TestClass.over100);

Assert.expectEq("Filtered custom class",
  "150,400",
  v2filtered.toString()
);
