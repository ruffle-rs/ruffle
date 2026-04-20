/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Expressions";        // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";                // Version of ECMAScript or ActionScript
// var TITLE   = "delete operator";    // Provide ECMA section title or a description
var BUGNUMBER = "";


var arr:Array = new Array();
arr[0] = "abc"; // arr.length == 1
arr[1] = "def"; // arr.length == 2
arr[2] = "ghi"; // arr.length == 3

// arr[2] is deleted, but Array.length is not changed
Assert.expectEq("arr[2]", "ghi", arr[2]);
Assert.expectEq("delete arr[2] successful", true, delete arr[2]);
Assert.expectEq("arr[2]", undefined, arr[2]);
Assert.expectEq("array length after delete arr[2]", 3, arr.length);
Assert.expectEq("array contents after delete arr[2]", "abc,def,", arr.toString());

Assert.expectEq("delete arr[2] again", true, delete arr[2]);
Assert.expectEq("arr[2]", undefined, arr[2]);

Assert.expectEq("delete non-existent arr[3]", true, delete arr[3]);
Assert.expectEq("arr[3]", undefined, arr[3]);


              // displays results.
