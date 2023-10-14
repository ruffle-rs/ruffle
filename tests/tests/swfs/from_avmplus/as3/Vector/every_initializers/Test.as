/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
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
Assert.expectEq(    "every empty Vector",
  true,
  new <int>[].every(checker1));

var msg="";
Assert.expectEq(    "every small Vector returns true",
  true,
  new <int>[1,2,3].every(checker1));

Assert.expectEq(    "every small array check function",
  "checker1(1,0,[1,2,3])checker1(2,1,[1,2,3])checker1(3,2,[1,2,3])",
  msg);

var msg="";
Assert.expectEq(    "every small array returns false on 0",
  false,
  new <int>[2,1,0].every(checker1));

var msg="";
var thisobj=new Object();
thisobj.message="object";
new <int>[1,2,3,4,5,].every(checker3,thisobj);
Assert.expectEq(    "every small array with a specified this object",
  "objectobjectobjectobjectobject",
  msg);
