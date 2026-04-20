/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION="";
// var VERSION = "AS3";



// check for subclassing of Array, both with and without "dynamic"

dynamic class MyDynamicArray extends Array              // dynamic child
{
}

class MyNonDynamicArray extends Array                   // nondynamic child (dynamic isn't inherited in AS3)
{
}

dynamic class MyDynamicArray2 extends MyDynamicArray    // dynamic grandchild
{
}

class MyNonDynamicArray2 extends MyDynamicArray         // nondynamic grandchild
{
}

// Validate behaviour of "holes" in an array
function testArray(a:Array)
{
    var len = 100;
    a[0] = 0;
    for (var i:int = 1; i < len; ++i)
    {
        // test getprop & setprop (and .length)
        a[i] = a[i-1]+1;
    }

    // test delprop
    delete a[20];

    var tot = 0;
    for (var i:int = 1; i < a.length; ++i)
    {
        // test hasprop (should skip 20)
        if (a.hasOwnProperty(i))
            tot += a[i];
    }
    // print("tot "+tot);
    return tot;
}

Assert.expectEq("Validate behaviour of holes in Array",
            4930,
            testArray(new Array())
            );

Assert.expectEq("Validate behaviour of holes in subclassed dynamic Array",
            4930,
            testArray(new MyDynamicArray())
            );

Assert.expectEq("Validate behaviour of holes in subclassed grandchild of dynamic Array",
            4930,
            testArray(new MyDynamicArray2())
            );

var err = "no error";
try {
    testArray(new MyNonDynamicArray()); // expect ReferenceError: Error #1056
} catch (e) {
    err = Utils.grabError(e, e.toString());
} finally {
    Assert.expectEq("Validate behaviour of holes in subclassed non-dynamic Array",
                "Error #1056",
                err );
}

err = "no error";
try {
    testArray(new MyNonDynamicArray2());    // expect ReferenceError: Error #1056
} catch (e) {
    err = Utils.grabError(e, e.toString());
} finally {
    Assert.expectEq("Validate behaviour of holes in subclassed grandchild of non-dynamic Array",
                "Error #1056",
                err );
}

// test "construct"

var a = new Array(1,2,3);
Assert.expectEq("test array construct",
            3,
            a.length);

err = "no error";
try {
    var a = new MyDynamicArray(1,2,3);
    var temp = a.length;                            // expect ArgumentError: Error #1063
} catch (e) {
    err = Utils.grabError(e, e.toString());
} finally {
    Assert.expectEq("Test construct of subclassed dynamic Array",
                "Error #1063",
                err );
}

err = "no error";
try {
    var a = new MyDynamicArray2(1,2,3);
    var temp = a.length;                            // expect ArgumentError: Error #1063
} catch (e) {
    err = Utils.grabError(e, e.toString());
} finally {
    Assert.expectEq("Test construct of grandchild of subclassed dynamic Array",
                "Error #1063",
                err );
}

err = "no error";
try {
    var a = new MyNonDynamicArray(1,2,3);
    var temp = a.length;                            // expect ArgumentError: Error #1063
} catch (e) {
    err = Utils.grabError(e, e.toString());
} finally {
    Assert.expectEq("Test construct in subclassed non-dynamic Array",
                "Error #1063",
                err );
}

err = "no error";
try {
    var a = new MyNonDynamicArray2(1,2,3);
    var temp = a.length;                            // expect ArgumentError: Error #1063
} catch (e) {
    err = Utils.grabError(e, e.toString());
} finally {
    Assert.expectEq("Test construct in grandchild of subclassed non-dynamic Array",
                "Error #1063",
                err );
}

// test "call"
var a = Array(1,2,3);
Assert.expectEq("Test array call", 3, a.length);

err = "no error";
try {
    var a = MyDynamicArray(1,2,3);
    var temp = a.length;                            // expect ArgumentError: Error #1112
} catch (e) {
    err = Utils.grabError(e, e.toString());
} finally {
    Assert.expectEq("Test call in subclassed dynamic Array",
                "Error #1112",
                err );
}

err = "no error";
try {
    var a = MyDynamicArray2(1,2,3);
    var temp = a.length;                            // expect ArgumentError: Error #1112
} catch (e) {
    err = Utils.grabError(e, e.toString());
} finally {
    Assert.expectEq("Test call in grandchild of subclassed dynamic Array",
                "Error #1112",
                err );
}

err = "no error";
try {
    var a = MyNonDynamicArray(1,2,3);
    var temp = a.length;                            // expect ArgumentError: Error #1112
} catch (e) {
    err = Utils.grabError(e, e.toString());
} finally {
    Assert.expectEq("Test call in subclassed non-dynamic Array",
                "Error #1112",
                err );
}

err = "no error";
try {
    var a = MyNonDynamicArray2(1,2,3);
    var temp = a.length;                            // expect ArgumentError: Error #1112
} catch (e) {
    err = Utils.grabError(e, e.toString());
} finally {
    Assert.expectEq("Test call in grandchild of subclassed non-dynamic Array",
                "Error #1112",
                err );
}


// test holes vs elements-with-value-of-undefined
var a = [undefined];
for (var e in a) {
    Assert.expectEq("test holes vs elements-with-value-of-undefined: for-var-in",
                0,
                e);
}
for each (var e in a) {
    Assert.expectEq("test holes vs elements-with-value-of-undefined: for-each-var-in",
                undefined,
                e);
}


// Next two testcases: for loops will not loop over delete elements
var a = [1];
delete a[0];
var x=0;
for (var e in a) {
    x++;
}
Assert.expectEq("test holes vs elements-with-value-of-undefined - deleted element: for-var-in",
            0,
            x);

x=0;
for each (var e in a) {
    x++
}
Assert.expectEq("test holes vs elements-with-value-of-undefined - deleted element: for-each-var-in",
            0,
            x);


