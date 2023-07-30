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
 File Name:    every.es
 Description:  every(object,checker,thisObj=)
 calls checker on every Vector element of object in increasing numerical index order, stopping
 as soon as any call returns false.
 checker is called with three arguments: the property value, the property index
 and the object itself.  The thisobj is used as the this object in the call.
 returns true if all the calls to checker returned true values, otherwise it returns false.
 *
 */
// var SECTION="";
// var VERSION = "ECMA_1";



function checker1(value,index,obj):Boolean {
    msg+="checker1("+value+","+index+",["+obj+"])";
    if (value==0)
        return false;
    return true;
}
function checker3(value,index,obj):Boolean {
    msg+=this.message;
    return true;
}

var msg="";
var v1=new Vector.<int>();
Assert.expectEq(    "every empty array",
  true,
  v1.every(checker1));

var msg="";
var v1=new Vector.<int>();
for (var i=0;i<3;i++) v1.push(i+1);
Assert.expectEq(    "every small array returns true",
  true,
  v1.every(checker1));

Assert.expectEq(    "every small array check function",
  "checker1(1,0,[1,2,3])checker1(2,1,[1,2,3])checker1(3,2,[1,2,3])",
  msg);

var msg="";
var v1=new Vector.<int>();
for (var i=0;i<3;i++) v1.push(2-i);
Assert.expectEq(    "every small array returns false on 0",
  false,
  v1.every(checker1));

var v1 = Vector.<Number>([3.1415, Number.MAX_VALUE, -0.00032]);
Assert.expectEq("every: Number vector does not contain a zero",
  true,
  v1.every(checker1));

var v1 = Vector.<Number>([3.1415, Number.MAX_VALUE, 0.00, -0.00032]);
Assert.expectEq("every: Number vector does contain a zero",
  false,
  v1.every(checker1));

var v1 = Vector.<uint>([31415, uint.MAX_VALUE, 999999]);
Assert.expectEq("every: uint vector does not contain a zero",
  true,
  v1.every(checker1));

var v1 = Vector.<uint>([31415, uint.MAX_VALUE,0, 999999]);
Assert.expectEq("every: uint vector does not contain a zero",
  false,
  v1.every(checker1));

var msg="";
var thisobj=new Object();
thisobj.message="object";
var v1=new Vector.<int>(5);
v1.every(checker3,thisobj);
Assert.expectEq(    "every small array with a specified this object",
  "objectobjectobjectobjectobject",
  msg);

// Custom vector class
class TestClass {
    private var myVal:Object;
    public function TestClass(v:Object):void {
        myVal = v;
    }
    public function toString():String {
        return myVal.toString();
    }

    public function doubleMyVar():void {
        myVal *= 2;
    }

    public static function double(item:Object, index:int, vector:Vector.<TestClass>):Object {
        item.doubleMyVar();
        return item;
    }

    public static function lessThan100(obj:TestClass):Boolean {
        if (obj.myVal < 100) {
            return true;
        } else {
            return false;
        }
    }
}

function thisObjectTest(item:Object, index:int, vector:Vector.<TestClass>):Object {
    return this.lessThan100(item);
}

var v4:Vector.<TestClass> = new Vector.<TestClass>();
v4.push(new TestClass(33));
v4.push(new TestClass(44));
v4.push(new TestClass(1));
v4.push(new TestClass(50));

Assert.expectEq("thisObject test 1",
  true,
  v4.every(thisObjectTest, TestClass)
);

v4.push(new TestClass(500));

Assert.expectEq("thisObject test 2",
  false,
  v4.every(thisObjectTest, TestClass)
);

var errorMsg = "";
try {
    v4.every(thisObjectTest, undefined);
} catch (e) {
    errorMsg = e.toString();
}

Assert.expectEq("thisObject test 3",
  "TypeError: Error #1006",
  Utils.parseError(errorMsg,"TypeError: Error #1006".length)
);

errorMsg = "";
try {
    v4.every(thisObjectTest, NaN);
} catch (e) {
    errorMsg = e.toString();
}

Assert.expectEq("thisObject test 4",
  "TypeError: Error #1006",
  Utils.parseError(errorMsg,"TypeError: Error #1006".length)
);

errorMsg = "";
try {
    v4.every(thisObjectTest, false);
} catch (e) {
    errorMsg = e.toString();
}

Assert.expectEq("thisObject test 5",
  "TypeError: Error #1006",
  Utils.parseError(errorMsg,"TypeError: Error #1006".length)
);
