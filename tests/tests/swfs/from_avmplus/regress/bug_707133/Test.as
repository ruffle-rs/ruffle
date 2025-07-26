/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

// var SECTION = "regress_707133";
// var VERSION = "AS3";
// var TITLE   = "adjusting arrays length should not introduce phantom elements";
// var bug = "707133";


function reducedTestCase()
{
    var a = new Array();
    a.length = 1; // or any semi-small K1; leaves m_denseArray at 0 elems
    a.length = 2; // or any semi-small K2+K1; oldLength = K1, and inserts
                  //   only K2 atomNotFound's instead of K2+K1 such entries.

    var keys = [];
    for (var keyObj:Object in a)
        keys.push(keyObj);
    keys.sort();

    var output="";
    for (var i=0; i < keys.length; i++)
    {
        var val:Object = a[keys[i]];
        output += "<" + keys[i] + ", " + val +">; ";
    }
    return output;
}

function originalTestCase()
{
    var ar:Array = new Array();

    for(var i:int = 0; i < 4; i++)
    {
        var key:String = "keyname_" + i;
        var value:Object = new Object();
        ar[key] = value;
        ar.length++;
    }

    // normalize output by making list of keys and sorting it.
    var keys = [];
    for (var keyObj:Object in ar)
        keys.push(keyObj);
    keys.sort();

    var output = "";
    for (var i=0; i < keys.length; i++)
    {
        var val:Object = ar[keys[i]];
        output += "<" + keys[i] + ", " + val +">; ";
    }
    return output;
}

Assert.expectEq("reduced test case", "", reducedTestCase());
Assert.expectEq("original test case",
            ("<keyname_0, [object Object]>; "+
             "<keyname_1, [object Object]>; "+
             "<keyname_2, [object Object]>; "+
             "<keyname_3, [object Object]>; "),
            originalTestCase());

