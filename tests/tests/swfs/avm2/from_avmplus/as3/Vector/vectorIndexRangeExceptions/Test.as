/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import avmplus.System;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = " ";
// var VERSION = "AS3";

var REFWRITE_MSG  = "ReferenceError: Error #1056";
var REFREAD_MSG   = "ReferenceError: Error #1069";
var RANGE_MSG     = "RangeError: Error #1125";
var NOERROR_MSG   = "exception not thrown";

var NONE      = 0;
var RANGE     = 1;
var REFERENCE = 2;

Vector.<*>.prototype[3]="quux";
Vector.<*>.prototype[-3]="quux";
Vector.<*>.prototype[3.14]="quux";
Vector.<*>.prototype[-3.14]="quux";
Vector.<*>.prototype["bar"]="baz";
Vector.<int>.prototype[3]="quux";
Vector.<int>.prototype[-3]="quux";
Vector.<int>.prototype[3.14]="quux";
Vector.<int>.prototype[-3.14]="quux";
Vector.<int>.prototype["bar"]="baz";
Vector.<uint>.prototype[3]="quux";
Vector.<uint>.prototype[-3]="quux";
Vector.<uint>.prototype[3.14]="quux";
Vector.<uint>.prototype[-3.14]="quux";
Vector.<uint>.prototype["bar"]="baz";
Vector.<Number>.prototype[3]="quux";
Vector.<Number>.prototype[-3]="quux";
Vector.<Number>.prototype[3.14]="quux";
Vector.<Number>.prototype[-3.14]="quux";
Vector.<Number>.prototype["bar"]="baz";
Vector.<String>.prototype[3]="quux";
Vector.<String>.prototype[-3]="quux";
Vector.<String>.prototype[3.14]="quux";
Vector.<String>.prototype[-3.14]="quux";
Vector.<String>.prototype["bar"]="baz";

var v_a:Vector.<*>      = new Vector.<*>();
var v_i:Vector.<int>    = new Vector.<int>();
var v_u:Vector.<uint>   = new Vector.<uint>();
var v_n:Vector.<Number> = new Vector.<Number>();
var v_s:Vector.<String> = new Vector.<String>();

function Reset()
{
  v_a = new Vector.<*>();
  v_i = new Vector.<int>();
  v_u = new Vector.<uint>();
  v_n = new Vector.<Number>();
  v_s = new Vector.<String>();
}

function RecordResult(mode, element_kind, index, index_type, error, expected)
{
  if (index is String)
    index = "'" + index + "'";

  var errmsg = NOERROR_MSG;
  switch (expected) {
    case RANGE:
      errmsg = RANGE_MSG;
      break;
    case REFERENCE:
      errmsg = (mode == "write") ? REFWRITE_MSG : REFREAD_MSG;
      break;
  }

  var description = mode + " v_" + element_kind + "[" + index + ":" + index_type + "]";

  Assert.expectEq(description, errmsg, Utils.parseError(error, errmsg.length));
}

function Read_A(index, c, expected)
{
  var err = NOERROR_MSG;
  try {
    switch (c) {
      case 'a':
        v_a[index] == null;
        break;
      case 'i':
        v_i[index] == -1;
        break;
      case 'u':
        v_u[index] == 55;
        break;
      case 'n':
        v_n[index] == 2.718;
        break;
      case 's':
        v_s[index] == "foo";
        break;
    }
  } catch (e) {
    err = e.toString();
  }
  RecordResult("read", c, index, "*", err, expected);
}

function Write_A(index, c, expected)
{
  var err = NOERROR_MSG;
  try {
    switch (c) {
      case 'a':
        v_a[index] = null;
        break;
      case 'i':
        v_i[index] = -1;
        break;
      case 'u':
        v_u[index] = 55;
        break;
      case 'n':
        v_n[index] = 2.718;
        break;
      case 's':
        v_s[index] = "foo";
        break;
    }
  } catch (e) {
    err = e.toString();
  }
  RecordResult("write", c, index, "*", err, expected);
}

function Test_A(index, c, expected_r, expected_w)
{
  Read_A(index, c, expected_r);
  Write_A(index, c, expected_w);
  Reset();
}

function Read_I(index:int, c, expected)
{
  var err = NOERROR_MSG;
  try {
    switch (c) {
      case 'a':
        v_a[index] == null;
        break;
      case 'i':
        v_i[index] == -1;
        break;
      case 'u':
        v_u[index] == 55;
        break;
      case 'n':
        v_n[index] == 2.718;
        break;
      case 's':
        v_s[index] == "foo";
        break;
    }
  } catch (e) {
    err = e.toString();
  }
  RecordResult("read", c, index, "int", err, expected);
}

function Write_I(index:int, c, expected)
{
  var err = NOERROR_MSG;
  try {
    switch (c) {
      case 'a':
        v_a[index] = null;
        break;
      case 'i':
        v_i[index] = -1;
        break;
      case 'u':
        v_u[index] = 55;
        break;
      case 'n':
        v_n[index] = 2.718;
        break;
      case 's':
        v_s[index] = "foo";
        break;
    }
  } catch (e) {
    err = e.toString();
  }
  RecordResult("write", c, index, "int", err, expected);
}

function Test_I(index:int, c, expected_r, expected_w)
{
  Read_I(index, c, expected_r);
  Write_I(index, c, expected_w);
  Reset();
}

function Read_U(index:uint, c, expected)
{
  var err = NOERROR_MSG;
  try {
    switch (c) {
      case 'a':
        v_a[index] == null;
        break;
      case 'i':
        v_i[index] == -1;
        break;
      case 'u':
        v_u[index] == 55;
        break;
      case 'n':
        v_n[index] == 2.718;
        break;
      case 's':
        v_s[index] == "foo";
        break;
    }
  } catch (e) {
    err = e.toString();
  }
  RecordResult("read", c, index, "uint", err, expected);
}

function Write_U(index:uint, c, expected)
{
  var err = NOERROR_MSG;
  try {
    switch (c) {
      case 'a':
        v_a[index] = null;
        break;
      case 'i':
        v_i[index] = -1;
        break;
      case 'u':
        v_u[index] = 55;
        break;
      case 'n':
        v_n[index] = 2.718;
        break;
      case 's':
        v_s[index] = "foo";
        break;
    }
  } catch (e) {
    err = e.toString();
  }
  RecordResult("write", c, index, "uint", err, expected);
}

function Test_U(index:uint, c, expected_r, expected_w)
{
  Read_U(index, c, expected_r);
  Write_U(index, c, expected_w);
  Reset();
}

function Read_N(index:Number, c, expected)
{
  var err = NOERROR_MSG;
  try {
    switch (c) {
      case 'a':
        v_a[index] == null;
        break;
      case 'i':
        v_i[index] == -1;
        break;
      case 'u':
        v_u[index] == 55;
        break;
      case 'n':
        v_n[index] == 2.718;
        break;
      case 's':
        v_s[index] == "foo";
        break;
    }
  } catch (e) {
    err = e.toString();
  }
  RecordResult("read", c, index, "Number", err, expected);
}

function Write_N(index:Number, c, expected)
{
  var err = NOERROR_MSG;
  try {
    switch (c) {
      case 'a':
        v_a[index] = null;
        break;
      case 'i':
        v_i[index] = -1;
        break;
      case 'u':
        v_u[index] = 55;
        break;
      case 'n':
        v_n[index] = 2.718;
        break;
      case 's':
        v_s[index] = "foo";
        break;
    }
  } catch (e) {
    err = e.toString();
  }
  RecordResult("write", c, index, "Number", err, expected);
}

function Test_N(index:Number, c, expected_r, expected_w)
{
  Read_N(index, c, expected_r);
  Write_N(index, c, expected_w);
  Reset();
}

function Read_S(index:String, c, expected)
{
  var err = NOERROR_MSG;
  try {
    switch (c) {
      case 'a':
        v_a[index] == null;
        break;
      case 'i':
        v_i[index] == -1;
        break;
      case 'u':
        v_u[index] == 55;
        break;
      case 'n':
        v_n[index] == 2.718;
        break;
      case 's':
        v_s[index] == "foo";
        break;
    }
  } catch (e) {
    err = e.toString();
  }
  RecordResult("read", c, index, "String", err, expected);
}

function Write_S(index:String, c, expected)
{
  var err = NOERROR_MSG;
  try {
    switch (c) {
      case 'a':
        v_a[index] = null;
        break;
      case 'i':
        v_i[index] = -1;
        break;
      case 'u':
        v_u[index] = 55;
        break;
      case 'n':
        v_n[index] = 2.718;
        break;
      case 's':
        v_s[index] = "foo";
        break;
    }
  } catch (e) {
    err = e.toString();
  }
  RecordResult("write", c, index, "String", err, expected);
}

function Test_S(index:String, c, expected_r, expected_w)
{
  Read_S(index, c, expected_r);
  Write_S(index, c, expected_w);
  Reset();
}

function TestFn(index, expected_r, expected_w)
{
  var kinds:Array = ['a', 'i', 'u', 'n', 's'];

  for each (var c:String in kinds) {

    Test_A(index, c, expected_r, expected_w);

    if (index is int) {
      Test_I(index, c, expected_r, expected_w);
    }

    if (index is uint) {
      Test_U(index, c, expected_r, expected_w);
    }

    if (index is Number) {
      Test_N(index, c, expected_r, expected_w);
    }

    if (index is String) {
      Test_S(index, c, expected_r, expected_w);
    } else {
      Test_S(index.toString(), c, expected_r, expected_w);
    }
  }
}

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

// We test behavior only for SWF11+.
// Earlier versions exhibit inconsistent behavior depending on
// whether code is interpreted or compiled, and whether certain
// type specializations have occurred.
// These are well-formed indices, but no element is defined there.
// We forbid the definition of new index properties in any case.

TestFn(        7,      RANGE,              RANGE     );
TestFn(      "7",      RANGE,              RANGE     );
TestFn(      7.0,      RANGE,              RANGE     );
TestFn(    "7.0",      RANGE,              RANGE     );

// Indices that are negative or have a non-zero fractional
// part do not name a vector element, and we do not allow such
// properties to be defined.

TestFn(      5.1,      RANGE,              RANGE     );
TestFn(    "5.1",      RANGE,              RANGE     );
TestFn(     -5.1,      RANGE,              RANGE     );
TestFn(   "-5.1",      RANGE,              RANGE     );

TestFn(       -6,      RANGE,              RANGE     );
TestFn(     "-6",      RANGE,              RANGE     );
TestFn(     -6.0,      RANGE,              RANGE     );
TestFn(   "-6.0",      RANGE,              RANGE     );

// These properties are defined in the Vector prototypes.
// Being index properties, we should not search the prototype chain
// for these properties, and should not discover the definitions.
// We forbid the definition of new index properties in any case.

TestFn(        3,      RANGE,              RANGE     );
TestFn(      "3",      RANGE,              RANGE     );
TestFn(      3.0,      RANGE,              RANGE     );
TestFn(    "3.0",      RANGE,              RANGE     );

TestFn(     3.14,      RANGE,              RANGE     );
TestFn(   "3.14",      RANGE,              RANGE     );

TestFn(       -3,      RANGE,              RANGE     );
TestFn(     "-3",      RANGE,              RANGE     );
TestFn(     -3.0,      RANGE,              RANGE     );
TestFn(   "-3.0",      RANGE,              RANGE     );
TestFn(    -3.14,      RANGE,              RANGE     );
TestFn(  "-3.14",      RANGE,              RANGE     );

// We do not allow definition of new non-index properties.
// Non-index properties may be inherited from the prototype chain.

TestFn(    "foo",      REFERENCE,          REFERENCE );  // "foo" undefined in prototype
TestFn(    "bar",      NONE,               REFERENCE );  // "bar" defined in prototype

// Check high and low extremes.  Properties that could name a vector element,
// should the vector of be sufficient length, must yield RangeError.  It is open
// to question whether other index properties, e.g, negative or fractional, should
// yield RangeError or ReferenceError, but we have decided on the former.

TestFn( max_uint_p1,   RANGE,              RANGE     );
TestFn( max_uint,      RANGE,              RANGE     );
TestFn( max_uint_m1,   RANGE,              RANGE     );

TestFn( max_int_p1,    RANGE,              RANGE     );
TestFn( max_int,       RANGE,              RANGE     );
TestFn( max_int_m1,    RANGE,              RANGE     );

TestFn( max_int28_p1,  RANGE,              RANGE     );
TestFn( max_int28,     RANGE,              RANGE     );
TestFn( max_int28_m1,  RANGE,              RANGE     );

TestFn( min_int_p1,    RANGE,              RANGE     );
TestFn( min_int,       RANGE,              RANGE     );
TestFn( min_int_m1,    RANGE,              RANGE     );
