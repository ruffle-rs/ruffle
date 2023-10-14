/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

/**
 *   File Name:    insertremove.as
 *   Description:  Test Vector.insertAt() and Vector.removeAt().  These are AS3 extensions defined in terms of AS3 Vector.splice().
 */

// var SECTION="";
// var VERSION = "ECMA_1";

var kNoError = "**OK**";
var kTypeError = "**TypeError 1034**";
var kReferenceError = "**ReferenceError 1065**";

function CallInsertAt(vec, index, element)
{
  var status = "**OK**";
  try {
    vec.insertAt(index, element);
  }
  catch (e: RangeError) {
    status = "**RangeError " + e.errorID + "**";
  }
  catch (e: TypeError) {
    status = "**TypeError " + e.errorID + "**";
  }
  return status;
}

function CheckInsertAt(vec, index, element, fixed=false)
{
  var description = "insert into " + vec.length + "-element vector at index " + index;

  var vec1 = vec.concat(); // shallow copy
  vec1.fixed = fixed;

  var status1 = "**OK**";
  try {
    vec1.splice(index, 0, element);
  }
  catch (e: RangeError) {
    status1 = "**RangeError " + e.errorID + "**";
  }
  catch (e: TypeError) {
    status1 = "**TypeError " + e.errorID + "**";
  }

  Assert.expectEq(description + ": length after splice",	 vec1.length, vec.length+1);

  var vec2 = vec.concat(); // shallow copy
  vec2.fixed = fixed;

  var status2 = CallInsertAt(vec2, index, element);

  Assert.expectEq(description + ": length after insertAt", vec2.length, vec.length+1);

  Assert.expectEq(description + ": status", status1, status2);

  for (var i = 0; i < vec.length+1; i++)
  {
    Assert.expectEq(description + ": element " + i, vec1[i], vec2[i]);
  }
}

function CheckRemoveAt(vec, index, fixed=false)
{
  var description = "remove from " + vec.length + "-element vector at index " + index;

  var vec1 = vec.concat(); // shallow copy
  vec1.fixed = fixed;

  var element1 = "**OtherError**";
  try {
    element1 = vec1.splice(index, 1)[0];
  }
  catch (e: RangeError) {
    element1 = "**RangeError " + e.errorID + "**";
  }

  var vec2 = vec.concat(); // shallow copy
  vec2.fixed = fixed;

  var element2 = "**OtherError**";
  try {
    element2 = vec2.removeAt(index);
  }
  catch (e: RangeError) {
    element2 = "**RangeError " + e.errorID + "**";
  }

  Assert.expectEq(description + ": length", vec1.length, vec2.length);

  Assert.expectEq(description + ": result", element1, element2);

  var count = vec1.length-1;
  if (count > vec2.length-1) count = vec2.length-1;

  for (var i = 0; i < count; i++)
  {
    Assert.expectEq(description + ": element " + i, vec1[i], vec2[i]);
  }
}

function CheckVector(vec)
{
  var index = 0;

  CheckInsertAt(vec, index, "foo");
  CheckRemoveAt(vec, index);

  var last = vec.length + 2;
  for (var i = 1; i <= last; i++)
  {
    index = i;

    CheckInsertAt(vec, index, "foo");
    CheckRemoveAt(vec, index);

    index = -i;

    CheckInsertAt(vec, index, "foo");
    CheckRemoveAt(vec, index);
  }
}

// Untraced Vector.<T>.
// The scalar type T used as the argument is not essential, so we choose "int" as representative.

var u_vec0 = new Vector.<int>();
CheckVector(u_vec0);

var u_vec1 = new <int>[1];
CheckVector(u_vec1);

var u_vec2 = new <int>[1, 2];
CheckVector(u_vec2);

var u_vec3 = new <int>[1, 2, 3];
CheckVector(u_vec3);


// Traced Vector.<T>.
// The object type T used as the argument is not essential, so we choose "String" as representative.

var t_vec0 = new Vector.<String>();
CheckVector(t_vec0);

var t_vec1 = new <String>["one"];
CheckVector(t_vec1);

var t_vec2 = new <String>["one", "two"];
CheckVector(t_vec2);

var t_vec3 = new <String>["one", "two", "three"];
CheckVector(t_vec3);


// Verify that we coerce the inserted value to the element type.

class MyClass {};
class MyOtherClass {};

var c_vec0 = new Vector.<int>(1);
Assert.expectEq("insert String coercible to int into Vector.<int>", kNoError, CallInsertAt(c_vec0, 0, "555"));
Assert.expectEq("verify inserted int value coerced from String", 555, c_vec0[0]);

var c_vec1 = new Vector.<*>(1);
Assert.expectEq("insert String into Vector.<*>", kNoError, CallInsertAt(c_vec1, 0, "foo"));  // String is coercible to *
Assert.expectEq("verify inserted * value coerced from String", "foo", c_vec1[0]);

var c_vec2 = new Vector.<Object>(1);
Assert.expectEq("insert String into Vector.<Object>", kNoError, CallInsertAt(c_vec2, 0, "foo"));  // String is coercible to Object
Assert.expectEq("verify inserted Object value coerced from String", "foo", c_vec2[0]);

var c_vec3 = new Vector.<MyClass>(1);
Assert.expectEq("insert String into Vector.<MyClass>", kTypeError, CallInsertAt(c_vec3, 0, "foo"));	// String is not coercible to MyClass

var c_vec4 = new Vector.<MyClass>(1);
Assert.expectEq("insert MyOtherClass into Vector.<MyClass>", kTypeError, CallInsertAt(c_vec4, 0, new MyOtherClass()));	// MyOtherClass is not coercible to MyClass
