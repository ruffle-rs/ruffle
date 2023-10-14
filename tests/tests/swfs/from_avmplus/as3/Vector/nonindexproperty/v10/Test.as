/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

/**
 File Name:    nonindexproperty.es
 Description:  Vector properties not uint will throw a runtime error.
 the exception is a current issue with properties defined in the prototype.
 *
 * Coverage is rather poor, and this test has been superseded by vectorIndexRangeExceptions.as for
 * SWF version 11 and above.  It is retained to preserve existing tests applicable to prior versions.
 */
import flash.system.*;
import avmplus.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;
// TODO: REVIEW AS4 CONVERSION ISSUE
// var SECTION = " ";
// var VERSION = "AS3";

Vector.<*>.prototype[3.14]="three";
var v1:Vector.<*>=new Vector.<*>();
var playerType:String = Capabilities.playerType;

v1[0]="zero";
v1["1"]="one";
v1[2.0]="two";
v1["3.0"]="three";

Assert.expectEq(    "standard 0 uint index",
  "zero",
  v1[0]);
Assert.expectEq(    "uint 1 as string index",
  "one",
  v1[1]);
Assert.expectEq(    "number 3.0 as string index",
  "two",
  v1[2]);
Assert.expectEq(    "number 2.0 index",
  "three",
  v1["3.0"]);


var NONE      = "exception not thrown";
var RANGE     = "RangeError: Error #1125";
var REFREAD   = "ReferenceError: Error #1069";
var REFWRITE  = "ReferenceError: Error #1056";

// Index is generic (Atom).

function AddVectorReadExceptionTest(description, index, expected)
{
  var err = "exception not thrown";
  try {
    v1[index] == description;  // read
  } catch(e) {
    err = e.toString();
  }

  Assert.expectEq("read index " + description + " throws exception because non-uint property",
    expected,
    Utils.parseError(err, expected.length));
}

function AddVectorWriteExceptionTest(description, index, expected)
{
  var err = "exception not thrown";
  try {
    v1[index] = description;  // write
  } catch(e) {
    err = e.toString();
  }

  Assert.expectEq("write index " + description + " throws exception because non-uint property",
    expected,
    Utils.parseError(err, expected.length));
}

// Index specialized to Number.

function AddVectorReadExceptionTest_D(description, index, expected)
{
  var err = "exception not thrown";
  var idx:Number = index;
  try {
    v1[idx] == description;  // read
  } catch(e) {
    err = e.toString();
  }

  Assert.expectEq("read index " + description + " throws exception because non-uint property",
    expected,
    Utils.parseError(err, expected.length));
}

function AddVectorWriteExceptionTest_D(description, index, expected)
{
  var err = "exception not thrown";
  var idx:Number = index;
  try {
    v1[idx] = description;  // write
  } catch(e) {
    err = e.toString();
  }

  Assert.expectEq("write index " + description + " throws exception because non-uint property",
    expected,
    Utils.parseError(err, expected.length));
}

// Index specialized to int.

function AddVectorReadExceptionTest_I(description, index, expected)
{
  var err = "exception not thrown";
  var idx:int = index;
  try {
    v1[idx] == description;  // read
  } catch(e) {
    err = e.toString();
  }

  Assert.expectEq("read index " + description + " throws exception because non-uint property",
    expected,
    Utils.parseError(err, expected.length));
}

function AddVectorWriteExceptionTest_I(description, index, expected)
{
  var err = "exception not thrown";
  var idx:int = index;
  try {
    v1[idx] = description;  // write
  } catch(e) {
    err = e.toString();
  }

  Assert.expectEq("write index " + description + " throws exception because non-uint property",
    expected,
    Utils.parseError(err, expected.length));
}

// 5.1
AddVectorReadExceptionTest    ("5.1", 5.1, REFREAD);
AddVectorWriteExceptionTest   ("5.1", 5.1, REFWRITE);
AddVectorReadExceptionTest_D  ("5.1", 5.1, REFREAD);
AddVectorWriteExceptionTest_D ("5.1", 5.1, REFWRITE);

// "5.1"
AddVectorReadExceptionTest  ("'5.1'", "5.1", REFREAD);
AddVectorWriteExceptionTest ("'5.1'", "5.1", REFWRITE);

// -5.1
AddVectorReadExceptionTest    ("-5.1", -5.1, REFREAD);
AddVectorWriteExceptionTest   ("-5.1", -5.1, REFWRITE);
AddVectorReadExceptionTest_D  ("-5.1", -5.1, REFREAD);
AddVectorWriteExceptionTest_D ("-5.1", -5.1, REFWRITE);

// "-5.1"
AddVectorReadExceptionTest  ("'-5.1'", "-5.1", REFREAD);
AddVectorWriteExceptionTest ("'-5.1'", "-5.1", REFWRITE);

// -6
AddVectorReadExceptionTest    ("-6", -6, REFREAD);
AddVectorWriteExceptionTest   ("-6", -6, REFWRITE);
if (playerType == 'AVMPlus') {
  if (System.getRunmode().indexOf('interp') != -1) {
    // Forced interpretation.
    AddVectorReadExceptionTest_I  ("-6", -6, REFREAD);
    AddVectorWriteExceptionTest_I ("-6", -6, REFWRITE);
  } else {
    // Compiled by default.  This is JIT behavior.
    AddVectorReadExceptionTest_I  ("-6", -6, RANGE);
    AddVectorWriteExceptionTest_I ("-6", -6, RANGE);
  }
}

// -6.0
AddVectorReadExceptionTest    ("-6.0", -6.0, REFREAD);
AddVectorWriteExceptionTest   ("-6.0", -6.0, REFWRITE);
AddVectorReadExceptionTest_D  ("-6.0", -6.0, REFREAD);
AddVectorWriteExceptionTest_D ("-6.0", -6.0, REFWRITE);

// "-6"
AddVectorReadExceptionTest  ("'-6'", "-6", REFREAD);
AddVectorWriteExceptionTest ("'-6'", "-6", REFWRITE);

// "-6.0"
AddVectorReadExceptionTest  ("'-6.0'", "-6.0", REFREAD);
AddVectorWriteExceptionTest ("'-6.0'", "-6.0", REFWRITE);

// "foo"
AddVectorReadExceptionTest  ("'foo'", "foo", REFREAD);
AddVectorWriteExceptionTest ("'foo'", "foo", REFWRITE);

// Odd test case for a known issue.  Note that the property 3.14 is defined in the prototype.

var err1 = "exception not thrown";
try {
  v1["3.14"]=="seven";  // reference property defined in prototype
} catch(e) {
  err1 = e.toString();
}

Assert.expectEq("when Vector.<*>.prototype[3.14] is set throws exception because non-uint property",
  REFREAD,
  Utils.parseError(err1, REFREAD.length));

// Check high and low extremes.

var max_int28_p1 =    268435456;
var max_int28    =    268435455;
var max_int28_m1 =    268435454;

var max_uint_p1  =   4294967296;
var max_uint     =   4294967295;
var max_uint_m1  =   4294967294;

var max_int_p1   =   2147483648;
var max_int      =   2147483647;
var max_int_m1   =   2147483646;

var min_int_p1   =  -2147483647;
var min_int      =  -2147483648;
var min_int_m1   =  -2147483649;

// Implementation limits prevent us from actually allocating a vector as large
// as these sizes, so all of these references will be to undefined properties.

AddVectorReadExceptionTest  ("max_uint_p1", max_uint_p1,   REFREAD);
AddVectorWriteExceptionTest ("max_uint_p1", max_uint_p1,   REFWRITE);
if (playerType == 'AVMPlus') {
  if (System.getRunmode().indexOf('interp') != -1) {
    // Forced interpretation.
    AddVectorReadExceptionTest  ("max_uint",    max_uint,  REFREAD);
    AddVectorWriteExceptionTest ("max_uint",    max_uint,  REFWRITE);
  } else {
    // Compiled by default.  This is JIT behavior.
    AddVectorReadExceptionTest  ("max_uint",    max_uint,  REFREAD);
    AddVectorWriteExceptionTest ("max_uint",    max_uint,  RANGE);
  }
}

AddVectorReadExceptionTest  ("max_uint_m1", max_uint_m1,   RANGE);
AddVectorWriteExceptionTest ("max_uint_m1", max_uint_m1,   RANGE);
AddVectorReadExceptionTest  ("max_int_p1",  max_int_p1,    RANGE);
AddVectorWriteExceptionTest ("max_int_p1",  max_int_p1,    RANGE);
AddVectorReadExceptionTest  ("max_int",     max_int,       RANGE);
AddVectorWriteExceptionTest ("max_int",     max_int,       RANGE);
AddVectorReadExceptionTest  ("max_int_m1",  max_int_m1,    RANGE);
AddVectorWriteExceptionTest ("max_int_m1",  max_int_m1,    RANGE);

// The implementation does some case analysis at 28 bits.

AddVectorReadExceptionTest  ("max_int28_p1", max_int28_p1, RANGE);
AddVectorWriteExceptionTest ("max_int28_p1", max_int28_p1, RANGE);
AddVectorReadExceptionTest  ("max_int28",    max_int28,    RANGE);
AddVectorWriteExceptionTest ("max_int28",    max_int28,    RANGE);
AddVectorReadExceptionTest  ("max_int28_m1", max_int28_m1, RANGE);
AddVectorWriteExceptionTest ("max_int28_m1", max_int28_m1, RANGE);

AddVectorReadExceptionTest  ("min_int_p1",  min_int_p1,    REFREAD);   // Negative index not allowed.
AddVectorWriteExceptionTest ("min_int_p1",  min_int_p1,    REFWRITE);  // Negative index not allowed.
AddVectorReadExceptionTest  ("min_int",     min_int,       REFREAD);   // Negative index not allowed.
AddVectorWriteExceptionTest ("min_int",     min_int,       REFWRITE);  // Negative index not allowed.
AddVectorReadExceptionTest  ("min_int_m1",  min_int_m1,    REFREAD);   // Negative index not allowed.
AddVectorWriteExceptionTest ("min_int_m1",  min_int_m1,    REFWRITE);  // Negative index not allowed.


// restore prototype properties
delete Vector.<*>.prototype[3.14];
